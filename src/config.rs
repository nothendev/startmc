use std::{
    path::{Path, PathBuf},
    process::ExitStatus,
};

use reqwest::Url;
use serde::Deserialize;
use startmc_downloader::Download;
use startmc_mojapi::model::{AssetIndex, VersionManifestV2, VersionPackage};

use crate::cache::{get_cached, get_cached_with_custom_path};

#[derive(Deserialize, Debug)]
pub struct UnresolvedConfig {
    pub version: String,
    #[serde(default)]
    pub libraries_path: Option<String>,
    #[serde(default)]
    pub java_path: Option<String>,
    #[serde(default)]
    pub assets_path: Option<String>,
    pub minecraft_dir: String,
    #[serde(default)]
    pub jvm_args: String,
    #[serde(default)]
    pub game_args: String,
}

impl UnresolvedConfig {
    pub fn read(path: &Path) -> Result<Self, std::io::Error> {
        let contents = std::fs::read_to_string(path)?;
        toml::from_str(&contents)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    pub fn find(instance: &str) -> Result<Self, std::io::Error> {
        let paths = vec![
            dirs::config_dir()
                .expect("config directory not found")
                .join(format!("startmc/{instance}.toml")),
            PathBuf::from(format!("./{instance}.startmc.toml")),
        ];

        for path in paths {
            if path.exists() {
                return Self::read(&path);
            }
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "config not found",
        ))
    }

    pub async fn resolve(self, rq: &reqwest::Client) -> Result<Config, std::io::Error> {
        Ok(Config {
            version: serde_json::from_str::<VersionPackage>(
                &get_cached(
                    &serde_json::from_str::<VersionManifestV2>(
                        &get_cached(
                            "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json",
                            rq,
                        )
                        .await?,
                    )
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
                    .versions
                    .into_iter()
                    .find(|v| v.id == self.version)
                    .unwrap()
                    .url,
                    rq,
                )
                .await?,
            )
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?,
            java_path: self.java_path.unwrap_or_else(|| {
                std::env::var("JAVA_HOME")
                    .expect("JAVA_HOME not set, and java_path is not specified in config")
            }),
            libraries_path: self.libraries_path.unwrap_or_else(|| {
                let path = dirs::data_dir()
                    .expect("data directory not found")
                    .join("startmc/libraries");
                std::fs::create_dir_all(&path).expect("failed to create libraries directory");
                path.to_str()
                    .expect("libraries path is not utf-8")
                    .to_string()
            }),
            assets_path: self.assets_path.unwrap_or_else(|| {
                let path = dirs::data_dir()
                    .expect("data directory not found")
                    .join("startmc/assets");
                std::fs::create_dir_all(&path).expect("failed to create assets directory");
                path.to_str().expect("assets path is not utf-8").to_string()
            }),
            minecraft_dir: self.minecraft_dir,
            jvm_args: self.jvm_args.split(' ').map(|s| s.to_string()).collect(),
            game_args: self.game_args.split(' ').map(|s| s.to_string()).collect(),
        })
    }
}

#[derive(Debug)]
pub struct Config {
    pub version: VersionPackage,
    pub libraries_path: String,
    pub java_path: String,
    pub minecraft_dir: String,
    pub assets_path: String,
    pub jvm_args: Vec<String>,
    pub game_args: Vec<String>,
}

impl Config {
    pub fn get_client_jar_path(&self) -> PathBuf {
        Path::new(&self.libraries_path).join(format!(
            "net/minecraft/client/{id}/minecraft-{id}-client.jar",
            id = self.version.id
        ))
    }

    pub fn download_client(&self, queue: &mut Vec<Download>) {
        let path = self.get_client_jar_path();
        if path.try_exists().unwrap_or(false) {
            return;
        }

        std::fs::create_dir_all(path.parent().unwrap()).expect("failed to create directory");
        queue.push(Download::new(
            &Url::parse(&self.version.downloads.client.url).unwrap(),
            path.to_str().unwrap(),
            Some(format!("minecraft-{}-client.jar", self.version.id)),
        ));
    }

