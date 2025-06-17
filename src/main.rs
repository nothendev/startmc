use color_eyre::eyre::Context;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

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

    let cli = startmc::cli::Cli::parse().context("parse cli")?;
    tracing::debug!("PARSED CLI: {:?}", cli);
    cli.exec().await
}
