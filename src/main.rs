mod config;
mod filter;
mod matching_rules;
mod operation;
mod source;
mod util;

use crate::config::{Config, RepositoryMappingProducer};
use anyhow::Result;
use clap::{CommandFactory, Parser};
use std::path::PathBuf;

/// Tool for selectively mirroring Git repositories
/// (https://github.com/DanNixon/git-collage)
#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Path to configuration file(s)
    #[clap(short, long, value_name = "FILE", default_value = "./config.toml")]
    config: Vec<PathBuf>,

    #[clap(subcommand)]
    command: operation::Command,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    if let operation::Command::Completions { shell } = cli.command {
        let mut cmd = Cli::command();
        let name = cmd.get_name().to_string();
        clap_complete::generate(shell, &mut cmd, name, &mut std::io::stdout());
        return Ok(());
    }

    let config = Config::load(&cli.config)?;
    log::trace!("Config = {:#?}", config);

    let mappings = config.repository_mappings().await;
    log::trace!("Repository mappings = {:#?}", mappings);

    cli.command.run(mappings)
}
