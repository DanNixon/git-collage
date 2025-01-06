use crate::source::{SourceRepositoryMapping, SourceRepositoryMappingProducer};
use anyhow::Result;
use async_trait::async_trait;
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Deserialize)]
pub(crate) struct GithubAuthenticatedUser {
    token: String,
    #[serde(flatten)]
    visibility: Visibilities,
    #[serde(flatten)]
    affiliation: Affiliations,
}

#[async_trait]
impl SourceRepositoryMappingProducer for GithubAuthenticatedUser {
    async fn repository_mappings(&self) -> Result<Vec<SourceRepositoryMapping>> {
        let octocrab = Octocrab::builder()
            .personal_token(self.token.clone())
            .build()?;

        let mut page = octocrab
            .current()
            .list_repos_for_authenticated_user()
            .affiliation(self.affiliation.to_string())
            .visibility(self.visibility.to_string())
            .per_page(100)
            .send()
            .await?;

        let mut repos = page.take_items();

        while let Ok(Some(mut new_page)) = octocrab.get_page(&page.next).await {
            repos.extend(new_page.take_items());
            page = new_page;
        }

        let username = octocrab.current().user().await?.login;

        Ok(repos
            .iter()
            .map(|r| {
                let mut git_url = r.clone_url.clone().unwrap();
                git_url.set_username(&username).unwrap();
                git_url.set_password(Some(&self.token)).unwrap();

                SourceRepositoryMapping {
                    path: match &r.full_name {
                        Some(n) => n.clone(),
                        None => r.name.clone(),
                    }
                    .parse()
                    .unwrap(),
                    ref_match: None,
                    git_url,
                }
            })
            .collect())
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Visibility {
    Public,
    Private,
    Internal,
}

#[derive(Debug, Deserialize)]
struct Visibilities {
    visibility: Vec<Visibility>,
}

impl Display for Visibilities {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.visibility.len() {
            1 => write!(
                f,
                "{}",
                serde_variant::to_variant_name(&self.visibility[0]).unwrap()
            ),
            2 => {
                if self.visibility.contains(&Visibility::Private)
                    && (self.visibility.contains(&Visibility::Private)
                        || self.visibility.contains(&Visibility::Internal))
                {
                    write!(f, "both")
                } else {
                    write!(f, "")
                }
            }
            _ => write!(f, ""),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum Affiliation {
    Owner,
    Collaborator,
    OrganizationMember,
}

#[derive(Debug, Deserialize)]
struct Affiliations {
    affiliation: Vec<Affiliation>,
}

impl Display for Affiliations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self
            .affiliation
            .iter()
            .map(|i| serde_variant::to_variant_name(i).unwrap())
            .collect::<Vec<&str>>()
            .join(",");
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visibility_empty() {
        let v = Visibilities { visibility: vec![] };
        assert_eq!(v.to_string(), "");
    }

    #[test]
    fn visibility_single() {
        let v = Visibilities {
            visibility: vec![Visibility::Private],
        };
        assert_eq!(v.to_string(), "private");
    }

    #[test]
    fn visibility_both_public() {
        let v = Visibilities {
            visibility: vec![Visibility::Private, Visibility::Public],
        };
        assert_eq!(v.to_string(), "both");
    }

    #[test]
    fn visibility_both_internal() {
        let v = Visibilities {
            visibility: vec![Visibility::Private, Visibility::Internal],
        };
        assert_eq!(v.to_string(), "both");
    }
}
