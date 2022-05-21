use crate::{
    config::RepositoryMapping,
    operation::{CommandError, CommandResult, CommandResultDetails},
};
use anyhow::{anyhow, Result};
use crossbeam_channel::Sender;
use rayon::prelude::*;
use std::{
    fmt,
    path::{Path, PathBuf},
    process::Command,
    str,
};

struct GarbageCollectResult {
    path: PathBuf,
}

impl CommandResultDetails for GarbageCollectResult {}

impl fmt::Display for GarbageCollectResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OK: {}", self.path.display())
    }
}

fn gc(path: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(["-C", path.to_str().unwrap(), "gc"])
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(anyhow!("{}", str::from_utf8(&output.stderr)?))
    }
}

pub(crate) fn run(
    mappings: &[RepositoryMapping],
    s: Sender<CommandResult>,
) -> std::result::Result<(), usize> {
    let failure_count = mappings
        .par_iter()
        .map(|m| {
            log::info!("Processing: {}", m.path.display());

            let result = gc(&m.path);

            s.send(match result {
                Ok(()) => Ok(Box::new(GarbageCollectResult {
                    path: m.path.to_owned(),
                })),
                Err(ref e) => Err(Box::new(CommandError {
                    identifier: m.to_string(),
                    msg: e.to_string(),
                })),
            })
            .unwrap();

            result
        })
        .filter(|r| r.is_err())
        .count();

    if failure_count == 0 {
        Ok(())
    } else {
        Err(failure_count)
    }
}
