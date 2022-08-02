use super::FilterRepository;
use crate::source::SourceRepositoryMapping;
use serde::Deserialize;
use url::Url;

fn remove_url_credentials(mut u: url::Url) -> url::Url {
    u.set_username("").unwrap();
    u.set_password(None).unwrap();
    u
}

#[derive(Debug, Deserialize)]
pub(crate) struct Blocklist {
    pub(super) urls: Vec<Url>,
}

impl FilterRepository for Blocklist {
    fn filter(&self, r: &SourceRepositoryMapping) -> bool {
        !self
            .urls
            .contains(&remove_url_credentials(r.git_url.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn filter_basics() {
        let filter = Blocklist {
            urls: vec![
                url::Url::parse("https://github.com/dannixon/nope").unwrap(),
                url::Url::parse("https://github.com/dannixon/nope2.git").unwrap(),
            ],
        };

        assert!(filter.filter(&SourceRepositoryMapping {
            path: PathBuf::new(),
            ref_match: None,
            git_url: url::Url::parse("https://github.com/dannixon/yes").unwrap(),
        }));

        assert!(!filter.filter(&SourceRepositoryMapping {
            path: PathBuf::new(),
            ref_match: None,
            git_url: url::Url::parse("https://github.com/dannixon/nope").unwrap(),
        }));
    }

    #[test]
    fn filter_with_credentials_in_url() {
        let filter = Blocklist {
            urls: vec![
                url::Url::parse("https://github.com/dannixon/nope").unwrap(),
                url::Url::parse("https://github.com/dannixon/nope2.git").unwrap(),
            ],
        };

        assert!(filter.filter(&SourceRepositoryMapping {
            path: PathBuf::new(),
            ref_match: None,
            git_url: url::Url::parse("https://user:pass@github.com/dannixon/yes").unwrap(),
        }));

        assert!(!filter.filter(&SourceRepositoryMapping {
            path: PathBuf::new(),
            ref_match: None,
            git_url: url::Url::parse("https://user:pass@github.com/dannixon/nope").unwrap(),
        }));
    }
}
