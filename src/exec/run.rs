use color_eyre::eyre::Context;
use nu_ansi_term::Color;
use startmc_downloader::DownloaderBuilder;

pub async fn exec(instance: &str) -> color_eyre::Result<()> {
    let config = crate::config::UnresolvedConfig::find(instance).context("find config")?;
    let config = config.resolve().await?;
    let cols = Color::Blue.bold().paint("::");
    let star = Color::Yellow.bold().paint("*");

    println!(
        "{cols} {running} {instance}",
        running = Color::Default.bold().paint("Running instance"),
        instance = Color::Green.paint(instance)
    );

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

    Ok(())
}
