use std::ffi::OsString;

use crate::sync::SyncFilter;
use clap::*;
use ferinth::structures::search::Sort;

/// The main CLI struct.
///
/// You can invoke the CLI programmatically by either
#[derive(Debug)]
pub struct Cli {
    pub command: CliCommand,
    pub instance: String,
}

#[derive(Debug)]
pub enum CliCommand {
    Run,
    Init(CliInit),
    Sync(CliSync),
    Upgrade(CliUpgrade),
    Remove(CliRemove),
}

#[derive(Debug)]
pub struct CliInit {
    pub version: Option<String>,
    pub fabric: Option<String>,
}

#[derive(Debug)]
pub struct CliSync {
    pub refresh: bool,
    pub upgrade: bool,
    pub operand: SyncOperand,
    pub loader: Option<String>,
}

#[derive(Debug)]
pub enum SyncOperand {
    Search {
        filter: SyncFilter,
        sort: Option<Sort>,
    },
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
    /// Build the clap command.
    pub fn command() -> clap::Command {
        Command::new("startmc")
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .about("Start Minecraft and manage content on any number of instances")
            // this is an arg on the global command, so that:
            // - you can leave it empty and it will default to "default"
            // - you can specify a different instance name like `startmc INSTANCENAME` without any ugly arg flags
            // - you can specify it and then use a command like `startmc INSTANCENAME -I` for example, which will init an instance with name `INSTANCENAME`
            // - you can leave it empty and then use commands like `startmc -Sy` will sync the default instance's index
            .arg(
                Arg::new("instance")
                    .help("Instance name")
                    .default_value("default")
                    .action(ArgAction::Set),
            )
            .subcommand(
                Command::new("init")
                    .short_flag('I')
                    .long_flag("init")
                    .about("Initialize a new instance")
                    .arg(
                        Arg::new("version")
                            .short('m')
                            .long("version")
                            .help("Minecraft version")
                            .action(ArgAction::Set),
                    )
                    .arg(
                        Arg::new("fabric")
                            .short('f')
                            .long("fabric")
                            .help("Fabric version, optional")
                            .action(ArgAction::Set),
                    )
                    .arg(
                        Arg::new("java")
                            .short('j')
                            .long("java")
                            .help("Java path")
                            .action(ArgAction::Set),
                    )
                    .arg(
                        Arg::new("libraries")
                            .short('l')
                            .long("libraries")
                            .help("Libraries path")
                            .action(ArgAction::Append),
                    )
                    .arg(
                        Arg::new("directory")
                            .short('d')
                            .long("directory")
                            .help("Instance directory")
                            .action(ArgAction::Set),
                    ),
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
                            .help("search remote repository for matching strings"),
                    )
                    .arg(
                        Arg::new("sort")
                            .short('o')
                            .long("sort")
                            .action(ArgAction::Set)
                            .help("sort the search results"),
                    )
                    .arg(
                        Arg::new("package")
                            .help("packages")
                            .action(ArgAction::Set)
                            .num_args(1..)
                            .conflicts_with("search")
                            .action(ArgAction::Set),
                    )
                    .arg(
                        Arg::new("loader")
                            .short('l')
                            .long("loader")
                            .action(ArgAction::Set)
                            .help("set the needed mod loader"),
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
    }

    /// Parse a [`Cli`] from the command line arguments, i.e. [`std::env::args_os()`].
    pub fn parse() -> color_eyre::Result<Self> {
        Self::parse_from_matches(Self::command().try_get_matches()?)
    }

    /// Parse a [`Cli`] from a list of arguments. These can be anything that can be [`Into`]'d into an [`OsString`].
    pub fn parse_from<S: Into<OsString> + Clone>(
        args: impl IntoIterator<Item = S>,
    ) -> color_eyre::Result<Self> {
        Self::parse_from_matches(Self::command().try_get_matches_from(args)?)
    }

    /// Parse a [`Cli`] from clap [`ArgMatches`].
    pub fn parse_from_matches(clap: ArgMatches) -> color_eyre::Result<Self> {
        let instance = clap.get_one::<String>("instance").unwrap().to_string();

        Ok(Cli {
            instance,
            command: match clap.subcommand() {
                None => CliCommand::Run,
                Some(("init", matches)) => {
                    let version = matches.get_one::<String>("version").map(|s| s.to_string());
                    let fabric = matches.get_one::<String>("fabric").map(|s| s.to_string());
                    CliCommand::Init(CliInit { version, fabric })
                }
                Some(("sync", matches)) => {
                    let refresh = matches.get_flag("refresh");
                    let upgrade = matches.get_flag("upgrade");
                    let search = matches.get_one::<String>("search");
                    let package = matches.get_many::<String>("package");
                    let loader = matches.get_one::<String>("loader").map(|s| s.to_string());
                    let sort = matches.get_one::<String>("sort").map(|s| {
                        let s = s.to_lowercase();
                        match s.as_str() {
                            "relevance" => Sort::Relevance,
                            "downloads" => Sort::Downloads,
                            "follows" => Sort::Follows,
                            "newest" => Sort::Newest,
                            "updated" => Sort::Updated,
                            _ => panic!("invalid sort: {s}"),
                        }
                    });

                    CliCommand::Sync(CliSync {
                        operand: match (search, package) {
                            (None, Some(packages)) => SyncOperand::Install(
                                packages
                                    .map(|p| p.parse().expect("invalid package"))
                                    .collect(),
                            ),
                            (Some(search), None) => SyncOperand::Search {
                                filter: search.parse().expect("invalid package"),
                                sort,
                            },
                            _ => SyncOperand::Nothing,
                        },
                        loader,
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
