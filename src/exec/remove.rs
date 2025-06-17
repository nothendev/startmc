use std::path::Path;

use color_eyre::eyre::Context;
use owo_colors::OwoColorize;

use crate::{cli::CliRemove, util::cols};

impl CliRemove {
    pub async fn exec(
        self,
        instance: &str,
    ) -> color_eyre::Result<()> {
        let (config_path, config) = crate::config::UnresolvedConfig::find_with_path(instance).context("find config")?;
        let mut sync =
            crate::sync::Sync::new(&config_path, Path::new(&config.minecraft.directory))?;
        sync.maybe_refresh().await?;
        let cols = cols();

        println!(
            "{cols} {removing}",
            removing = format!(
                "{} {} packages...",
                if self.disable {
                    "Disabling"
                } else {
                    "Removing"
                },
                self.packages.len()
            ).bold()
        );

        for filter in self.packages {
            let indices = sync.index.find_packages(&filter);

            for i in indices {
                if self.disable && !sync.index.packages[i].disabled {
                    sync.index.packages[i]
                        .disable_and_move(Path::new(&config.minecraft.directory))?;
                } else {
                    let pkg = sync.index.packages.swap_remove(i);
                    pkg.remove_from_fs(Path::new(&config.minecraft.directory))
                        .context("remove from fs")?;
                }
            }
        }

        sync.index.write(&config_path)?;

        Ok(())
    }
}
