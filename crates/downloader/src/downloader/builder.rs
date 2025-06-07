use super::*;

/// A builder used to create a [`Downloader`].
///
/// ```rust
/// # fn main()  {
/// use trauma::downloader::DownloaderBuilder;
///
/// let d = DownloaderBuilder::new().retries(5).directory("downloads".into()).build();
/// # }
/// ```
pub struct DownloaderBuilder(Downloader);

impl DownloaderBuilder {
    /// Creates a builder with the default options.
    pub fn new() -> Self {
        DownloaderBuilder::default()
    }

    /// Convenience function to hide the progress bars.
    pub fn hidden() -> Self {
        let d = DownloaderBuilder::default();
        d.style_options(StyleOptions::new(
            ProgressBarOpts::hidden(),
            ProgressBarOpts::hidden(),
        ))
    }

    /// Set the number of retries per [`Download`].
    pub fn retries(mut self, retries: u32) -> Self {
        self.0.retries = retries;
        self
    }

    /// Set the number of concurrent [`Download`]s.
    pub fn concurrent_downloads(mut self, concurrent_downloads: usize) -> Self {
        self.0.concurrent_downloads = concurrent_downloads;
        self
    }

    /// Set the downloader style options.
    pub fn style_options(mut self, style_options: StyleOptions) -> Self {
        self.0.style_options = style_options;
        self
    }

    fn new_header(&self) -> HeaderMap {
        match self.0.headers {
            Some(ref h) => h.to_owned(),
            _ => HeaderMap::new(),
        }
    }

    /// Add the http headers.
    ///
    /// You need to pass in a `HeaderMap`, not a `HeaderName`.
    /// `HeaderMap` is a set of http headers.
    ///
    /// You can call `.headers()` multiple times and all `HeaderMap` will be merged into a single one.
    ///
    /// # Example
    ///
    /// ```
    /// use reqwest::header::{self, HeaderValue, HeaderMap};
    /// use trauma::downloader::DownloaderBuilder;
    ///
    /// let ua = HeaderValue::from_str("curl/7.87").expect("Invalid UA");
    ///
    /// let builder = DownloaderBuilder::new()
    ///     .headers(HeaderMap::from_iter([(header::USER_AGENT, ua)]))
    ///     .build();
    /// ```
    ///
    /// See also [`header()`].
    ///
    /// [`header()`]: DownloaderBuilder::header
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        let mut new = self.new_header();
        new.extend(headers);

        self.0.headers = Some(new);
        self
    }

    /// Add the http header
    ///
    /// # Example
    ///
    /// You can use the `.header()` chain to add multiple headers
    ///
    /// ```
    /// use reqwest::header::{self, HeaderValue};
    /// use trauma::downloader::DownloaderBuilder;
    ///
    /// const FIREFOX_UA: &str =
    /// "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/109.0";
    ///
    /// let ua = HeaderValue::from_str(FIREFOX_UA).expect("Invalid UA");
    /// let auth = HeaderValue::from_str("Basic aGk6MTIzNDU2Cg==").expect("Invalid auth");
    ///
    /// let builder = DownloaderBuilder::new()
    ///     .header(header::USER_AGENT, ua)
    ///     .header(header::AUTHORIZATION, auth)
    ///     .build();
    /// ```
    ///
    /// If you need to pass in a `HeaderMap`, instead of calling `.header()` multiple times.
    /// See also [`headers()`].
    ///
    /// [`headers()`]: DownloaderBuilder::headers
    pub fn header<K: IntoHeaderName>(mut self, name: K, value: HeaderValue) -> Self {
        let mut new = self.new_header();

        new.insert(name, value);

        self.0.headers = Some(new);
        self
    }

    /// Create the [`Downloader`] with the specified options.
    pub fn build(self) -> Downloader {
        Downloader {
            retries: self.0.retries,
            concurrent_downloads: self.0.concurrent_downloads,
            style_options: self.0.style_options,
            resumable: self.0.resumable,
            headers: self.0.headers,
        }
    }
}

impl Default for DownloaderBuilder {
    fn default() -> Self {
        Self(Downloader {
            retries: Downloader::DEFAULT_RETRIES,
            concurrent_downloads: Downloader::DEFAULT_CONCURRENT_DOWNLOADS,
            style_options: StyleOptions::default(),
            resumable: true,
            headers: None,
        })
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_builder_defaults() {
        let d = DownloaderBuilder::new().build();
        assert_eq!(d.retries, Downloader::DEFAULT_RETRIES);
        assert_eq!(
            d.concurrent_downloads,
            Downloader::DEFAULT_CONCURRENT_DOWNLOADS
        );
    }
}
