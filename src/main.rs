#[macro_use]
extern crate tracing;

use std::path::Path;

use color_eyre::eyre::Context;
use nu_ansi_term::Color;
use reqwest::Url;
use startmc_downloader::DownloaderBuilder;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

mod cache;
mod cli;
mod config;
mod sync;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let cli = cli::Cli::parse();
    debug!("{:?}", cli);

    let cols = Color::Blue.bold().paint("::");
    match cli {
        cli::Cli::Run(instance) => {
            let unresolved = config::UnresolvedConfig::find(&instance).context("config not found")?;
            println!(
                "{cols} {running} {instance}",
                running = Color::Default.bold().paint("Running instance"),
                instance = Color::Green.paint(&instance)
            );

            let config = unresolved.resolve().await.context("resolve config")?;
            let star = Color::Yellow.bold().paint("*");
            println!(
                "{star} Using Java path: {javapath}",
                javapath = Color::Cyan.paint(&config.java_path)
            );
            println!(
                "{star} Using libraries path: {librariespath}",
                librariespath = Color::Cyan.paint(&config.libraries_path)
            );
            println!(
                "{star} Using Minecraft directory: {minecraftdir}",
                minecraftdir = Color::Cyan.paint(&config.minecraft_dir)
            );

            let mut queue: Vec<startmc_downloader::Download> = vec![];

            config.download_client(&mut queue);
            config.download_libraries(&mut queue).await?;
            config.download_assets(&mut queue).await?;
            if queue.len() > 0 {
                println!(
                    "{cols} {downloading}",
                    downloading = Color::Default.bold().paint("Downloading assets...")
                );

                let downloader = DownloaderBuilder::new().concurrent_downloads(10).build();
                downloader.download(&queue).await;
            }

            println!(
                "{cols} {starting} {version}",
                starting = Color::Default.bold().paint("Starting Minecraft"),
                version = Color::Green.paint(&config.version.id)
            );

            let status = config.start().await?;
            let code = status.code().unwrap_or(i32::MIN);

            println!(
                "{cols} {exited} {status}",
                exited = Color::Default.bold().paint("Minecraft finished"),
                status = if code == 0 {
                    Color::Green.paint("successfully").to_string()
                } else {
                    format!(
                        "{} {}",
                        Color::Red.paint("with exit code").to_string(),
                        code
                    )
                }
            );
        }

        cli::Cli::Sync(sync) => {
            let (path, unresolved) =
                config::UnresolvedConfig::find_with_path(&sync.instance).context("config not found")?;
            let client = sync::Sync::new(&path)?;

            if sync.refresh {
                println!("{cols} {refreshing}", refreshing = Color::Default.bold().paint("Refreshing content index..."));
                client.refresh().await?;
            }
            if sync.upgrade {
                // TODO: upgrade content
                println!("{cols} {upgrading}", upgrading = Color::Default.bold().paint("Upgrading content..."));
            }

            client.index.write(&path)?;
        }

        cli::Cli::Upgrade(upgrade) => {
            let config =
                config::UnresolvedConfig::find(&upgrade.instance).context("config not found")?;
            let mut queue: Vec<startmc_downloader::Download> = vec![];
            let dest = match &upgrade.kind {
                cli::UpgradeKind::Mod => {
                    println!(
                        "{cols} {downloading} {amount} mods",
                        downloading = Color::Default.bold().paint("Downloading"),
                        amount = Color::Green.paint(upgrade.packages.len().to_string())
                    );

                    Path::new(&config.minecraft.directory).join("mods")
                }
                cli::UpgradeKind::Resourcepack => {
                    println!(
                        "{cols} {downloading} {amount} resourcepacks",
                        downloading = Color::Default.bold().paint("Downloading"),
                        amount = Color::Green.paint(upgrade.packages.len().to_string())
                    );

                    Path::new(&config.minecraft.directory).join("resourcepacks")
                }
            };

            for package in &upgrade.packages {
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

            match &upgrade.kind {
                cli::UpgradeKind::Mod => {
                    println!(
                        "{cols} {installed} {amount} mods",
                        installed = Color::Default.bold().paint("Finished downloading"),
                        amount = Color::Green.paint(upgrade.packages.len().to_string())
                    );
                }
                cli::UpgradeKind::Resourcepack => {
                    println!(
                        "{cols} {installed} {amount} resourcepacks",
                        installed = Color::Default.bold().paint("Finished downloading"),
                        amount = Color::Green.paint(upgrade.packages.len().to_string())
                    );
                }
            }
        }

        cli::Cli::Remove(remove) => {
            let config = config::UnresolvedConfig::find(&remove.instance).context("config not found")?;
        }
    }

    Ok(())
}
