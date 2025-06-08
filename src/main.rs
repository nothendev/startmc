#[macro_use]
extern crate tracing;

use nu_ansi_term::Color;
use startmc_downloader::DownloaderBuilder;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

mod cache;
mod cli;
mod config;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let cli = cli::Cli::parse();
    debug!("{:?}", cli);
    let rq = reqwest::Client::new();

    let cols = Color::Blue.bold().paint("::");
    match cli {
        cli::Cli::Run(instance, opts) => {
            let unresolved = config::UnresolvedConfig::find(&instance).expect("config not found");
            println!(
                "{cols} {running} {instance}",
                running = Color::Default.bold().paint("Running instance"),
                instance = Color::Green.paint(&instance)
            );

            let config = unresolved.resolve(&rq).await?;
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
            config.download_libraries(&mut queue, &rq).await?;
            config.download_assets(&mut queue, &rq).await?;
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

            let status = config.start(&rq).await?;
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
            println!("Sync: {sync:#?}");
        }

        cli::Cli::Upgrade(upgrade) => {
            let config = {}; // TODO
            match upgrade.kind {
                cli::UpgradeKind::Mod => {
                    println!(
                        "{cols} {downloading} {amount} mods",
                        downloading = Color::Default.bold().paint("Downloading"),
                        amount = Color::Green.paint(upgrade.packages.len().to_string())
                    );

                    let mut queue: Vec<startmc_downloader::Download> = vec![];
                }
                cli::UpgradeKind::Resourcepack => println!("Upgrade: {upgrade:#?}"),
            }
        }
    }

    Ok(())
}
