use super::FilterRepository;
use crate::source::SourceRepositoryMapping;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct UrlPathPrefix {
    prefixes: Vec<String>,
}

impl FilterRepository for UrlPathPrefix {
    fn filter(&self, r: &SourceRepositoryMapping) -> bool {
        self.prefixes
            .iter()
            .any(|p| r.git_url.path().starts_with(p))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn filter_basics() {
        let filter = UrlPathPrefix {
            prefixes: vec!["/one".to_string(), "/three".to_string()],
        };

        assert!(filter.filter(&SourceRepositoryMapping {
            path: PathBuf::new(),
            ref_match: None,
            git_url: url::Url::parse("https://github.com/one/repo").unwrap(),
        }));

        assert!(!filter.filter(&SourceRepositoryMapping {
            path: PathBuf::new(),
            ref_match: None,
            git_url: url::Url::parse("https://github.com/two/repo").unwrap(),
        }));

        assert!(filter.filter(&SourceRepositoryMapping {
            path: PathBuf::new(),
            ref_match: None,
            git_url: url::Url::parse("https://github.com/three/repo").unwrap(),
        }));
    }
}
