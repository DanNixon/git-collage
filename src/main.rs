mod config;
mod filter;
mod matching_rules;
mod operation;
mod source;
mod util;

use crate::config::{Config, RepositoryMappingProducer};
use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::CompleteEnv;
use log::trace;
use std::path::PathBuf;

/// Tool for selectively mirroring Git repositories
/// (https://github.com/DanNixon/git-collage)
#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Path to configuration file(s) or directory containing .toml files.
    /// Can be specified multiple times to include multiple files or directories.
    #[clap(short, long, value_name = "FILE|DIR")]
    config: Vec<PathBuf>,

    #[clap(subcommand)]
    command: operation::Command,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    CompleteEnv::with_factory(Cli::command).complete();
    let cli = Cli::parse();

    let config = Config::load(&cli.config)?;
    trace!("Config = {:#?}", config);

    let mappings = config.repository_mappings().await;
    trace!("Repository mappings = {:#?}", mappings);

    cli.command.run(mappings)
}
