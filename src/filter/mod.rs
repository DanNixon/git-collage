mod blocklist;

use crate::source::SourceRepositoryMapping;
use enum_dispatch::enum_dispatch;
use serde::Deserialize;

#[enum_dispatch(Filter)]
pub(crate) trait FilterRepository {
    fn filter(&self, _: &SourceRepositoryMapping) -> bool;
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[enum_dispatch]
enum Filter {
    Blocklist(blocklist::Blocklist),
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct Chain {
    filters: Vec<Filter>,
}

impl FilterRepository for Chain {
    fn filter(&self, r: &SourceRepositoryMapping) -> bool {
        for f in &self.filters {
            if !f.filter(r) {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn chain_basics() {
        let chain = Chain {
            filters: vec![
                Filter::Blocklist(blocklist::Blocklist {
                    urls: vec![url::Url::parse("https://github.com/dannixon/nope").unwrap()],
                }),
                Filter::Blocklist(blocklist::Blocklist {
                    urls: vec![url::Url::parse("https://github.com/dannixon/very_nope").unwrap()],
                }),
            ],
        };

        assert!(chain.filter(&SourceRepositoryMapping {
            path: PathBuf::new(),
            ref_match: None,
            git_url: url::Url::parse("https://github.com/dannixon/yes").unwrap(),
        }));

        assert!(!chain.filter(&SourceRepositoryMapping {
            path: PathBuf::new(),
            ref_match: None,
            git_url: url::Url::parse("https://github.com/dannixon/nope").unwrap(),
        }));

        assert!(!chain.filter(&SourceRepositoryMapping {
            path: PathBuf::new(),
            ref_match: None,
            git_url: url::Url::parse("https://github.com/dannixon/very_nope").unwrap(),
        }));
    }
}
