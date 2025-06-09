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
}
