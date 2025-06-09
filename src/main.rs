#[macro_use]
extern crate tracing;

use color_eyre::eyre::Context;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

mod cache;
mod cli;
mod config;
mod exec;
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

    let cli = cli::Cli::parse().context("parse cli")?;
    debug!("PARSED CLI: {:?}", cli);
    cli.exec().await

    // let cols = Color::Blue.bold().paint("::");
    // let (config_path, config) = config::UnresolvedConfig::find_with_path(&cli.instance).context("config not found")?;
    // let sync = sync::Sync::new(&config.path, Path::new(&config.minecraft.directory))?;
    // match cli.command {
    //     cli::CliCommand::Sync(cli) => {
    //         if cli.refresh {
    //             println!(
    //                 "{cols} {refreshing}",
    //                 refreshing = Color::Default.bold().paint("Refreshing content index...")
    //             );
    //             sync.refresh().await?;
    //         }
    //         if cli.upgrade {
    //             // TODO: upgrade content
    //             println!(
    //                 "{cols} {upgrading}",
    //                 upgrading = Color::Default.bold().paint("Upgrading content...")
    //             );
    //         }

    //         sync.index.write(&path)?;
    //     }

    //     cli::CliCommand::Upgrade(upgrade) => {
    //         let mut queue: Vec<startmc_downloader::Download> = vec![];
    //         let dest = match &upgrade.kind {
    //             cli::UpgradeKind::Mod => {
    //                 println!(
    //                     "{cols} {downloading} {amount} mods",
    //                     downloading = Color::Default.bold().paint("Downloading"),
    //                     amount = Color::Green.paint(upgrade.packages.len().to_string())
    //                 );

    //                 Path::new(&config.minecraft.directory).join("mods")
    //             }
    //             cli::UpgradeKind::Resourcepack => {
    //                 println!(
    //                     "{cols} {downloading} {amount} resourcepacks",
    //                     downloading = Color::Default.bold().paint("Downloading"),
    //                     amount = Color::Green.paint(upgrade.packages.len().to_string())
    //                 );

    //                 Path::new(&config.minecraft.directory).join("resourcepacks")
    //             }
    //         };

    //         for package in &upgrade.packages {
    //             match Url::parse(&package) {
    //                 Ok(url) => {
    //                     queue.push(startmc_downloader::Download::new(
    //                         &url,
    //                         dest.join(package.split('/').last().unwrap())
    //                             .to_str()
    //                             .unwrap(),
    //                         None,
    //                     ));
    //                 }
    //                 Err(_) => {
    //                     let path = Path::new(&package);
    //                     std::fs::copy(path, dest.join(path.file_name().unwrap())).unwrap();
    //                 }
    //             }
    //         }

    //         let downloader = DownloaderBuilder::new().concurrent_downloads(10).build();
    //         downloader.download(&queue).await;

    //         match &upgrade.kind {
    //             cli::UpgradeKind::Mod => {
    //                 println!(
    //                     "{cols} {installed} {amount} mods",
    //                     installed = Color::Default.bold().paint("Finished downloading"),
    //                     amount = Color::Green.paint(upgrade.packages.len().to_string())
    //                 );
    //             }
    //             cli::UpgradeKind::Resourcepack => {
    //                 println!(
    //                     "{cols} {installed} {amount} resourcepacks",
    //                     installed = Color::Default.bold().paint("Finished downloading"),
    //                     amount = Color::Green.paint(upgrade.packages.len().to_string())
    //                 );
    //             }
    //         }
    //     }

    //     cli::CliCommand::Remove(remove) => {
    //         sync.maybe_refresh().await?;
    //     }
    // }

    // Ok(())
}
