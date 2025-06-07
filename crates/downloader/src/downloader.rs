//! Represents the download controller.

use crate::download::{Download, Status, Summary};
use futures::stream::{self, StreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use reqwest::{
    StatusCode,
    header::{HeaderMap, HeaderValue, IntoHeaderName, RANGE},
};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use std::{path::PathBuf, sync::Arc};
use tokio::{fs::OpenOptions, io::AsyncWriteExt};
use tracing::debug;

mod builder;
mod style;

pub use builder::*;
pub use style::*;

pub struct TimeTrace;

/// Represents the download controller.
///
/// A downloader can be created via its builder:
///
/// ```rust
/// # fn main()  {
/// use trauma::downloader::DownloaderBuilder;
///
/// let d = DownloaderBuilder::new().build();
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Downloader {
    /// Number of retries per downloaded file.
    retries: u32,
    /// Number of maximum concurrent downloads.
    concurrent_downloads: usize,
    /// Downloader style options.
    style_options: StyleOptions,
    /// Resume the download if necessary and possible.
    resumable: bool,
    /// Custom HTTP headers.
    headers: Option<HeaderMap>,
}

impl Downloader {
    const DEFAULT_RETRIES: u32 = 3;
    const DEFAULT_CONCURRENT_DOWNLOADS: usize = 32;

    /// Starts the downloads.
    pub async fn download(&self, downloads: &[Download]) -> Vec<Summary> {
        self.download_inner(downloads, None).await
    }

    /// Starts the downloads with proxy.
    pub async fn download_with_proxy(
        &self,
        downloads: &[Download],
        proxy: reqwest::Proxy,
    ) -> Vec<Summary> {
        self.download_inner(downloads, Some(proxy)).await
    }

    /// Starts the downloads.
    pub async fn download_inner(
        &self,
        downloads: &[Download],
        proxy: Option<reqwest::Proxy>,
    ) -> Vec<Summary> {
        // Prepare the HTTP client.
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(self.retries);

        let mut inner_client_builder = reqwest::Client::builder();
        if let Some(proxy) = proxy {
            inner_client_builder = inner_client_builder.proxy(proxy);
        }
        if let Some(headers) = &self.headers {
            inner_client_builder = inner_client_builder.default_headers(headers.clone());
        }

        let inner_client = inner_client_builder.build().unwrap();

        let client = ClientBuilder::new(inner_client)
            // Retry failed requests.
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        // Prepare the progress bar.
        let multi = match self.style_options.clone().is_enabled() {
            true => Arc::new(MultiProgress::new()),
            false => Arc::new(MultiProgress::with_draw_target(ProgressDrawTarget::hidden())),
        };
        let main = Arc::new(
            multi.add(
                self.style_options
                    .main
                    .clone()
                    .to_progress_bar(downloads.len() as u64),
            ),
        );
        main.tick();

        // Download the files asynchronously.
        let summaries = stream::iter(downloads)
            .map(|d| self.fetch(&client, d, multi.clone(), main.clone()))
            .buffer_unordered(self.concurrent_downloads)
            .collect::<Vec<_>>()
            .await;

        // Finish the progress bar.
        if self.style_options.main.clear {
            main.finish_and_clear();
        } else {
            main.finish();
        }

        // Return the download summaries.
        summaries
    }

    /// Fetches the files and write them to disk.
    async fn fetch(
        &self,
        client: &ClientWithMiddleware,
        download: &Download,
        multi: Arc<MultiProgress>,
        main: Arc<ProgressBar>,
    ) -> Summary {
        // Create a download summary.
        let mut size_on_disk: u64 = 0;
        let mut content_length: Option<u64> = None;
        let mut can_resume = false;
        let mut summary = Summary::new(
            download.clone(),
            StatusCode::BAD_REQUEST,
            size_on_disk,
            can_resume,
        );
        let output = PathBuf::from(&download.filename);
        if !output.exists() {
            if let Err(e) = std::fs::create_dir_all(output.parent().unwrap()) {
                return summary.fail(e);
            }
        }

        // If resumable is turned on...
        if self.resumable {
            can_resume = match download.is_resumable(client).await {
                Ok(r) => r,
                Err(e) => {
                    return summary.fail(e);
                }
            };

            // Check if there is a file on disk already.
            if can_resume && output.exists() {
                debug!("A file with the same name already exists at the destination.");
                // If so, check file length to know where to restart the download from.
                size_on_disk = match output.metadata() {
                    Ok(m) => m.len(),
                    Err(e) => {
                        return summary.fail(e);
                    }
                };

                // Retrieve the download size from the header if possible.
                content_length = match download.content_length(client).await {
                    Ok(l) => l,
                    Err(e) => {
                        return summary.fail(e);
                    }
                };
            }

            // Update the summary accordingly.
            summary.set_resumable(can_resume);
        }

        // If resumable is turned on...
        // Request the file.
        debug!("Fetching {}", &download.url);
        let mut req = client.get(download.url.clone());
        if self.resumable && can_resume {
            req = req.header(RANGE, format!("bytes={}-", size_on_disk));
        }

        // Add extra headers if needed.
        if let Some(ref h) = self.headers {
            req = req.headers(h.to_owned());
        }

        // Ensure there was no error while sending the request.
        let mut res = match req.send().await {
            Ok(res) => res,
            Err(e) => {
                return summary.fail(e);
            }
        };

        // Check wether or not we need to download the file.
        if let Some(content_length) = content_length {
            if content_length == size_on_disk {
                return summary.with_status(Status::Skipped(
                    "the file was already fully downloaded".into(),
                ));
            }
        }

        // Check the status for errors.
        match res.error_for_status_ref() {
            Ok(_res) => (),
            Err(e) => return summary.fail(e),
        };

        // Update the summary with the collected details.
        let size = content_length.unwrap_or_default() + size_on_disk;
        let status = res.status();
        summary = Summary::new(download.clone(), status, size, can_resume);

        // If there is nothing else to download for this file, we can return.
        if size_on_disk > 0 && size == size_on_disk {
            return summary.with_status(Status::Skipped(
                "the file was already fully downloaded".into(),
            ));
        }

        // Create the progress bar.
        // If the download is being resumed, the progress bar position is
        // updated to start where the download stopped before.
        let pb = multi.add(
            self.style_options
                .child
                .clone()
                .to_progress_bar(size)
                .with_position(size_on_disk)
                .with_message(
                    download
                        .name
                        .as_ref()
                        .map(|it| it.to_string())
                        .unwrap_or_else(|| {
                            output.file_name().unwrap().to_string_lossy().to_string()
                        }),
                ),
        );

        debug!("Creating destination file {:?}", &output);
        let mut file = match OpenOptions::new()
            .create(true)
            .write(true)
            .append(can_resume)
            .open(output)
            .await
        {
            Ok(file) => file,
            Err(e) => {
                return summary.fail(e);
            }
        };

        let mut final_size = size_on_disk;

        // Download the file chunk by chunk.
        debug!("Retrieving chunks...");
        while let Some(mut chunk) = match res.chunk().await {
            Ok(chunk) => chunk,
            Err(e) => {
                return summary.fail(e);
            }
        } {
            // Retrieve chunk.
            let chunk_size = chunk.len() as u64;
            final_size += chunk_size;
            pb.inc(chunk_size);

            // Write the chunk to disk.
            match file.write_all_buf(&mut chunk).await {
                Ok(_res) => (),
                Err(e) => {
                    return summary.fail(e);
                }
            };
        }

        // Finish the progress bar once complete, and optionally remove it.
        if self.style_options.child.clear {
            pb.finish_and_clear();
        } else {
            pb.finish();
        }

        // Advance the main progress bar.
        main.inc(1);

        // Create a new summary with the real download size
        let summary = Summary::new(download.clone(), status, final_size, can_resume);
        // Return the download summary.
        summary.with_status(Status::Success)
    }
}
