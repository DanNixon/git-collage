use crate::{
    filter::{Chain, FilterRepository},
    matching_rules::Ruleset,
    source::{Provider, SourceRepositoryMappingProducer},
    util::safe_display_url,
};
use anyhow::{Result, anyhow};
use futures::stream::{self, StreamExt};
use log::{error, warn};
use serde::Deserialize;
use std::{collections::HashSet, fmt, fs, path::PathBuf};
use url::Url;

fn get_config_paths(paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut config_files = Vec::new();
    let mut seen_canonical = HashSet::new();

    for path in paths {
        if path.is_dir() {
            let entries = fs::read_dir(path)?;
            for entry in entries {
                let entry = entry?;
                let entry_path = entry.path();
                if entry_path.is_file() {
                    if entry_path.extension().is_some_and(|ext| ext == "toml") {
                        // Try to canonicalize the path to detect duplicates
                        match entry_path.canonicalize() {
                            Ok(canonical) => {
                                if seen_canonical.insert(canonical.clone()) {
                                    config_files.push(entry_path);
                                } else {
                                    warn!("Duplicate config file: {}", entry_path.display());
                                }
                            }
                            Err(e) => {
                                warn!(
                                    "Failed to canonicalize path {}: {}",
                                    entry_path.display(),
                                    e
                                );
                                // Still add the file if canonicalization fails
                                config_files.push(entry_path);
                            }
                        }
                    } else {
                        warn!("Skipping file: {}", entry_path.display());
                    }
                } else if entry_path.is_dir() {
                    warn!(
                        "Skipping subdirectory (non-recursive config search): {}",
                        entry_path.display()
                    );
                }
            }
        } else if path.is_file() {
            // Try to canonicalize the path to detect duplicates
            match path.canonicalize() {
                Ok(canonical) => {
                    if seen_canonical.insert(canonical) {
                        config_files.push(path.to_path_buf());
                    } else {
                        warn!("Duplicate config file: {}", path.display());
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to canonicalize path {}: {}",
                        path.display(),
                        e
                    );
                    // Still add the file if canonicalization fails
                    config_files.push(path.to_path_buf());
                }
            }
        } else if !path.exists() {
            warn!("Path does not exist: {}", path.display());
        } else {
            warn!("Path is not a file or directory: {}", path.display());
        }
    }

    Ok(config_files)
}

pub(crate) trait RepositoryMappingProducer {
    async fn repository_mappings(&self) -> Vec<Result<RepositoryMapping>>;
}

#[derive(Clone, Debug)]
pub(crate) struct RepositoryMapping {
    pub path: PathBuf,
    pub ref_match: Ruleset,
    pub git_url: Url,
}

impl fmt::Display for RepositoryMapping {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} => {}",
            safe_display_url(self.git_url.clone()),
            self.path.display(),
        )
    }
}

#[derive(Debug)]
pub(crate) struct Config {
    providers: Vec<ProviderConfig>,
}

impl Config {
    pub(crate) fn load(paths: &[PathBuf]) -> Result<Self> {
        let config_files = get_config_paths(paths)?;

        if config_files.is_empty() {
            return Err(anyhow!("No configuration files found"));
        }

        let mut providers = Vec::new();

        for file_path in config_files {
            match fs::read_to_string(&file_path) {
                Ok(s) => match toml::from_str::<ProviderConfig>(&s) {
                    Ok(provider) => providers.push(provider),
                    Err(e) => {
                        error!("Failed to parse config file {}: {}", file_path.display(), e);
                        return Err(anyhow!(
                            "Failed to parse config file {}: {}",
                            file_path.display(),
                            e
                        ));
                    }
                },
                Err(e) => {
                    error!("Failed to read file {}: {}", file_path.display(), e);
                    return Err(anyhow!(
                        "Failed to read file {}: {}",
                        file_path.display(),
                        e
                    ));
                }
            }
        }

        Ok(Self { providers })
    }
}

impl RepositoryMappingProducer for Config {
    async fn repository_mappings(&self) -> Vec<Result<RepositoryMapping>> {
        stream::iter(&self.providers)
            .then(|p| p.repository_mappings())
            .flat_map(stream::iter)
            .collect()
            .await
    }
}

#[derive(Debug, Deserialize)]
struct ProviderConfig {
    path: PathBuf,
    ref_matchers: Ruleset,
    source: Provider,
    #[serde(default)]
    repo_filters: Chain,
}

impl RepositoryMappingProducer for ProviderConfig {
    async fn repository_mappings(&self) -> Vec<Result<RepositoryMapping>> {
        match self.source.repository_mappings().await {
            Ok(m) => m
                .into_iter()
                .filter(|r| self.repo_filters.filter(r))
                .map(|r| {
                    Ok(RepositoryMapping {
                        path: self.path.join(if r.path.has_root() {
                            r.path.strip_prefix("/").unwrap()
                        } else {
                            &r.path
                        }),
                        ref_match: match r.ref_match {
                            Some(m) => m,
                            None => self.ref_matchers.clone(),
                        },
                        git_url: r.git_url,
                    })
                })
                .collect(),
            Err(e) => vec![Err(e)],
        }
    }
}
