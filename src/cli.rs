use clap::*;

#[derive(Debug)]
pub enum Cli {
    Run(String, RunOptions),
    Sync(CliSync),
    Upgrade(CliUpgrade),
}

impl Cli {
    pub fn parse() -> Self {
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
                Command::new("run")
                    .short_flag('R')
                    .long_flag("run")
                    .about("Run a Minecraft instance")
                    .arg(
                        Arg::new("username")
                            .help("Username")
                            .action(ArgAction::Set)
                            .default_value("Steve"),
                    )
                    .arg(
                        Arg::new("uuid")
                            .help("UUID")
                            .action(ArgAction::Set)
                            .default_value("12345678-1234-1234-1234-123456789012"),
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
            .get_matches();

        let instance = clap.get_one::<String>("instance").unwrap().to_string();
        match clap.subcommand() {
            Some(("run", matches)) => Cli::Run(
                instance,
                RunOptions {
                    username: matches.get_one::<String>("username").unwrap().to_string(),
                },
            ),
            None => Cli::Run(
                instance,
                RunOptions {
                    username: "Steve".to_string(),
                },
            ),
            Some(("sync", matches)) => {
                let refresh = matches.get_flag("refresh");
                let upgrade = matches.get_flag("upgrade");
                let search = matches.get_many::<String>("search");
                let package = matches.get_many::<String>("package");

                Cli::Sync(CliSync {
                    instance,
                    operand: match (search, package) {
                        (None, Some(packages)) => {
                            SyncOperand::Install(packages.map(|p| p.to_string()).collect())
                        }
                        (Some(search), None) => {
                            SyncOperand::Search(search.map(|s| s.to_string()).collect())
                        }
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
                Cli::Upgrade(CliUpgrade {
                    instance,
                    packages,
                    kind,
                })
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct RunOptions {
    pub username: String,
}

#[derive(Debug)]
pub struct CliSync {
    pub instance: String,
    pub refresh: bool,
    pub upgrade: bool,
    pub operand: SyncOperand,
}

#[derive(Debug)]
pub enum SyncOperand {
    Search(Vec<String>),
    Install(Vec<String>),
    Nothing,
}

#[derive(Debug)]
pub struct CliUpgrade {
    pub instance: String,
    pub packages: Vec<String>,
    pub kind: UpgradeKind,
}

#[derive(Debug, Default)]
pub enum UpgradeKind {
    #[default]
    Mod,
    Resourcepack,
}
