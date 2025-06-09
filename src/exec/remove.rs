use std::path::Path;

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

        println!("{cols} {removing}", removing = Color::Default.bold().paint(format!("Removing {} packages...", self.packages.len())));

        Ok(())
    }
}
