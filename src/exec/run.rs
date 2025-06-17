use color_eyre::eyre::Context;
use owo_colors::OwoColorize;
use startmc_downloader::DownloaderBuilder;

use crate::util::{cols, green_arrow};

pub async fn exec(instance: &str) -> color_eyre::Result<()> {
    let config = crate::config::UnresolvedConfig::find(instance).context("find config")?;
    let config = config.resolve().await?;
    let cols = cols();
    let arrow = green_arrow();

    println!(
        "{cols} {running} {instance}",
        running = "Running instance".bold(),
    );

    println!(
        "{arrow} Using Java path: {javapath}",
        javapath = config.java_path
    );
    println!(
        "{arrow} Using libraries path: {librariespath}",
        librariespath = config.libraries_path
    );
    println!(
        "{arrow} Using Minecraft directory: {minecraftdir}",
        minecraftdir = config.minecraft_dir
    );

    let mut queue: Vec<startmc_downloader::Download> = vec![];

    config.download_client(&mut queue);
    config.download_libraries(&mut queue).await?;
    config.download_assets(&mut queue).await?;
    if queue.len() > 0 {
        println!(
            "{cols} {downloading}",
            downloading = "Downloading assets...".bold()
        );

        let downloader = DownloaderBuilder::new().concurrent_downloads(10).build();
        downloader.download(&queue).await;
    }

    println!(
        "{cols} {starting} {version}",
        starting = "Starting Minecraft".bold(),
        version = config.version.id.green()
    );

    let status = config.start().await?;
    let code = status.code().unwrap_or(i32::MIN);

    println!(
        "{cols} {exited} {status}",
        exited = "Minecraft finished".bold(),
        status = if code == 0 {
            "successfully".green().to_string()
        } else {
            format!(
                "{} {}",
                "with exit code".red().to_string(),
                code
            )
        }
    );

    Ok(())
}
