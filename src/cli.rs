use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Get information about a specific version
    Version(VersionCommand),
    /// Get all versions
    Versions
}

#[derive(Args, Debug)]
pub struct VersionCommand {
    /// The version to get information about
    pub version: String,
}
