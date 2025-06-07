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
            config.download_libraries(&mut queue);
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

            println!("{config:#?}");

            let status = config.start()?;
            println!("{status:#?}");
        }

        cli::Cli::Sync(sync) => {
            println!("Sync: {sync:#?}");
        }
    }

    Ok(())
}
