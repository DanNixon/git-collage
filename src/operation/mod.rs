mod garbage_collect;
mod list;
mod mirror;
mod stale;

use crate::config::RepositoryMapping;
use anyhow::{anyhow, Result};
use clap::Subcommand;
use clap_complete::Shell;
use crossbeam_channel::{select, unbounded};
use std::{fmt, thread};

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// List repositories that will be mirrored and their local destinations
    #[clap(name = "ls")]
    ListRepositories,

    /// Update local mirrors
    Mirror,

    /// Run `git gc` on local mirrors
    #[clap(name = "gc")]
    GarbageCollect,

    /// Identify stale/unmanaged local mirrors
    #[clap(name = "stale")]
    IdentifyStale(stale::Cli),

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
}

pub(crate) type CommandResult =
    std::result::Result<Box<dyn CommandResultDetails>, Box<dyn CommandResultDetails>>;

pub(crate) trait CommandResultDetails: fmt::Display + Send {}

struct CommandError {
    identifier: String,
    msg: String,
}

impl CommandResultDetails for CommandError {}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n{}", self.identifier, self.msg)
    }
}

impl Command {
    pub(crate) fn run(&self, mappings: Vec<Result<RepositoryMapping>>) -> Result<()> {
        let (s, r) = unbounded::<CommandResult>();

        thread::spawn(move || loop {
            select! {
                recv(r) -> s => {
                    match s {
                        Ok(s) => {
                            match s {
                                Ok(s) => log::info!("{}", s),
                                Err(e) => log::error!("{}", e),
                            }
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }
            }
        });

        if mappings.iter().any(|m| m.is_err()) {
            for e in mappings.into_iter().flat_map(|m| m.err()) {
                log::error!("{}", e);
            }
            Err(anyhow!("Configuration problems found"))
        } else {
            let mappings: Vec<RepositoryMapping> =
                mappings.into_iter().flat_map(|m| m.ok()).collect();

            let result = match &self {
                Command::ListRepositories => list::run(&mappings),
                Command::Mirror => mirror::run(&mappings, s),
                Command::GarbageCollect => garbage_collect::run(&mappings, s),
                Command::IdentifyStale(args) => stale::run(&mappings, args),
                Command::Completions { .. } => {
                    panic!("Should not reach here when the completions subcommand was used")
                }
            };

            match result {
                Ok(()) => Ok(()),
                Err(c) => Err(anyhow!("{} jobs failed", c)),
            }
        }
    }
}
