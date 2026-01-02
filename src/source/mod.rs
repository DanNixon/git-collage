mod github_authed_user;
mod static_list;

use crate::{
    matching_rules::Ruleset,
    source::{github_authed_user::GithubAuthenticatedUser, static_list::StaticList},
};
use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;
use url::Url;

pub(crate) trait SourceRepositoryMappingProducer {
    async fn repository_mappings(&self) -> Result<Vec<SourceRepositoryMapping>>;
}

#[derive(Clone, Debug)]
pub(crate) struct SourceRepositoryMapping {
    pub path: PathBuf,
    pub ref_match: Option<Ruleset>,
    pub git_url: Url,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum Provider {
    StaticList(StaticList),
    GithubAuthenticatedUser(GithubAuthenticatedUser),
}

impl SourceRepositoryMappingProducer for Provider {
    async fn repository_mappings(&self) -> Result<Vec<SourceRepositoryMapping>> {
        match &self {
            Provider::StaticList(p) => p.repository_mappings().await,
            Provider::GithubAuthenticatedUser(p) => p.repository_mappings().await,
        }
    }
}
