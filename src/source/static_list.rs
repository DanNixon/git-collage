use crate::{
    matching_rules::Ruleset,
    source::{SourceRepositoryMapping, SourceRepositoryMappingProducer},
};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use std::path::PathBuf;
use url::Url;

#[derive(Debug, Deserialize)]
pub(crate) struct StaticList {
    repos: Vec<Repository>,
}

#[async_trait]
impl SourceRepositoryMappingProducer for StaticList {
    async fn repository_mappings(&self) -> Result<Vec<SourceRepositoryMapping>> {
        Ok(self
            .repos
            .iter()
            .map(|r| SourceRepositoryMapping {
                path: match &r.path {
                    Some(p) => p.clone(),
                    None => r.git_url.path().into(),
                },
                ref_match: r.ref_match.clone(),
                git_url: r.git_url.clone(),
            })
            .collect())
    }
}

#[derive(Debug, Deserialize)]
struct Repository {
    pub git_url: Url,
    pub path: Option<PathBuf>,
    pub ref_match: Option<Ruleset>,
}
