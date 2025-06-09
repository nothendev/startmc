use std::{collections::HashMap, path::{Path, PathBuf}};

use color_eyre::Result;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct SyncIndexEntry {
    pub id: String,
    pub version: String,
    pub modrinth_project: Option<String>,
    pub file: String,
    pub disabled: bool,
}

pub struct Sync {
    pub fer: ferinth::Ferinth<()>,
    pub index: SyncIndex,
    pub minecraft_directory: PathBuf,
}

impl Sync {
    pub fn new(normal_config_path: &Path, minecraft_directory: &Path) -> Result<Self> {
        Ok(Self {
            fer: ferinth::Ferinth::<()>::new(
                env!("CARGO_PKG_REPOSITORY"),
                Some(env!("CARGO_PKG_VERSION")),
                Some(env!("CARGO_PKG_HOMEPAGE")),
            ),
            index: SyncIndex::read(normal_config_path)?,
            minecraft_directory: minecraft_directory.to_path_buf(),
        })
    }

    pub async fn refresh(&mut self) -> Result<()> {
        Ok(())
    }

    pub async fn maybe_refresh(&mut self) -> Result<()> {
        if self.index.0.is_empty() {
            self.refresh().await
        } else {
            Ok(())
        }
    }
}

#[derive(Debug)]
pub struct SyncIndex(pub HashMap<String, SyncIndexEntry>);

impl SyncIndex {
    pub fn get_lock_path(normal_config_path: &Path) -> PathBuf {
        // default.toml -> default.lock.toml, default.startmc.toml -> default.lock.startmc.toml
        normal_config_path.with_extension(format!(
            "lock.{}",
            normal_config_path
                .extension()
                .map(|it| it.to_string_lossy().into_owned())
                .unwrap_or_else(|| "toml".to_string())
        ))
    }

    pub fn read(normal_config_path: &Path) -> Result<Self> {
        let path = Self::get_lock_path(normal_config_path);
        if path.exists() {
            let contents = std::fs::read_to_string(path)?;
            Ok(Self(toml::from_str(&contents)?))
        } else {
            Ok(Self(HashMap::new()))
        }
    }

    pub fn write(&self, normal_config_path: &Path) -> Result<()> {
        let contents = toml::to_string(&self.0)?;
        std::fs::write(Self::get_lock_path(normal_config_path), contents)?;
        Ok(())
    }
}
