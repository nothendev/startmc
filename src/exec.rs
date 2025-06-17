mod init;
mod remove;
mod run;
mod sync;
mod upgrade;

use crate::cli::{Cli, CliCommand};

impl Cli {
    pub async fn exec(self) -> color_eyre::Result<()> {
        match self.command {
            CliCommand::Remove(remove) => remove.exec(&self.instance).await,
            CliCommand::Run => run::exec(&self.instance).await,
            CliCommand::Sync(sync) => sync.exec(&self.instance).await,
            CliCommand::Upgrade(upgrade) => upgrade.exec(&self.instance).await,
            CliCommand::Init(init) => init.exec(&self.instance).await,
        }
    }
}
