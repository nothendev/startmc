#[macro_use]
extern crate tracing;

use clap::Parser;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

mod cli;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    info!("startmc v{}", env!("CARGO_PKG_VERSION"));

    let cli = cli::Cli::parse();
    debug!("{:?}", cli);
    let rq = reqwest::Client::new();

    match cli.command {
        cli::Command::Version(version_command) => {
            debug!("get version {:?}", version_command);
            let versions = startmc_mojapi::model::VersionManifestV2::fetch(&rq).await.unwrap();
            let version = versions.versions.into_iter().find(|v| v.id == version_command.version).unwrap();
            let version = version.fetch(&rq).await.unwrap();
            println!("{:#?}", version);
        }
        cli::Command::Versions => {
            debug!("Versions");
        }
    }
}
