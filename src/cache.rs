pub async fn get_cached(url: &str, rq: &reqwest::Client) -> Result<String, std::io::Error> {
    let path = get_cached_path(url);
    if path.exists() {
        let contents = std::fs::read_to_string(path)?;
        Ok(contents)
    } else {
        std::fs::create_dir_all(path.parent().expect("path parent"))?;
        let res = rq
            .get(url)
            .send()
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let contents = res
            .text()
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
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

pub async fn get_cached_with_custom_path(
    url: &str,
    rq: &reqwest::Client,
    path: &std::path::Path,
) -> Result<String, std::io::Error> {
    if path.exists() {
        let contents = std::fs::read_to_string(path)?;
        Ok(contents)
    } else {
        std::fs::create_dir_all(path.parent().expect("path parent"))?;
        let res = rq
            .get(url)
            .send()
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let contents = res
            .text()
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, &contents)?;
        Ok(contents)
    }
}
