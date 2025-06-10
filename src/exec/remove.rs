use std::path::Path;

use color_eyre::eyre::{Context, ContextCompat};
use nu_ansi_term::Color;

use crate::cli::CliRemove;

impl CliRemove {
    pub async fn exec(
        self,
        instance: &str,
        config: crate::config::UnresolvedConfig,
        config_path: &Path,
    ) -> color_eyre::Result<()> {
        let mut sync =
            crate::sync::Sync::new(&config_path, Path::new(&config.minecraft.directory))?;
        sync.maybe_refresh().await?;
        let cols = Color::Blue.bold().paint("::");

        println!(
            "{cols} {removing}",
            removing = Color::Default.bold().paint(format!(
                "{} {} packages...",
                if self.disable {
                    "Disabling"
                } else {
                    "Removing"
                },
                self.packages.len()
            ))
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

        sync.index.write(config_path)?;

        Ok(())
    }
}
