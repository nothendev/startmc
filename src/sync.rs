use std::{
    collections::HashMap,
    fs::DirEntry,
    path::{Path, PathBuf},
};

use color_eyre::{
    Result,
    eyre::{Context, ContextCompat},
};
use ferinth::structures::project::ProjectType;
use serde::{Deserialize, Serialize};
use sha1_smol::Sha1;

use crate::sync::version::VersionTuple;

mod version;

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexEntryKind {
    Mod,
    Resourcepack,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SyncIndexEntry {
    pub id: String,
    pub version: String,
    pub modrinth_project: Option<String>,
    pub modrinth_version_id: Option<String>,
    pub file: String,
    pub disabled: bool,
    pub kind: IndexEntryKind,
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
        let mut hashes = HashMap::new();
        let mods_dir = self.minecraft_directory.join("mods");
        let resourcepacks_dir = self.minecraft_directory.join("resourcepacks");
        let entries = tokio::task::spawn_blocking(|| {
            std::fs::read_dir(mods_dir)?
                .chain(std::fs::read_dir(
                    resourcepacks_dir
                )?)
                .collect::<std::io::Result<Vec<DirEntry>>>()
        })
        .await
        .context("tokio fail")??;

        let entries_len = entries.len();
        for (i, entry) in entries.into_iter().enumerate() {
            if entry.file_type()?.is_dir() {
                continue;
            }
            let path = entry.path();
            let _path = path.clone();
            debug!("({i}/{entries_len}) Hashing {}", path.display());
            let hash = tokio::task::spawn_blocking(|| {
                Ok::<_, std::io::Error>(Sha1::from(std::fs::read(_path)?.as_slice()).hexdigest())
            })
            .await
            .context("tokio fail")??;
            debug!("({i}/{entries_len}) Hashed {}", path.display());
            hashes.insert(path, hash);
        }

        debug!("Refreshing for {} files", hashes.len());

        let versions = self
            .fer
            .version_get_from_multiple_hashes(hashes.values().cloned().collect::<Vec<_>>())
            .await?;

        debug!(
            "Got {} versions, diff = {}",
            versions.len(),
            hashes.len() - versions.len()
        );

        for (path, hash) in hashes.iter() {
            let filename = path.file_name().unwrap().to_str().unwrap();
            let disabled = filename.ends_with(".disabled");
            let index = self
                .index
                .packages
                .iter()
                .enumerate()
                .find(|(i, it)| it.file == filename)
                .map(|it| it.0);
            debug!("Processing {} ({})", filename, hash);
            match versions.get(hash) {
                Some(version) => {
                    debug!("Found version {}", version.version_number);
                    if let Some(index) = index {
                        let package = &mut self.index.packages[index];
                        package.modrinth_project = Some(version.project_id.clone());
                        package.modrinth_version_id = Some(version.id.clone());
                        package.version = version.version_number.clone();
                    } else {
                        let project = self.fer.project_get(&version.project_id).await?;
                        self.index.packages.push(SyncIndexEntry {
                            id: project.slug,
                            disabled,
                            file: filename.to_string(),
                            modrinth_project: Some(version.project_id.clone()),
                            modrinth_version_id: Some(version.id.clone()),
                            version: version.version_number.clone(),
                            kind: match project.project_type {
                                ProjectType::Mod => IndexEntryKind::Mod,
                                ProjectType::Resourcepack => IndexEntryKind::Resourcepack,
                                t => {
                                    error!("unsupported project type: {t:?}, skipping...");
                                    continue;
                                }
                            }
                        });
                    }
                }
                None => {
                    let tuple = VersionTuple::parse(filename).context("parse version")?;
                    let path = Path::new(filename);
                    let kind = match path.extension().and_then(|it| it.to_str()).context("parse extension")?.trim_end_matches(".disabled") {
                        "zip" => IndexEntryKind::Resourcepack,
                        "jar" => IndexEntryKind::Mod,
                        _ => {
                            error!("unsupported file extension: {filename}, skipping...");
                            continue;
                        }
                    };

                    debug!("{filename} not found on modrinth, parsed as {tuple:?} and kind={kind:?}");
                    if let Some(index) = index {
                        debug!("{filename} found in index");
                        let package = &mut self.index.packages[index];
                        package.version = tuple.version;
                    } else {
                        debug!("Inserting {filename} into index");
                        self.index.packages.push(SyncIndexEntry {
                            id: tuple.name,
                            disabled,
                            file: filename.to_string(),
                            modrinth_project: None,
                            modrinth_version_id: None,
                            version: tuple.version,
                            kind
                        });
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn maybe_refresh(&mut self) -> Result<()> {
        if self.index.packages.is_empty() {
            self.refresh().await
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct SyncIndex {
    #[serde(default)]
    pub packages: Vec<SyncIndexEntry>,
}

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
            Ok(toml::from_str(&contents)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn write(&self, normal_config_path: &Path) -> Result<()> {
        let contents = toml::to_string(&self)?;
        std::fs::write(Self::get_lock_path(normal_config_path), contents)?;
        Ok(())
    }
}

impl SyncIndexEntry {
    pub fn disable_and_move(&mut self, prefix: &Path) {
        self.disabled = true;
        let mut path = prefix.join(&self.file);
        path.set_extension(path.extension().unwrap().to_str().unwrap().trim_end_matches(".disabled"), );
        std::fs::rename(path, self.file.clone()).unwrap();
    }
}
