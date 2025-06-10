use std::path::Path;

use ferinth::structures::search::{Facet, Sort};
use nu_ansi_term::Color;

use crate::cli::{CliSync, SyncOperand};

impl CliSync {
    pub async fn exec(
        self,
        instance: &str,
        config: crate::config::UnresolvedConfig,
        config_path: &Path,
    ) -> color_eyre::Result<()> {
        let cols = Color::Blue.bold().paint("::");
        let mut sync =
            crate::sync::Sync::new(&config_path, Path::new(&config.minecraft.directory))?;
        if self.refresh {
            println!(
                "{cols} {refreshing}",
                refreshing = Color::Default.bold().paint("Refreshing content index...")
            );
            sync.refresh().await?;
        }

        if self.upgrade {
            // TODO: upgrade content
            println!(
                "{cols} {upgrading}",
                upgrading = Color::Default.bold().paint("Upgrading content...")
            );
        }

        match self.operand {
            SyncOperand::Nothing => {}
            SyncOperand::Search { filter, sort } => {
                let loader_facet = if let Some(loader) =
                    self.loader.or_else(|| config.minecraft.get_loader_type())
                {
                    vec![Facet::Categories(loader)]
                } else {
                    vec![]
                };
                let results = sync
                    .fer
                    .search(
                        &filter.name,
                        sort.as_ref().unwrap_or(&Sort::Relevance),
                        vec![loader_facet],
                    )
                    .await?;
                for result in results.hits {
                    // REFERENCE:
                    // {CYAN}aur/{DEFAULT BOLD}mrpack-install {GREEN BOLD}0.16.10-1 [{DEFAULT BOLD}+0 {DEFAULT BOLD}~0.00]
                    // \t{DEFAULT}Modrinth Modpack server deployment

                    let slug = result.slug.expect("no slug");
                    println!(
                        "[{slug}] {title} {version}",
                        slug = Color::Cyan.paint(&slug),
                        title = Color::Default.bold().paint(result.title),
                        version = Color::Green.bold().paint(result.latest_version)
                    );
                }
            }
            SyncOperand::Install(packages) => {
                // TODO: install
                println!(
                    "{cols} {installing} {len} packages",
                    installing = Color::Default.bold().paint("Installing"),
                    len = packages.len()
                );
            }
        }

        sync.index.write(&config_path)?;
        Ok(())
    }
}
