use indicatif::ProgressBar;
use startmc_mojapi::model::{
    FABRIC_VERSIONS_GAME, FABRIC_VERSIONS_LOADER, FabricVersionsGame, FabricVersionsLoader,
    VERSION_MANIFEST_V2, VersionManifestV2,
};

use crate::{cache::use_cached_json, cli::CliInit, config::ModLoader, util::SpinExt};

impl CliInit {
    pub async fn exec(self, instance: &str) -> color_eyre::Result<()> {
        let dialoguer_theme = crate::util::dialoguer_theme();
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

        let modloader = match self.fabric {
            Some(fabric) => ModLoader::Fabric { version: fabric },
            None => {
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
                        .spin_until_ready(
                            ProgressBar::new_spinner().with_message("Get fabric versions..."),
                        )
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

        let java_path = {
            let input = dialoguer::Input::with_theme(&dialoguer_theme).with_prompt("Java path");
            if let Some(java_home) = std::env::var("JAVA_HOME").ok() {
                input.default(java_home)
            } else {
                input
            }
            .interact_text()
        }?;

        Ok(())
    }
}
