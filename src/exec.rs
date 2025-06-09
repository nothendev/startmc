mod remove;
mod run;
mod sync;
mod upgrade;

use color_eyre::eyre::Context;

use crate::{
    cli::{Cli, CliCommand},
    config::UnresolvedConfig,
};

impl Cli {
    pub async fn exec(self) -> color_eyre::Result<()> {
        let (path, config) =
            UnresolvedConfig::find_with_path(&self.instance).context("config not found")?;

        match self.command {
            CliCommand::Remove(remove) => remove.exec(&self.instance, config, &path).await,
            CliCommand::Run => run::exec(&self.instance, config.resolve().await?).await,
            CliCommand::Sync(sync) => sync.exec(&self.instance, config, &path).await,
            CliCommand::Upgrade(upgrade) => upgrade.exec(&self.instance, config, &path).await,
        }
    }
}
