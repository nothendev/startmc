use color_eyre::eyre::Context;
use color_eyre::Result;

pub async fn use_cached(url: &str) -> Result<String> {
    use_cache_custom_path(url, &get_cached_path(url)).await
}

/// Normalize a URL into a file path, then concat it with the ~/.cache/startmc (or whatever platform equivalent) directory.
///
/// Normalization performs 2 steps:
/// 1. Removes the protocol (https://) and the trailing slash (/)
/// 2. Replaces all slashes (/) with double underscores (__) (double underscores are used instead of single ones to avoid conflicts with similar urls if they used underscores)
///
/// So, from `https://meta.fabricmc.net/v2/versions/loader/1.21.4/`, we get `meta.fabricmc.net__v2__versions__loader__1.21.4`
pub fn get_cached_path(url: &str) -> std::path::PathBuf {
    let cache = dirs::cache_dir()
        .expect("cache directory not found")
        .join("startmc");
    let url = url.trim_start_matches("https://").trim_end().trim_end_matches('/');
    debug!("Pure url: {url}");
    let url = url.replace('/', "__");
    debug!("Normalized url: {url}");
    cache.join(url)
}

pub async fn use_cache_custom_path(url: &str, path: &std::path::Path) -> Result<String> {
    if path.exists() {
        debug!("{path} exists! Reading...", path = path.display());
        let contents = std::fs::read_to_string(path)?;
        Ok(contents)
    } else {
        debug!("{path} doesn't exist! Creating...", path = path.display());
        let parent = path.parent().expect("path parent").to_owned();
        tokio::task::spawn_blocking(|| std::fs::create_dir_all(parent)).await.context("tokio fail")??;
        debug!("Downloading {url} to {path}", url = url, path = path.display());
        let res = reqwest::get(url).await?;
        let contents = res.text().await?;
        debug!("Writing {bytes} bytes to {path}", bytes = contents.len(), path = path.display());
        std::fs::write(path, &contents)?;
        Ok(contents)
    }
}

pub async fn use_cached_json<T: serde::de::DeserializeOwned>(url: &str) -> Result<T> {
    let contents = use_cached(url).await?;
    Ok(serde_json::from_str(&contents)?)
}
