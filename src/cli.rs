use crate::sync::{SyncFilter, VersionTuple};
use clap::*;

#[derive(Debug)]
pub struct Cli {
    pub command: CliCommand,
    pub instance: String,
}

#[derive(Debug)]
pub enum CliCommand {
    Run,
    Sync(CliSync),
    Upgrade(CliUpgrade),
    Remove(CliRemove),
}

#[derive(Debug)]
pub struct CliSync {
    pub refresh: bool,
    pub upgrade: bool,
    pub operand: SyncOperand,
}

#[derive(Debug)]
pub enum SyncOperand {
    Search(Vec<SyncFilter>),
    Install(Vec<SyncFilter>),
    Nothing,
}

#[derive(Debug)]
pub struct CliUpgrade {
    pub packages: Vec<String>,
    pub kind: UpgradeKind,
}

#[derive(Debug, Default)]
pub enum UpgradeKind {
    #[default]
    Mod,
    Resourcepack,
}

#[derive(Debug)]
pub struct CliRemove {
    pub disable: bool,
    pub packages: Vec<SyncFilter>,
}

impl Cli {
    pub fn parse() -> color_eyre::Result<Self> {
        let clap = Command::new("startmc")
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .about("Start Minecraft")
            .arg(
                Arg::new("instance")
                    .help("Instance name")
                    .default_value("default")
                    .action(ArgAction::Set),
            )
            .subcommand(
                Command::new("sync")
                    .short_flag('S')
                    .long_flag("sync")
                    .about("Download content onto a Minecraft instance")
                    .arg(
                        Arg::new("refresh")
                            .short('y')
                            .long("refresh")
                            .help("Refresh the content index")
                            .action(ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("upgrade")
                            .short('u')
                            .long("upgrade")
                            .help("Upgrade all content")
                            .action(ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("search")
                            .short('s')
                            .long("search")
                            .action(ArgAction::Set)
                            .num_args(1..)
                            .help("search remote repository for matching strings"),
                    )
                    .arg(
                        Arg::new("package")
                            .help("packages")
                            .action(ArgAction::Set)
                            .num_args(1..)
                            .conflicts_with("search")
                            .action(ArgAction::Set),
                    ),
            )
            .subcommand(
                Command::new("upgrade")
                    .short_flag('U')
                    .long_flag("upgrade")
                    .about("Download content onto a Minecraft instance")
                    .arg(
                        Arg::new("packages")
                            .help("packages")
                            .action(ArgAction::Append)
                            .num_args(1..)
                            .required(true),
                    )
                    .arg(
                        Arg::new("resourcepack")
                            .short('r')
                            .long("resourcepack")
                            .action(ArgAction::SetTrue)
                            .help("Packages are resourcepacks, not mods"),
                    ),
            )
            .subcommand(
                Command::new("remove")
                    .short_flag('R')
                    .long_flag("remove")
                    .about("Remove some content from a Minecraft instance")
                    .arg(
                        Arg::new("disable")
                            .short('d')
                            .long("disable")
                            .action(ArgAction::SetTrue)
                            .help("Disable, not remove"),
                    )
                    .arg(
                        Arg::new("packages")
                            .help("packages")
                            .action(ArgAction::Append)
                            .num_args(1..)
                            .required(true),
                    ),
            )
            .try_get_matches()?;

        let instance = clap.get_one::<String>("instance").unwrap().to_string();
        Ok(Cli {
            instance,
            command: match clap.subcommand() {
                None => CliCommand::Run,
                Some(("sync", matches)) => {
                    let refresh = matches.get_flag("refresh");
                    let upgrade = matches.get_flag("upgrade");
                    let search = matches.get_many::<String>("search");
                    let package = matches.get_many::<String>("package");

                    CliCommand::Sync(CliSync {
                        operand: match (search, package) {
                            (None, Some(packages)) => SyncOperand::Install(
                                packages
                                    .map(|p| p.parse().expect("invalid package"))
                                    .collect(),
                            ),
                            (Some(search), None) => SyncOperand::Search(
                                search
                                    .map(|s| s.parse().expect("invalid package"))
                                    .collect(),
                            ),
                            _ => SyncOperand::Nothing,
                        },
                        refresh,
                        upgrade,
                    })
                }
                Some(("upgrade", matches)) => {
                    let packages = matches
                        .get_many::<String>("packages")
                        .unwrap()
                        .map(|p| p.to_string())
                        .collect();
                    let kind = if matches.get_flag("resourcepack") {
                        UpgradeKind::Resourcepack
                    } else {
                        UpgradeKind::Mod
                    };
                    CliCommand::Upgrade(CliUpgrade { packages, kind })
                }
                Some(("remove", matches)) => {
                    let disable = matches.get_flag("disable");
                    let packages = matches
                        .get_many::<String>("packages")
                        .unwrap()
                        .map(|p| p.parse().expect("invalid package"))
                        .collect();
                    CliCommand::Remove(CliRemove { disable, packages })
                }
                _ => unreachable!(),
            },
        })
    }
}
