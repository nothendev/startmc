use color_eyre::Result;

pub async fn use_cached(url: &str) -> Result<String> {
    let path = get_cached_path(url);
    if path.exists() {
        let contents = std::fs::read_to_string(path)?;
        Ok(contents)
    } else {
        std::fs::create_dir_all(path.parent().expect("path parent"))?;
        let res = reqwest::get(url).await?;
        let contents = res.text().await?;
        std::fs::write(path, &contents)?;
        Ok(contents)
    }
}

pub fn get_cached_path(url: &str) -> std::path::PathBuf {
    let cache = dirs::cache_dir()
        .expect("cache directory not found")
        .join("startmc");
    let temp = url.split('/').collect::<Vec<_>>();
    let [.., a, b] = temp.as_slice() else {
        unreachable!(
            "we're only caching piston-meta urls which are /v1/packages/HASH/WHATEVER.json"
        )
    };
    let path = cache.join(a).join(b);
    path
}

pub async fn use_cache_custom_path(
    url: &str,
    path: &std::path::Path,
) -> Result<String> {
    if path.exists() {
        let contents = std::fs::read_to_string(path)?;
        Ok(contents)
    } else {
        std::fs::create_dir_all(path.parent().expect("path parent"))?;
        let res = reqwest::get(url).await?;
        let contents = res.text().await?;
        std::fs::write(path, &contents)?;
        Ok(contents)
    }
}

pub async fn use_cached_json<T: serde::de::DeserializeOwned>(url: &str) -> Result<T> {
    let contents = use_cached(url).await?;
    Ok(serde_json::from_str(&contents)?)
}
