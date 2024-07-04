use std::{path::Path, str::FromStr};

use indexmap::IndexMap;

use crate::entity::MetricHeader;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub(crate) enum RuleError {
    Max { value: f64, limit: f64 },
    Min { value: f64, limit: f64 },
    MaxIncrease { value: f64, limit: f64 },
    MaxDecrease { value: f64, limit: f64 },
}

impl std::fmt::Display for RuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Max { value, limit } => {
                write!(f, "{value} is greater than the max allowed {limit}")
            }
            Self::Min { value, limit } => write!(f, "{value} is less than the min allowed {limit}"),
            Self::MaxDecrease { value, limit } => write!(
                f,
                "decreased of {:.1}%, with a limit at {:.1}%",
                value * 100.0,
                limit * 100.0
            ),
            Self::MaxIncrease { value, limit } => write!(
                f,
                "increased of {:.1}%, with a limit at {:.1}%",
                value * 100.0,
                limit * 100.0
            ),
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub(crate) enum Rule {
    Max { value: f64 },
    Min { value: f64 },
    MaxIncrease { ratio: f64 },
    MaxDecrease { ratio: f64 },
}

impl Rule {
    fn check(&self, previous: Option<f64>, current: f64) -> Option<RuleError> {
        match (previous, self) {
            (_, Self::Max { value }) if current > *value => Some(RuleError::Max {
                value: current,
                limit: *value,
            }),
            (_, Self::Min { value }) if current < *value => Some(RuleError::Min {
                value: current,
                limit: *value,
            }),
            (Some(previous), Self::MaxIncrease { ratio }) if previous != 0.0 => {
                let value = (current - previous) / previous;
                if value > *ratio {
                    Some(RuleError::MaxIncrease {
                        value,
                        limit: *ratio,
                    })
                } else {
                    None
                }
            }
            (Some(previous), Self::MaxDecrease { ratio }) if previous != 0.0 => {
                let value = (previous - current) / previous;
                if value > *ratio {
                    Some(RuleError::MaxDecrease {
                        value,
                        limit: *ratio,
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct SubsetConfig {
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    matching: IndexMap<String, String>,
    #[serde(default)]
    rules: Vec<Rule>,
}

impl SubsetConfig {
    fn match_all_tags(&self, header: &MetricHeader) -> bool {
        self.matching
            .iter()
            .all(|(key, value)| header.tags.get(key).map(|v| v == value).unwrap_or(false))
    }

    fn check(
        &self,
        header: &MetricHeader,
        previous: Option<f64>,
        current: f64,
        failing: &mut Vec<RuleError>,
    ) {
        // if they don't match all tags, then it satisfies
        if !self.match_all_tags(header) {
            return;
        }

        for rule in self.rules.iter() {
            if let Some(error) = rule.check(previous, current) {
                failing.push(error);
            }
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct MetricConfig {
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    rules: Vec<Rule>,
    #[serde(default)]
    subsets: IndexMap<String, SubsetConfig>,
}

impl MetricConfig {
    fn check(
        &self,
        header: &MetricHeader,
        previous: Option<f64>,
        current: f64,
        failing: &mut Vec<RuleError>,
    ) {
        for rule in self.rules.iter() {
            if let Some(error) = rule.check(previous, current) {
                failing.push(error);
            }
        }

        for subset in self.subsets.values() {
            subset.check(header, previous, current, failing);
        }
    }
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub(crate) struct Config {
    #[serde(default)]
    metrics: IndexMap<String, MetricConfig>,
}

impl Config {
    pub(crate) fn check(
        &self,
        header: &MetricHeader,
        previous: Option<f64>,
        current: f64,
    ) -> Vec<RuleError> {
        let mut rules = Vec::new();
        if let Some(m) = self.metrics.get(&header.name) {
            m.check(header, previous, current, &mut rules);
        }
        rules
    }
}

impl Config {
    pub(crate) fn from_path(path: &Path) -> std::io::Result<Config> {
        let content = std::fs::read_to_string(path)?;

        Config::from_str(content.as_str())
    }
}

impl FromStr for Config {
    type Err = std::io::Error;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        use std::io::{Error, ErrorKind};

        toml::de::from_str(source).map_err(|err| Error::new(ErrorKind::InvalidData, err))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use indexmap::IndexMap;

    #[test]
    fn should_format_max() {
        assert_eq!(
            super::RuleError::Max {
                value: 20.0,
                limit: 10.0
            }
            .to_string(),
            "20 is greater than the max allowed 10"
        );
    }

    #[test]
    fn should_format_min() {
        assert_eq!(
            super::RuleError::Min {
                value: 10.0,
                limit: 20.0
            }
            .to_string(),
            "10 is less than the min allowed 20"
        );
    }

    #[test]
    fn should_format_max_decrease() {
        assert_eq!(
            super::RuleError::MaxDecrease {
                value: 0.2,
                limit: 0.1
            }
            .to_string(),
            "decreased of 20.0%, with a limit at 10.0%"
        );
    }

    #[test]
    fn should_format_max_increase() {
        assert_eq!(
            super::RuleError::MaxIncrease {
                value: 0.2,
                limit: 0.1
            }
            .to_string(),
            "increased of 20.0%, with a limit at 10.0%"
        );
    }

    #[test_case::test_case(None, 5.0, None; "with smaller value")]
    #[test_case::test_case(None, 10.0, None; "with same value")]
    #[test_case::test_case(None, 12.0, Some(crate::config::RuleError::Max { value: 12.0, limit: 10.0 }); "with bigger value")]
    fn should_comply_max_rules(
        previous: Option<f64>,
        current: f64,
        expected: Option<super::RuleError>,
    ) {
        assert_eq!(
            super::Rule::Max { value: 10.0 }.check(previous, current),
            expected
        );
    }

    #[test_case::test_case(None, 5.0, Some(crate::config::RuleError::Min { value: 5.0, limit: 10.0 }); "with smaller value")]
    #[test_case::test_case(None, 10.0, None; "with same value")]
    #[test_case::test_case(None, 15.0, None; "with bigger value")]
    fn should_comply_min_rules(
        previous: Option<f64>,
        current: f64,
        expected: Option<super::RuleError>,
    ) {
        assert_eq!(
            super::Rule::Min { value: 10.0 }.check(previous, current),
            expected
        );
    }

    #[test_case::test_case(None, 100.0, None; "without previous value")]
    #[test_case::test_case(Some(120.0), 100.0, None; "with decrease")]
    #[test_case::test_case(Some(100.0), 101.0, None; "with small increase")]
    #[test_case::test_case(Some(100.0), 110.0, None; "with same increase")]
    #[test_case::test_case(Some(100.0), 120.0, Some(crate::config::RuleError::MaxIncrease { value: 0.2, limit: 0.1 }); "with big increase")]
    fn should_comply_max_increase_rules(
        previous: Option<f64>,
        current: f64,
        expected: Option<super::RuleError>,
    ) {
        assert_eq!(
            super::Rule::MaxIncrease { ratio: 0.1 }.check(previous, current),
            expected
        );
    }

    #[test_case::test_case(None, 100.0, None; "without previous value")]
    #[test_case::test_case(Some(100.0), 110.0, None; "with increase")]
    #[test_case::test_case(Some(100.0), 99.0, None; "with small decrease")]
    #[test_case::test_case(Some(100.0), 90.0, None; "with same decrease")]
    #[test_case::test_case(Some(100.0), 80.0, Some(crate::config::RuleError::MaxDecrease { value: 0.2, limit: 0.1 }); "with big decrease")]
    fn should_comply_max_decrease_rules(
        previous: Option<f64>,
        current: f64,
        expected: Option<super::RuleError>,
    ) {
        assert_eq!(
            super::Rule::MaxDecrease { ratio: 0.1 }.check(previous, current),
            expected
        );
    }

    fn should_deserialize(payload: &str, names: &[&str]) {
        let config = super::Config::from_str(payload).unwrap();
        for name in names {
            assert!(
                config.metrics.contains_key(*name),
                "should contain key {name}"
            );
        }
    }

    #[test]
    fn should_deserialize_with_simple_name() {
        should_deserialize(
            r#"[metrics.binary_size]
    rules = [{ type = "max-increase", ratio = 0.1 }]
    "#,
            &["binary_size"],
        );
    }

    #[test]
    fn should_deserialize_with_name_containing_dot() {
        should_deserialize(
            r#"[metrics.binary_size]
    rules = [{ type = "max-increase", ratio = 0.1 }]

    [metrics."binary.size"]
    rules = [{ type = "max-increase", ratio = 0.1 }]
    "#,
            &["binary_size", "binary.size"],
        );
    }

    #[test]
    fn should_match_abstract_rules() {
        let config = super::Config::from_str(
            r#"[[metrics.binary_size.rules]]
type = "min"
value = 10.0

[[metrics.binary_size.rules]]
type = "max"
value = 20.0
"#,
        )
        .unwrap();
        let header = crate::entity::MetricHeader {
            name: "binary_size".into(),
            tags: Default::default(),
        };
        assert!(config.check(&header, None, 15.0).is_empty());
        assert_eq!(
            config.check(&header, None, 8.0),
            vec![super::RuleError::Min {
                value: 8.0,
                limit: 10.0
            }]
        );
        assert_eq!(
            config.check(&header, None, 22.0),
            vec![super::RuleError::Max {
                value: 22.0,
                limit: 20.0
            }]
        );
    }

    #[test]
    fn should_match_relative_rules() {
        let config = super::Config::from_str(
            r#"[[metrics.binary_size.rules]]
type = "max-decrease"
ratio = 0.1

[[metrics.binary_size.rules]]
type = "max-increase"
ratio = 0.1
"#,
        )
        .unwrap();
        let header = crate::entity::MetricHeader {
            name: "binary_size".into(),
            tags: Default::default(),
        };
        assert!(config.check(&header, None, 100.0).is_empty());
        assert!(config.check(&header, Some(100.0), 100.0).is_empty());
        assert_eq!(
            config.check(&header, Some(100.0), 80.0),
            vec![super::RuleError::MaxDecrease {
                value: 0.2,
                limit: 0.1
            }]
        );
        assert_eq!(
            config.check(&header, Some(100.0), 120.0),
            vec![super::RuleError::MaxIncrease {
                value: 0.2,
                limit: 0.1
            }]
        );
    }

    #[test]
    fn matching_complex() {
        let config = super::Config::from_str(
            r#"[metrics.first]
description = "This is the first metric"

[[metrics.first.rules]]
type = "max-decrease"
ratio = 0.1

[[metrics.first.rules]]
type = "max-increase"
ratio = 0.1

[metrics.first.subsets.for-linux-platforms]
description = "for linux platforms"
matching = { "platform.os" = "linux" }

[[metrics.first.subsets.for-linux-platforms.rules]]
type = "max-increase"
ratio = 0.05
"#,
        )
        .unwrap();
        let first = crate::entity::MetricHeader {
            name: "first".into(),
            tags: Default::default(),
        };
        assert!(config.check(&first, None, 100.0).is_empty());
        assert!(config.check(&first, Some(100.0), 100.0).is_empty());
        assert_eq!(
            config.check(&first, Some(100.0), 80.0),
            vec![super::RuleError::MaxDecrease {
                value: 0.2,
                limit: 0.1
            }]
        );
        assert_eq!(
            config.check(&first, Some(100.0), 120.0),
            vec![super::RuleError::MaxIncrease {
                value: 0.2,
                limit: 0.1
            }]
        );

        let first_linux = crate::entity::MetricHeader {
            name: "first".into(),
            tags: {
                let mut tags: IndexMap<String, String> = Default::default();
                tags.insert("platform.os".into(), "linux".into());
                tags
            },
        };
        assert!(config.check(&first_linux, None, 100.0).is_empty());
        assert!(config.check(&first_linux, Some(100.0), 100.0).is_empty());
        assert_eq!(
            config.check(&first_linux, Some(100.0), 80.0),
            vec![super::RuleError::MaxDecrease {
                value: 0.2,
                limit: 0.1
            }]
        );
        assert!(config.check(&first_linux, Some(100.0), 104.0).is_empty());
        assert_eq!(
            config.check(&first_linux, Some(100.0), 106.0),
            vec![super::RuleError::MaxIncrease {
                value: 0.06,
                limit: 0.05
            }]
        );

        let second = crate::entity::MetricHeader {
            name: "second".into(),
            tags: Default::default(),
        };
        assert!(config.check(&second, None, 100.0).is_empty());
    }
}