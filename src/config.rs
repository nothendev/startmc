use std::path::Path;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub version: String,
    #[serde(default)]
    pub libraries_path: Option<String>,
    #[serde(default)]
    pub java_path: Option<String>,
}

impl Config {
    pub fn read(path: &Path) -> Result<Self, std::io::Error> {
        let contents = std::fs::read_to_string(path)?;
        toml::from_str(&contents).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    pub fn resolve_paths(&mut self) {
        if self.java_path.is_none() {
            self.java_path = Some(std::env::var("JAVA_HOME").expect("JAVA_HOME not set, and java_path is not specified in config"));
        }

        if self.libraries_path.is_none() {
            let path = dirs::data_dir().expect("data directory not found").join("startmc/libraries");
            std::fs::create_dir_all(&path).expect("failed to create libraries directory");
            self.libraries_path = Some(path.to_str().expect("libraries path is not utf-8").to_string());
        }
    }
}
