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
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressFinish, ProgressStyle};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sha1_smol::Sha1;

mod filter;
pub use filter::*;
mod version;
use startmc_downloader::ProgressBarOpts;
pub use version::VersionTuple;
use version_compare::Cmp;

use crate::util::arrow_error;

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

/// High level operations on sync, like refreshing the index
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
        let mods_dir = self.minecraft_directory.join("mods");
        let resourcepacks_dir = self.minecraft_directory.join("resourcepacks");
        let entries = tokio::task::spawn_blocking(|| {
            std::fs::read_dir(mods_dir)?
                .chain(std::fs::read_dir(resourcepacks_dir)?)
                .collect::<std::io::Result<Vec<DirEntry>>>()
        })
        .await
        .context("tokio fail")??;

        let entries_len = entries.len();
        let hashes: HashMap<PathBuf, String> = entries
            .into_par_iter()
            .progress_count(entries_len as u64)
            .with_style(
                ProgressStyle::default_bar()
                    .template("{wide_msg} [{bar:69}] {percent}%")
                    .unwrap()
                    .progress_chars(ProgressBarOpts::CHARS_HASHTAG),
            )
            .with_message("(1/2) Hashing files...")
            .with_finish(ProgressFinish::AndLeave)
            .filter(|it| !it.file_type().unwrap().is_dir())
            .map(|entry| {
                let path = entry.path();
                let hash =
                    Sha1::from(std::fs::read(&path).expect("fs fail").as_slice()).hexdigest();
                (path, hash)
            })
            .collect();

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

        let progress = ProgressBar::new(hashes.len() as u64).with_finish(ProgressFinish::AndLeave);
        progress.set_style(
            ProgressStyle::default_bar()
                .template("{wide_msg} [{bar:69}] {percent}%")
                .unwrap()
                .progress_chars(ProgressBarOpts::CHARS_HASHTAG),
        );
        progress.set_message("(2/2) Building index...");

        for (path, hash) in hashes.iter() {
            let filename = path.file_name().unwrap().to_str().unwrap();
            let disabled = filename.ends_with(".disabled");
            let index = self
                .index
                .packages
                .iter()
                .enumerate()
                .find(|(_, it)| it.file == filename)
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
                                ProjectType::ResourcePack => IndexEntryKind::Resourcepack,
                                t => {
                                    error!("unsupported project type: {t:?}, skipping...");
                                    continue;
                                }
                            },
                        });
                    }
                }
                None => {
                    let trimmed_filename = filename.trim_end_matches(".disabled");
                    let tuple = VersionTuple::parse(trimmed_filename).context("parse version")?;
                    let kind = if trimmed_filename.ends_with(".jar") {
                        IndexEntryKind::Mod
                    } else if trimmed_filename.ends_with(".zip") {
                        IndexEntryKind::Resourcepack
                    } else {
                        error!("unsupported file extension: {filename}, skipping...");
                        continue;
                    };

                    debug!(
                        "{filename} not found on modrinth, parsed as {tuple:?} and kind={kind:?}"
                    );
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
                            kind,
                        });
                    }
                }
            }
            progress.inc(1);
        }

        progress.finish();

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

/// Low level operations on the sync index and its entries
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

    pub fn find_packages(&self, filter: &SyncFilter) -> Vec<usize> {
        let indices = self
            .packages
            .iter()
            .enumerate()
            .filter_map(|(i, it)| {
                if it.id != filter.name {
                    return None;
                }

                if let Some(version) = &filter.version {
                    match version {
                        VersionFilter::Any => Some(i),
                        VersionFilter::Op(version, Cmp::Eq) => {
                            (it.version.as_str() == version.as_str()).then(|| i)
                        }
                        VersionFilter::Op(version, op) => matches!(
                            version_compare::compare_to(it.version.as_str(), version.as_str(), *op),
                            Ok(true)
                        )
                        .then(|| i),
                    }
                } else {
                    Some(i)
                }
            })
            .collect::<Vec<_>>();

        if filter.version.is_none() && indices.len() > 1 {
            arrow_error(format!(
                "Multiple matches found for {name}, please specify version.",
                name = filter.name
            ));
            return vec![];
        } else {
            indices
        }
    }
}

impl SyncIndexEntry {
    fn get_namespace(&self, prefix: &Path) -> PathBuf {
        match self.kind {
            IndexEntryKind::Mod => prefix.join("mods"),
            IndexEntryKind::Resourcepack => prefix.join("resourcepacks"),
        }
    }

    pub fn disable_and_move(&mut self, prefix: &Path) -> Result<(), std::io::Error> {
        self.disabled = true;
        let ns = self.get_namespace(prefix);
        std::fs::rename(
            ns.join(&self.file),
            ns.join(format!("{}.disabled", self.file)),
        )?;
        self.file = format!("{}.disabled", self.file);
        Ok(())
    }

    pub fn enable_and_move(&mut self, prefix: &Path) -> Result<(), std::io::Error> {
        self.disabled = false;
        let ns = self.get_namespace(prefix);
        let enabled_filename = self.file.trim_end_matches(".disabled");
        std::fs::rename(ns.join(&self.file), ns.join(enabled_filename))?;
        self.file = enabled_filename.to_string();
        Ok(())
    }

    pub fn remove_from_fs(&self, prefix: &Path) -> Result<(), std::io::Error> {
        let ns = self.get_namespace(prefix);
        let path = ns.join(&self.file);
        debug!("remove_from_fs {}", path.display());
        std::fs::remove_file(&path)?;
        Ok(())
    }
}