    pub fn download_libraries(&self, queue: &mut Vec<Download>) {
        let libs_path = Path::new(&self.libraries_path);
        for lib in &self.version.libraries {
            if !lib.check() {
                continue;
            }
            trace!("library: {}", lib.downloads.artifact.path);
            let path = libs_path.join(&lib.downloads.artifact.path);
            if path.try_exists().unwrap_or(false) {
                trace!("library {} already downloaded", lib.downloads.artifact.path);
                continue;
            }
            std::fs::create_dir_all(path.parent().unwrap()).expect("failed to create directory");
            let d = Download::new(
                &Url::parse(&lib.downloads.artifact.base.url).unwrap(),
                path.to_str().unwrap(),
                Some(
                    lib.downloads
                        .artifact
                        .path
                        .split('/')
                        .last()
                        .unwrap()
                        .to_string(),
                ),
            );
            trace!(
                "downloading library {}: {d:#?}",
                lib.downloads.artifact.path
            );
            queue.push(d);
        }
    }

    pub async fn download_assets(
        &self,
        queue: &mut Vec<Download>,
        rq: &reqwest::Client,
    ) -> Result<(), std::io::Error> {
        let index_path = Path::new(&self.assets_path)
            .join("indexes")
            .join(&self.version.asset_index.id);
        let asset_index =
            get_cached_with_custom_path(&self.version.asset_index.url, &rq, &index_path).await?;
        let asset_index: AssetIndex = serde_json::from_str(&asset_index)?;
        for asset in asset_index.objects.values() {
            let path = Path::new(&self.assets_path)
                .join("objects")
                .join(&asset.hash[..2])
                .join(&asset.hash);
            if path.try_exists().unwrap_or(false) {
                trace!("asset {} already downloaded", asset.hash);
                continue;
            }
            std::fs::create_dir_all(path.parent().unwrap()).expect("failed to create directory");
            queue.push(Download::new(
                &Url::parse(&format!(
                    "https://resources.download.minecraft.net/{}/{}",
                    &asset.hash[..2],
                    asset.hash
                ))
                .unwrap(),
                path.to_str().unwrap(),
                Some(format!("assets:{}", asset.hash)),
            ));
        }
        Ok(())
    }

    pub fn start(&self) -> Result<ExitStatus, std::io::Error> {
        let mut cmd =
            std::process::Command::new(Path::new(&self.java_path).join("bin").join("java"));
        cmd.current_dir(&self.minecraft_dir);
        cmd.arg("-cp");
        cmd.arg(
            [self.get_client_jar_path().to_str().unwrap().to_string()]
                .into_iter()
                .chain(self.version.libraries.iter().map(|l| {
                    format!(
                        "{}/{}",
                        self.libraries_path,
                        l.downloads.artifact.path.as_str()
                    )
                }))
                .collect::<Vec<_>>()
                .join(":"),
        );
        cmd.args(&self.jvm_args);
        cmd.arg(self.version.main_class.as_str());
        cmd.arg("--version");
        cmd.arg(&self.version.id);
        cmd.arg("--gameDir");
        cmd.arg(&self.minecraft_dir);
        cmd.arg("--assetsDir");
        cmd.arg(&self.assets_path);
        cmd.arg("--assetIndex");
        cmd.arg(&self.version.asset_index.id);
        cmd.arg("--uuid");
        cmd.arg("12345678-1234-1234-1234-123456789012");
        cmd.arg("--accessToken");
        cmd.arg("0");
        cmd.arg("--userType");
        cmd.arg("msa");
        cmd.arg("--versionType");
        cmd.arg("release");
        cmd.args(&self.game_args);
        println!("{:#?}", cmd.get_args());
        cmd.status()
    }
}
