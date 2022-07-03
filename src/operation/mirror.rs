use crate::{
    config::RepositoryMapping,
    matching_rules::Match,
    operation::{CommandError, CommandResult, CommandResultDetails},
    util::git_timestamp,
};
use anyhow::{anyhow, Result};
use chrono::{DateTime, FixedOffset};
use crossbeam_channel::Sender;
use git2::{
    Direction, Oid, RemoteHead, Repository, RepositoryInitMode, RepositoryInitOptions, Time,
};
use rayon::prelude::*;
use std::{fmt, process::Command, str};

struct MirrorResult {
    mapping: RepositoryMapping,
    refs: Vec<RefStatusReport>,
}

impl fmt::Display for MirrorResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OK: {}", self.mapping)?;
        for r in &self.refs {
            write!(f, "\n{}", r)?;
        }
        Ok(())
    }
}

impl CommandResultDetails for MirrorResult {}

struct RefStatusReport {
    name: String,

    previous_oid: Option<Oid>,
    previous_timestamp: Option<DateTime<FixedOffset>>,

    current_oid: Oid,
    current_timestamp: DateTime<FixedOffset>,
}

impl RefStatusReport {
    fn new(
        name: &str,
        previous_oid: Option<Oid>,
        previous_timestamp: Option<Time>,
        current_oid: Oid,
        current_timestamp: Time,
    ) -> Self {
        let previous_timestamp = previous_timestamp.map(git_timestamp);
        let current_timestamp = git_timestamp(current_timestamp);

        Self {
            name: name.to_string(),
            previous_oid,
            previous_timestamp,
            current_oid,
            current_timestamp,
        }
    }
}

impl fmt::Display for RefStatusReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.previous_oid.is_none() || self.previous_timestamp.is_none() {
            write!(
                f,
                "[new] {} {:?} : {}",
                self.current_oid, self.current_timestamp, self.name
            )
        } else if self.previous_oid.unwrap() == self.current_oid {
            write!(
                f,
                "[nop] {} {:?} : {}",
                self.current_oid, self.current_timestamp, self.name
            )
        } else {
            write!(
                f,
                "[chg] {} {:?} => {} {:?} : {}",
                self.previous_oid.unwrap(),
                self.previous_timestamp.as_ref().unwrap(),
                self.current_oid,
                self.current_timestamp,
                self.name
            )
        }
    }
}

fn mirror(config: &RepositoryMapping) -> Result<MirrorResult> {
    let repo = match Repository::open(&config.path) {
        Ok(r) => r,
        Err(_) => Repository::init_opts(
            &config.path,
            RepositoryInitOptions::new()
                .bare(true)
                .mode(RepositoryInitMode::SHARED_GROUP),
        )?,
    };

    Command::new("git")
        .args([
            "-C",
            config.path.to_str().unwrap(),
            "config",
            "core.logAllRefUpdates",
            "always",
        ])
        .output()?;

    let mut remote = match repo.find_remote("origin") {
        Ok(r) => r,
        Err(_) => repo.remote("origin", config.git_url.as_str())?,
    };

    let mut remote2 = remote.clone();
    remote2.connect(Direction::Fetch)?;
    // Note that references using "peeled" syntax are manually excluded here.
    // I'm not sure if doing this is to be expected or not, this is simply a syntax to get to the
    // first non tag object, so not actually a reference in it's own right.
    let remote_refs: Vec<&RemoteHead<'_>> = remote2
        .list()?
        .iter()
        .filter(|h| !h.name().ends_with("^{}"))
        .filter(|h| config.ref_match.matches(h.name()))
        .collect();

    let ref_names: Vec<&str> = remote_refs.iter().map(|h| h.name()).collect();
    if ref_names.is_empty() {
        return Err(anyhow!("Matched zero remote refs"));
    }

    remote.fetch(&ref_names, None, Some("git-collage fetch"))?;

    let mut ref_reports = Vec::new();

    for r in &remote_refs {
        let reflog = repo.reflog(r.name())?;
        let (previous_oid, previous_timestamp) = match reflog.get(0) {
            Some(l) => (Some(l.id_new()), Some(l.committer().when())),
            None => (None, None),
        };

        repo.reference(r.name(), r.oid(), true, "git-collage update")?;

        let reflog = repo.reflog(r.name())?;
        let current_oid = reflog.get(0).unwrap().id_new();
        let current_timestamp = reflog.get(0).unwrap().committer().when();

        ref_reports.push(RefStatusReport::new(
            r.name(),
            previous_oid,
            previous_timestamp,
            current_oid,
            current_timestamp,
        ));
    }

    Ok(MirrorResult {
        mapping: config.clone(),
        refs: ref_reports,
    })
}

pub(crate) fn run(
    mappings: &[RepositoryMapping],
    s: Sender<CommandResult>,
) -> std::result::Result<(), usize> {
    let failure_count = mappings
        .par_iter()
        .map(|m| {
            log::info!("Processing: {}", m);

            let result = mirror(m);

            let new_result = match result {
                Ok(_) => Ok(()),
                Err(_) => Err(()),
            };

            s.send(match result {
                Ok(r) => Ok(Box::new(r)),
                Err(ref e) => Err(Box::new(CommandError {
                    identifier: m.to_string(),
                    msg: e.to_string(),
                })),
            })
            .unwrap();

            new_result
        })
        .filter(|r| r.is_err())
        .count();

    if failure_count == 0 {
        Ok(())
    } else {
        Err(failure_count)
    }
}
