use std::path::PathBuf;

use color_eyre::eyre::{Context, ContextCompat};
use owo_colors::OwoColorize;
use startmc_mojapi::model::{
    FABRIC_VERSIONS_GAME, FabricVersionsGame, VERSION_MANIFEST_V2, VersionManifestV2,
};

use crate::{
    cache::use_cached_json,
    cli::CliInit,
    config::*,
    util::{arrow_warn, ask_yn, green_arrow},
};

impl CliInit {
    pub async fn exec(self, instance: &str) -> color_eyre::Result<()> {
        let dialoguer_theme = crate::util::dialoguer_theme();

        // ~/.local/share/startmc
        let share_startmc = format!(
            "{}/startmc",
            dirs::data_dir().context("no config dir")?.display()
        );
        let arrow = green_arrow();
        println!(
            "{arrow} {text} {instance}",
            text = "Creating instance".bold()
        );
        let versions = use_cached_json::<VersionManifestV2>(VERSION_MANIFEST_V2).await?;
        let version_ids = versions
            .versions
            .iter()
            .map(|v| v.id.as_str())
            .collect::<Vec<_>>();

        let minecraft_version = if let Some(version) = self.version {
            version
        } else {
            versions.versions[dialoguer::FuzzySelect::with_theme(&dialoguer_theme)
                .with_prompt("Minecraft version")
                .items(&version_ids)
                .default(0)
                .max_length(10)
                .interact()?]
            .id
            .clone()
        };

        debug!("Minecraft version: {minecraft_version}");

        let directory = if let Some(directory) = self.directory {
            directory
        } else {
            dialoguer::Input::with_theme(&dialoguer_theme)
                .with_prompt("Instance directory")
                .default(format!("{share_startmc}/instances/{instance}"))
                // TODO: completion
                .interact_text()?
        };

        let dir_path = PathBuf::from(&directory);
        tokio::task::spawn_blocking(move || std::fs::create_dir_all(dir_path))
            .await
            .context("fs panic")??;

        let modloader = match self.fabric {
            Some(fabric) => ModLoader::Fabric { version: fabric },
            None if self.vanilla => ModLoader::Vanilla,
            _ => {
                let loader = dialoguer::Select::with_theme(&dialoguer_theme)
                    .with_prompt("Mod loader")
                    .items(&["Vanilla", "Fabric"])
                    .default(0)
                    .interact()?;
                match loader {
                    0 => ModLoader::Vanilla,
                    1 => {
                        let fabric_versions = use_cached_json::<FabricVersionsGame>(&format!(
                            "{}/{minecraft_version}",
                            FABRIC_VERSIONS_GAME
                        ))
                        .await?;
                        let version_ids = fabric_versions
                            .iter()
                            .map(|v| v.loader.version.version.as_str())
                            .collect::<Vec<_>>();
                        let fabric_version_id =
                            dialoguer::FuzzySelect::with_theme(&dialoguer_theme)
                                .with_prompt("Fabric version")
                                .default(0)
                                .max_length(10)
                                .items(&version_ids)
                                .interact()?;

                        ModLoader::Fabric {
                            version: version_ids[fabric_version_id].to_string(),
                        }
                    }
                    _ => unreachable!(),
                }
            }
        };

        debug!("Modloader: {modloader:?}");

        let java_path = if let Some(java) = self.java {
            Some(java)
        } else {
            let input =
                dialoguer::Input::with_theme(&dialoguer_theme).with_prompt("Java path (optional)");
            let text: String = (if let Some(java_home) = std::env::var("JAVA_HOME").ok() {
                input.default(java_home)
            } else {
                input
            })
            .allow_empty(true)
            // TODO: completion
            .interact_text()?;
            if text.trim().is_empty() {
                None
            } else {
                Some(text)
            }
        };

        let username = if let Some(username) = self.username {
            Some(username)
        } else {
            let text: String = dialoguer::Input::with_theme(&dialoguer_theme)
                .with_prompt("Username (optional)")
                .allow_empty(true)
                // TODO: completion
                .interact_text()?;
            if text.trim().is_empty() {
                None
            } else {
                Some(text)
            }
        };

        let config = UnresolvedConfig {
            args: ArgsConfig::default(),
            log4j: Log4jConfig::default(),
            minecraft: MinecraftConfig {
                directory,
                fabric: if let ModLoader::Fabric { version } = modloader {
                    Some(FabricConfig { version })
                } else {
                    None
                },
                version: minecraft_version,
            },
            paths: PathsConfig {
                java: java_path,
                libraries: self.libraries,
                ..Default::default()
            },
            username,
            uuid: None,
        };

        let s = toml::to_string_pretty(&config).unwrap();

        println!(
            "{arrow} {msg}\n{s}",
            msg = "Final config:".bold(),
        );

        let config_path = dirs::config_dir()
            .context("config_dir not found")?
            .join(format!("startmc/{instance}.toml"));

        if ask_yn(format!(
            "Write config to {config_path}?",
            config_path = config_path.display()
        ))? {
            let cfg_path = config_path.clone();
            tokio::task::spawn_blocking(move || std::fs::write(cfg_path, s))
                .await
                .context("fs panic")??;
            println!(
                "{arrow} {msg} {config_path}",
                arrow = green_arrow(),
                msg = "Successfully wrote config to".bold(),
                config_path = config_path.display()
            );
        } else {
            arrow_warn("Not writing config file.");
        }

        Ok(())
    }
}
