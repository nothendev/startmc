use std::path::Path;

use color_eyre::eyre::Context;
use nu_ansi_term::Color;
use reqwest::Url;
use startmc_downloader::DownloaderBuilder;

use crate::cli::{CliUpgrade, UpgradeKind};

impl CliUpgrade {
    pub async fn exec(
        self,
        instance: &str,
    ) -> color_eyre::Result<()> {
        let (config_path, config) = crate::config::UnresolvedConfig::find_with_path(instance).context("find config")?;
        let cols = Color::Blue.bold().paint("::");
        let mut queue: Vec<startmc_downloader::Download> = vec![];
        let dest = match &self.kind {
            UpgradeKind::Mod => {
                println!(
                    "{cols} {downloading} {amount} mods",
                    downloading = Color::Default.bold().paint("Downloading"),
                    amount = Color::Green.paint(self.packages.len().to_string())
                );

                Path::new(&config.minecraft.directory).join("mods")
            }
            UpgradeKind::Resourcepack => {
                println!(
                    "{cols} {downloading} {amount} resourcepacks",
                    downloading = Color::Default.bold().paint("Downloading"),
                    amount = Color::Green.paint(self.packages.len().to_string())
                );

                Path::new(&config.minecraft.directory).join("resourcepacks")
            }
        };

        for package in &self.packages {
            match Url::parse(&package) {
                Ok(url) => {
                    queue.push(startmc_downloader::Download::new(
                        &url,
                        dest.join(package.split('/').last().unwrap())
                            .to_str()
                            .unwrap(),
                        None,
                    ));
                }
                Err(_) => {
                    let path = Path::new(&package);
                    std::fs::copy(path, dest.join(path.file_name().unwrap())).unwrap();
                }
            }
        }

        let downloader = DownloaderBuilder::new().concurrent_downloads(10).build();
        downloader.download(&queue).await;

        match &self.kind {
            UpgradeKind::Mod => {
                println!(
                    "{cols} {installed} {amount} mods",
                    installed = Color::Default.bold().paint("Finished downloading"),
                    amount = Color::Green.paint(self.packages.len().to_string())
                );
            }
            UpgradeKind::Resourcepack => {
                println!(
                    "{cols} {installed} {amount} resourcepacks",
                    installed = Color::Default.bold().paint("Finished downloading"),
                    amount = Color::Green.paint(self.packages.len().to_string())
                );
            }
        }

        let mut sync =
            crate::sync::Sync::new(&config_path, Path::new(&config.minecraft.directory))?;
        println!(
            "{cols} {refreshing}",
            refreshing = Color::Default.bold().paint("Refreshing content index...")
        );
        sync.refresh().await?;

        Ok(())
    }
}
