use clap::*;

#[derive(Debug)]
pub enum Cli {
    Run(String, RunOptions),
    Sync(CliSync),
}

impl Cli {
    pub fn parse() -> Self {
        let clap = Command::new("startmc")
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .about("Start Minecraft")
            .arg_required_else_help(true)
            .subcommand_required(true)
            .subcommand(
                Command::new("run")
                    .short_flag('R')
                    .long_flag("run")
                    .about("Run a Minecraft instance")
                    .arg(
                        Arg::new("instance")
                            .help("Instance name")
                            .default_value("default")
                            .action(ArgAction::Set),
                    )
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
            .get_matches();

        match clap.subcommand() {
            Some(("run", matches)) => {
                let instance = matches.get_one::<String>("instance").unwrap();
                Cli::Run(
                    instance.to_string(),
                    RunOptions {
                        username: matches.get_one::<String>("username").unwrap().to_string(),
                    },
                )
            }
            Some(("sync", matches)) => {
                let refresh = matches.get_flag("refresh");
                let upgrade = matches.get_flag("upgrade");
                let search = matches.get_many::<String>("search");
                let package = matches.get_many::<String>("package");

                Cli::Sync(CliSync {
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
