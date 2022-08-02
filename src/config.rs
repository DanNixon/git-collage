use crate::{
    filter::{Chain, FilterRepository},
    matching_rules::Ruleset,
    source::{Provider, SourceRepositoryMappingProducer},
    util::safe_display_url,
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::stream::{self, StreamExt};
use serde::Deserialize;
use std::{fmt, fs, path::PathBuf};
use url::Url;

#[async_trait]
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

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    providers: Vec<ProviderConfig>,
}

impl Config {
    pub(crate) fn load(files: &[PathBuf]) -> Result<Self> {
        let configs: Vec<Config> = files
            .iter()
            .filter_map(|p| match fs::read_to_string(&p) {
                Ok(s) => Some(s),
                Err(e) => {
                    log::error!("Failed to read file: {}", e);
                    None
                }
            })
            .filter_map(|s| match toml::from_str::<Config>(&s) {
                Ok(c) => Some(c),
                Err(e) => {
                    log::error!("Failed to parse config: {}", e);
                    None
                }
            })
            .collect();

        if configs.len() == files.len() {
            Ok(Self {
                providers: configs.into_iter().flat_map(|c| c.providers).collect(),
            })
        } else {
            Err(anyhow!(
                "Some config files failed to load (loaded {} out of {})",
                configs.len(),
                files.len()
            ))
        }
    }
}

#[async_trait]
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
    ref_match: Ruleset,
    source: Provider,
    #[serde(default, flatten)]
    filters: Chain,
}

#[async_trait]
impl RepositoryMappingProducer for ProviderConfig {
    async fn repository_mappings(&self) -> Vec<Result<RepositoryMapping>> {
        match self.source.repository_mappings().await {
            Ok(m) => m
                .into_iter()
                .filter(|r| self.filters.filter(r))
                .map(|r| {
                    Ok(RepositoryMapping {
                        path: self.path.join(if r.path.has_root() {
                            r.path.strip_prefix("/").unwrap()
                        } else {
                            &r.path
                        }),
                        ref_match: match r.ref_match {
                            Some(m) => m,
                            None => self.ref_match.clone(),
                        },
                        git_url: r.git_url,
                    })
                })
                .collect(),
            Err(e) => vec![Err(e)],
        }
    }
}
