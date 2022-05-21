use regex::Regex;
use serde::{de::Visitor, Deserialize, Deserializer};
use std::fmt;

pub(crate) trait Match {
    fn matches(&self, name: &str) -> bool;
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Ruleset {
    rules: Vec<Rule>,
}

impl Match for Ruleset {
    fn matches(&self, name: &str) -> bool {
        self.rules.iter().map(|r| r.matches(name)).any(|i| i)
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", content = "expr", rename_all = "snake_case")]
enum Rule {
    Exact(String),
    Regex(RegexRule),
}

impl Match for Rule {
    fn matches(&self, name: &str) -> bool {
        match &self {
            Rule::Exact(s) => s == name,
            Rule::Regex(r) => r.matches(name),
        }
    }
}

#[derive(Clone, Debug)]
struct RegexRule {
    regex: Regex,
}

impl Match for RegexRule {
    fn matches(&self, name: &str) -> bool {
        self.regex.is_match(name)
    }
}

impl<'de> Deserialize<'de> for RegexRule {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(RegexRuleVisitor)
    }
}

struct RegexRuleVisitor;

impl<'de> Visitor<'de> for RegexRuleVisitor {
    type Value = RegexRule;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a valid regex string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match Regex::new(v) {
            Ok(regex) => Ok(RegexRule { regex }),
            Err(e) => Err(serde::de::Error::custom(format!(
                "failed to parse regex: {}",
                e
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ruleset() {
        let rs = Ruleset {
            rules: vec![
                Rule::Exact("refs/heads/main".to_string()),
                Rule::Regex(RegexRule {
                    regex: Regex::new("refs/tags/.*").unwrap(),
                }),
            ],
        };
        assert!(rs.matches("refs/heads/main"));
        assert!(!rs.matches("refs/heads/develop"));
        assert!(rs.matches("refs/tags/v0.1.0"));
        assert!(rs.matches("refs/tags/v0.1.1"));
    }

    #[test]
    fn rule_exact() {
        let r = Rule::Exact("refs/heads/main".to_string());
        assert!(r.matches("refs/heads/main"));
        assert!(!r.matches("refs/heads/develop"));
    }

    #[test]
    fn rule_regex() {
        let r = Rule::Regex(RegexRule {
            regex: Regex::new("refs/heads/ma.*").unwrap(),
        });
        assert!(r.matches("refs/heads/main"));
        assert!(r.matches("refs/heads/master"));
        assert!(!r.matches("refs/heads/develop"));

        let r = Rule::Regex(RegexRule {
            regex: Regex::new("refs/tags/.*").unwrap(),
        });
        assert!(r.matches("refs/tags/v0.1.0"));
        assert!(r.matches("refs/tags/v0.1.1"));
        assert!(!r.matches("refs/heads/develop"));
    }
}
