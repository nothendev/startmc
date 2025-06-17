use std::path::Path;

use color_eyre::eyre::Context;
use owo_colors::OwoColorize;
use reqwest::Url;
use startmc_downloader::DownloaderBuilder;

use crate::{
    cli::{CliUpgrade, UpgradeKind},
    util::cols,
};

impl CliUpgrade {
    pub async fn exec(self, instance: &str) -> color_eyre::Result<()> {
        let (config_path, config) =
            crate::config::UnresolvedConfig::find_with_path(instance).context("find config")?;
        let cols = cols();
        let mut queue: Vec<startmc_downloader::Download> = vec![];
        let dest = match &self.kind {
            UpgradeKind::Mod => {
                println!(
                    "{cols} {downloading} {amount} mods",
                    downloading = "Downloading".bold(),
                    amount = self.packages.len().green()
                );

                Path::new(&config.minecraft.directory).join("mods")
            }
            UpgradeKind::Resourcepack => {
                println!(
                    "{cols} {downloading} {amount} resourcepacks",
                    downloading = "Downloading".bold(),
                    amount = self.packages.len().green()
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
                    installed = "Finished downloading".bold(),
                    amount = self.packages.len().green()
                );
            }
            UpgradeKind::Resourcepack => {
                println!(
                    "{cols} {installed} {amount} resourcepacks",
                    installed = "Finished downloading".bold(),
                    amount = self.packages.len().green()
                );
            }
        }

        let mut sync =
            crate::sync::Sync::new(&config_path, Path::new(&config.minecraft.directory))?;
        println!(
            "{cols} {refreshing}",
            refreshing = "Refreshing content index...".bold()
        );
        sync.refresh().await?;

        Ok(())
    }
}
