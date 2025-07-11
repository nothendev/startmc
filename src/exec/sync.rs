use std::{path::Path, time::Duration};

use chrono::Utc;
use chrono_humanize::{Accuracy, Tense};
use color_eyre::eyre::Context;
use ferinth::structures::search::{Facet, Sort};
use indicatif::ProgressBar;
use owo_colors::OwoColorize;

use crate::{
    cli::{CliSync, SyncOperand},
    util::cols,
};

impl CliSync {
    pub async fn exec(self, instance: &str) -> color_eyre::Result<()> {
        let (config_path, config) =
            crate::config::UnresolvedConfig::find_with_path(instance).context("find config")?;
        let cols = cols();
        let mut sync =
            crate::sync::Sync::new(&config_path, Path::new(&config.minecraft.directory))?;
        if self.refresh {
            println!(
                "{cols} {refreshing}",
                refreshing = "Refreshing content index...".bold()
            );
            sync.refresh().await?;
        }

        if self.upgrade {
            // TODO: upgrade content
            println!(
                "{cols} {upgrading}",
                upgrading = "Starting full content upgrade...".bold()
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
                let spinner = ProgressBar::new_spinner();
                spinner.enable_steady_tick(Duration::from_millis(100));
                spinner.set_message("Querying Modrinth API...");
                let results = sync
                    .fer
                    .search(
                        &filter.name,
                        sort.as_ref().unwrap_or(&Sort::Relevance),
                        vec![loader_facet],
                    )
                    .await?;
                spinner.finish_and_clear();

                for result in results.hits {
                    // REFERENCE:
                    // {CYAN}aur/{DEFAULT BOLD}mrpack-install {GREEN BOLD}0.16.10-1 [{DEFAULT BOLD}+0 {DEFAULT BOLD}~0.00]
                    // \t{DEFAULT}Modrinth Modpack server deployment

                    let modified_since = Utc::now() - result.date_modified;
                    let ht = chrono_humanize::HumanTime::from(modified_since);
                    let upd = ht.to_text_en(Accuracy::Rough, Tense::Past);
                    let slug = result.slug.as_deref().unwrap_or(&result.project_id);
                    let downloads = re_format::approximate_large_number(result.downloads as f64);
                    let follows = re_format::approximate_large_number(result.follows as f64);
                    println!(
                        "{slug} <{upd}> [{dwl_icon} {downloads} {bar} {follow_icon} {follows}]\n    {title} {desc}",
                        slug = slug.bold(),
                        title = result.title.yellow().bold(),
                        upd = upd.green().italic(),
                        dwl_icon = "".green().bold(),
                        downloads = downloads.bold(),
                        bar = "|".bright_white().dimmed().bold(),
                        follow_icon = "".bright_magenta().bold(),
                        follows = follows.bold(),
                        desc = result.description,
                    );
                }
            }

            SyncOperand::Install(packages) => {
                // TODO: install
                println!(
                    "{cols} {installing} {len} packages",
                    installing = "Installing".bold(),
                    len = packages.len()
                );
            }
        }

        sync.index.write(&config_path)?;
        Ok(())
    }
}
