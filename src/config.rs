use std::{
    path::{Path, PathBuf},
    process::ExitStatus,
};

use color_eyre::{
    Result,
    eyre::{Context, ContextCompat, eyre},
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use startmc_downloader::Download;
use startmc_mojapi::{
    model::{AssetIndex, FABRIC_MAVEN, FabricVerisonGameLoader, VersionManifestV2, VersionPackage},
    util::maven::MavenVersion,
};

use crate::cache::{use_cache_custom_path, use_cached, use_cached_json};

#[derive(Deserialize, Serialize, Debug)]
pub struct MinecraftConfig {
    pub version: String,
    pub directory: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fabric: Option<FabricConfig>,
}

impl MinecraftConfig {
    pub fn get_loader_type(&self) -> Option<String> {
        match self.fabric {
            Some(FabricConfig { .. }) => Some(format!("fabric")),
            None => None,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FabricConfig {
    pub version: String,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct PathsConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub libraries: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assets: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub java: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct ArgsConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mem_min: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mem_max: Option<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub jvm: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub game: String,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Log4jConfig {
    #[default]
    Vanilla,
    None,
    Custom(String),
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UnresolvedConfig {
    pub minecraft: MinecraftConfig,
    #[serde(default)]
    pub paths: PathsConfig,
    #[serde(default)]
    pub args: ArgsConfig,
    #[serde(default)]
    pub log4j: Log4jConfig,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub uuid: Option<String>,
}

impl Log4jConfig {
    pub fn download(&self, base_path: &str, version: &VersionPackage, queue: &mut Vec<Download>) {
        match self {
            Log4jConfig::Vanilla => {
                queue.push(Download::new(
                    &Url::parse(&version.logging.client.file.base.url).unwrap(),
                    &format!("{base_path}/log4j2.vanilla.xml"),
                    None,
                ));
            }
            Log4jConfig::None => {}
            Log4jConfig::Custom(_) => {}
        }
    }

    pub fn get_path(&self, base_path: &str) -> Option<String> {
        match self {
            Log4jConfig::Vanilla => Some(format!("{base_path}/log4j2.vanilla.xml")),
            Log4jConfig::None => None,
            Log4jConfig::Custom(path) => Some(path.to_string()),
        }
    }
}

#[derive(Debug)]
pub enum ModLoader {
    Vanilla,
    Fabric { version: String },
}

async fn use_fabric_launcher_meta(game: &str, loader: &str) -> Result<FabricVerisonGameLoader> {
    let manifest = use_cached(&format!(
        "https://meta.fabricmc.net/v2/versions/loader/{game}/{loader}"
    ))
    .await?;
    let manifest: FabricVerisonGameLoader = serde_json::from_str(&manifest)?;
    Ok(manifest)
}

impl ModLoader {
    pub async fn get_main_class(&self, game_version: &VersionPackage) -> Result<String> {
        Ok(match self {
            ModLoader::Vanilla => game_version.main_class.to_string(),
            ModLoader::Fabric { version } => {
                use_fabric_launcher_meta(&game_version.id, version.as_str())
                    .await?
                    .launcher_meta
                    .main_class
                    .client
            }
        })
    }

    pub async fn build_classpath(&self, libraries_path: &str, game: &str) -> Result<Vec<String>> {
        Ok(match self {
            ModLoader::Vanilla => vec![],
            ModLoader::Fabric { version } => {
                let manifest = use_fabric_launcher_meta(game, version.as_str()).await?;
                manifest
                    .launcher_meta
                    .libraries
                    .client
                    .iter()
                    .chain(manifest.launcher_meta.libraries.common.iter())
                    .map(|it| MavenVersion::parse(&it.name).expect("invalid library name"))
                    .chain([
                        MavenVersion::parse(&manifest.loader.maven).expect("invalid loader name"),
                        MavenVersion::parse(&manifest.intermediary.maven)
                            .expect("invalid intermediary name"),
                        MavenVersion {
                            group: "net.minecrell".to_string(),
                            artifact: "terminalconsoleappender".to_string(),
                            version: "1.3.0".to_string(),
                        },
                    ])
                    .map(|l| format!("{}/{}", libraries_path, l.get_path()))
                    .collect()
            }
        })
    }
}

impl UnresolvedConfig {
    pub fn read(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&contents)?)
    }

    pub fn write(&self, path: &Path) -> Result<()> {
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }

    pub fn find(instance: &str) -> Result<Self> {
        Ok(Self::find_with_path(instance)?.1)
    }

    pub fn find_with_path(instance: &str) -> Result<(PathBuf, Self)> {
        if instance.ends_with(".toml") {
            return Ok((PathBuf::from(instance), Self::read(Path::new(instance))?));
        }

        let paths = vec![
            dirs::config_dir()
                .context("config_dir not found")?
                .join(format!("startmc/{instance}.toml")),
            PathBuf::from(format!("./{instance}.startmc.toml")),
        ];

        for path in paths {
            if path.exists() {
                let me = Self::read(&path)?;
                return Ok((path, me));
            }
        }

        Err(eyre!("Config not found"))
    }

    pub async fn resolve(self) -> Result<Config> {
        Ok(Config {
            version: use_cached_json::<VersionPackage>(
                &use_cached_json::<VersionManifestV2>(
                    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json",
                )
                .await?
                .versions
                .into_iter()
                .find(|v| v.id == self.minecraft.version)
                .unwrap()
                .url,
            )
            .await?,
            java_path: self.paths.java.unwrap_or_else(|| {
                std::env::var("JAVA_HOME")
                    .expect("JAVA_HOME not set, and java_path is not specified in config")
            }),
            libraries_path: self.paths.libraries.unwrap_or_else(|| {
                let path = dirs::data_dir()
                    .expect("data directory not found")
                    .join("startmc/libraries");
                std::fs::create_dir_all(&path).expect("failed to create libraries directory");
                path.to_str()
                    .expect("libraries path is not utf-8")
                    .to_string()
            }),
            assets_path: self.paths.assets.unwrap_or_else(|| {
                let path = dirs::data_dir()
                    .expect("data directory not found")
                    .join("startmc/assets");
                std::fs::create_dir_all(&path).expect("failed to create assets directory");
                path.to_str().expect("assets path is not utf-8").to_string()
            }),
            minecraft_dir: self.minecraft.directory,
            jvm_args: self.args.jvm.split(' ').map(|s| s.to_string()).collect(),
            game_args: self.args.game.split(' ').map(|s| s.to_string()).collect(),
            mem_min: self.args.mem_min.unwrap_or_else(|| "512M".to_string()),
            mem_max: self.args.mem_max.unwrap_or_else(|| "4G".to_string()),
            modloader: if let Some(fabric) = self.minecraft.fabric {
                ModLoader::Fabric {
                    version: fabric.version,
                }
            } else {
                ModLoader::Vanilla
            },
            log4j: self.log4j,
            username: self.username,
            uuid: self.uuid,
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
    pub modloader: ModLoader,
    pub log4j: Log4jConfig,
    pub username: Option<String>,
    pub uuid: Option<String>,
    pub mem_min: String,
    pub mem_max: String,
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

    pub async fn download_libraries(&self, queue: &mut Vec<Download>) -> Result<()> {
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

        self.log4j
            .download(&self.libraries_path, &self.version, queue);

        match &self.modloader {
            ModLoader::Vanilla => {}
            ModLoader::Fabric { version } => {
                let manifest = use_fabric_launcher_meta(&self.version.id, &version).await?;
                for lib in manifest
                    .launcher_meta
                    .libraries
                    .client
                    .iter()
                    .chain(manifest.launcher_meta.libraries.common.iter())
                    .map(|it| MavenVersion::parse(&it.name).expect("invalid library name"))
                    .chain([
                        MavenVersion::parse(&manifest.loader.maven).expect("invalid loader name"),
                        MavenVersion::parse(&manifest.intermediary.maven)
                            .expect("invalid intermediary name"),
                        MavenVersion {
                            group: "net.minecrell".to_string(),
                            artifact: "terminalconsoleappender".to_string(),
                            version: "1.3.0".to_string(),
                        },
                    ])
                {
                    let path = libs_path.join(&lib.get_path());
                    if path.try_exists().unwrap_or(false) {
                        trace!("library {} already downloaded", lib);
                        continue;
                    }
                    std::fs::create_dir_all(path.parent().unwrap())
                        .expect("failed to create directory");
                    queue.push(Download::new(
                        &Url::parse(&lib.get_url(if lib.group == "net.minecrell" {
                            "https://repo1.maven.org/maven2"
                        } else {
                            FABRIC_MAVEN
                        }))
                        .unwrap(),
                        path.to_str().unwrap(),
                        Some(lib.to_string()),
                    ));
                }
            }
        }

        Ok(())
    }

    pub async fn download_assets(&self, queue: &mut Vec<Download>) -> Result<()> {
        let index_path = Path::new(&self.assets_path)
            .join("indexes")
            .join(format!("{}.json", self.version.asset_index.id));
        let asset_index = use_cache_custom_path(&self.version.asset_index.url, &index_path).await?;
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

    pub async fn args(&self) -> Result<JavaArgs> {
        let mut args = JavaArgs {
            jvm: vec![
                format!("-Xms{}", self.mem_min),
                format!("-Xmx{}", self.mem_max),
                "-cp".to_string(),
                [self.get_client_jar_path().to_str().unwrap().to_string()]
                    .into_iter()
                    .chain(
                        self.modloader
                            .build_classpath(&self.libraries_path, &self.version.id)
                            .await?,
                    )
                    .chain(
                        self.version
                            .libraries
                            .iter()
                            .filter(|l| {
                                l.check()
                                    && if matches!(self.modloader, ModLoader::Fabric { .. }) {
                                        !l.name.contains("ow2.asm:asm")
                                    } else {
                                        true
                                    }
                            })
                            .map(|l| {
                                format!(
                                    "{}/{}",
                                    self.libraries_path,
                                    l.downloads.artifact.path.as_str()
                                )
                            }),
                    )
                    .collect::<Vec<_>>()
                    .join(":"),
            ],
            main: self.modloader.get_main_class(&self.version).await?,
            game: vec![
                "--version".to_string(),
                self.version.id.clone(),
                "--gameDir".to_string(),
                self.minecraft_dir.clone(),
                "--assetsDir".to_string(),
                self.assets_path.clone(),
                "--assetIndex".to_string(),
                self.version.asset_index.id.clone(),
                "--uuid".to_string(),
                self.uuid
                    .as_deref()
                    .unwrap_or("12345678-1234-1234-1234-123456789012")
                    .to_string(),
                "--accessToken".to_string(),
                "0".to_string(),
                "--userType".to_string(),
                "msa".to_string(),
                "--versionType".to_string(),
                "release".to_string(),
            ],
        };

        if self.jvm_args.len() > 0 {
            args.jvm.extend(
                self.jvm_args
                    .iter()
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| s.to_string()),
            );
        }
        let log4j_path = self.log4j.get_path(&self.libraries_path);
        if let Some(path) = log4j_path {
            args.jvm.push(format!("-Dlog4j.configurationFile={path}"));
        }

        if let Some(username) = self.username.as_deref() {
            args.game.push("--username".to_string());
            args.game.push(username.to_string());
        }

        if self.game_args.len() > 0 {
            args.game.extend(
                self.game_args
                    .iter()
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| s.to_string()),
            );
        }

        Ok(args)
    }

    pub async fn start(&self) -> Result<ExitStatus> {
        let mut cmd =
            std::process::Command::new(Path::new(&self.java_path).join("bin").join("java"));
        cmd.current_dir(&self.minecraft_dir);

        let args = self.args().await?;
        debug!("FINAL ARGUMENTS: {:#?}", args);
        cmd.args(args.jvm);
        cmd.arg(args.main);
        cmd.args(args.game);

        Ok(tokio::task::spawn_blocking(move || {
            let mut child = cmd.spawn()?;
            Ok::<_, color_eyre::Report>(child.wait()?)
        })
        .await
        .context("cmd panic")??)
    }
}

#[derive(Debug, Clone)]
pub struct JavaArgs {
    pub jvm: Vec<String>,
    pub main: String,
    pub game: Vec<String>,
}
