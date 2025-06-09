use std::path::Path;

use nu_ansi_term::Color;

use crate::cli::CliSync;

impl CliSync {
    pub async fn exec(self, instance: &str, config: crate::config::UnresolvedConfig, config_path: &Path) -> color_eyre::Result<()> {
        let cols = Color::Blue.bold().paint("::");
        let mut sync = crate::sync::Sync::new(&config_path, Path::new(&config.minecraft.directory))?;
        if self.refresh {
            println!("{cols} {refreshing}", refreshing = Color::Default.bold().paint("Refreshing content index..."));
            sync.refresh().await?;
        }

        if self.upgrade {
            // TODO: upgrade content
            println!("{cols} {upgrading}", upgrading = Color::Default.bold().paint("Upgrading content..."));
        }

        sync.index.write(&config_path)?;
        Ok(())
    }
}
