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

    let cli = match startmc::cli::Cli::parse() {
        Ok(cli) => cli,
        Err(err) => err.exit()
    };
    tracing::debug!("PARSED CLI: {:?}", cli);
    cli.exec().await
}
