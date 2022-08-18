use crate::config::RepositoryMapping;
use clap::Parser;
use rayon::prelude::*;
use std::{
    collections::HashSet,
    fs::{self, DirEntry},
    io,
    path::{Component, Path, PathBuf},
};

fn is_a_git_repo(path: &Path) -> bool {
    // Ignore likely non-bare repos (git-collage will only ever create bare repos)
    if path.components().last() == Some(Component::Normal(".git".as_ref())) {
        return false;
    }

    // Check for some files that will be present in a bare Git repo
    for filename in &["config", "HEAD"] {
        let mut path = path.to_path_buf();
        path.push(filename);
        if !(path.exists() && path.is_file()) {
            return false;
        }
    }

    // TODO: try and open the repo?

    true
}

fn find_git_repos(path: &Path) -> Vec<PathBuf> {
    if is_a_git_repo(path) {
        vec![path.to_path_buf()]
    } else {
        let entries: Vec<DirEntry> = match fs::read_dir(&path) {
            Ok(o) => o.filter_map(|i| i.ok()).collect(),
            Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
                log::warn!("{}: {}", e, &path.display());
                vec![]
            }
            Err(e) => {
                log::error!("{}: {}", e, &path.display());
                vec![]
            }
        };

        entries
            .par_iter()
            .filter(|i| i.file_type().unwrap().is_dir())
            .map(|i| find_git_repos(&i.path()))
            .flatten()
            .collect()
    }
}

#[derive(Debug, Parser)]
pub(crate) struct Cli {
    /// Directories to search for existing local repositories
    #[clap(parse(from_os_str), value_name = "PATH")]
    paths: Vec<PathBuf>,
}

pub(super) fn run(mappings: &[RepositoryMapping], args: &Cli) -> std::result::Result<(), usize> {
    let local_repo_paths: HashSet<PathBuf> =
        HashSet::from_iter(args.paths.iter().flat_map(|i| find_git_repos(i)));
    log::info!("{} repo(s) found locally", local_repo_paths.len());

    let remote_repo_paths: HashSet<PathBuf> =
        HashSet::from_iter(mappings.iter().map(|m| m.path.to_path_buf()));
    log::info!(
        "{} remote repo(s) in configuration",
        remote_repo_paths.len()
    );

    let unknown_repo_paths = local_repo_paths.difference(&remote_repo_paths);
    let mut unknown_repo_paths: Vec<&PathBuf> = unknown_repo_paths.into_iter().collect();
    unknown_repo_paths.sort();

    log::info!(
        "{} local repo(s) are not in configuration",
        unknown_repo_paths.len()
    );
    for p in unknown_repo_paths {
        println!("{}", p.display());
    }

    Ok(())
}
