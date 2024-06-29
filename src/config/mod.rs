use std::{path::Path, str::FromStr};

use indexmap::IndexMap;

use crate::entity::MetricHeader;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub(crate) enum Rule {
    Max { value: f64 },
    Min { value: f64 },
    MaxIncrease { ratio: f64 },
    MaxDecrease { ratio: f64 },
}

impl Rule {
    fn satisfies(&self, previous: Option<f64>, current: f64) -> bool {
        match self {
            Self::Max { value } => current <= *value,
            Self::Min { value } => current >= *value,
            Self::MaxIncrease { ratio } => previous
                .map(|prev| (current - prev) / prev)
                .map(|value| value <= *ratio)
                .unwrap_or(true),
            Self::MaxDecrease { ratio } => previous
                .map(|prev| (current - prev) / prev)
                .map(|value| value >= *ratio * -1.0)
                .unwrap_or(true),
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

    fn satisfies(&self, header: &MetricHeader, previous: Option<f64>, current: f64) -> bool {
        // if they don't match all tags, then it satisfies
        if !self.match_all_tags(header) {
            return true;
        }

        self.rules.iter().all(|r| r.satisfies(previous, current))
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
    fn satisfies(&self, header: &MetricHeader, previous: Option<f64>, current: f64) -> bool {
        if !self.rules.iter().all(|r| r.satisfies(previous, current)) {
            return false;
        }

        self.subsets
            .values()
            .all(|subset| subset.satisfies(header, previous, current))
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct Config {
    #[serde(default)]
    metrics: IndexMap<String, MetricConfig>,
}

impl Config {
    fn satisfies(&self, header: &MetricHeader, previous: Option<f64>, current: f64) -> bool {
        self.metrics
            .get(&header.name)
            .map(|m| m.satisfies(header, previous, current))
            .unwrap_or(true)
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

    #[test_case::test_case(None, 5.0, true; "with smaller value")]
    #[test_case::test_case(None, 10.0, true; "with same value")]
    #[test_case::test_case(None, 12.0, false; "with bigger value")]
    fn should_comply_max_rules(previous: Option<f64>, current: f64, expected: bool) {
        assert_eq!(
            super::Rule::Max { value: 10.0 }.satisfies(previous, current),
            expected
        );
    }

    #[test_case::test_case(None, 5.0, false; "with smaller value")]
    #[test_case::test_case(None, 10.0, true; "with same value")]
    #[test_case::test_case(None, 15.0, true; "with bigger value")]
    fn should_comply_min_rules(previous: Option<f64>, current: f64, expected: bool) {
        assert_eq!(
            super::Rule::Min { value: 10.0 }.satisfies(previous, current),
            expected
        );
    }

    #[test_case::test_case(None, 100.0, true; "without previous value")]
    #[test_case::test_case(Some(120.0), 100.0, true; "with decrease")]
    #[test_case::test_case(Some(100.0), 101.0, true; "with small increase")]
    #[test_case::test_case(Some(100.0), 110.0, true; "with same increase")]
    #[test_case::test_case(Some(100.0), 120.0, false; "with big increase")]
    fn should_comply_max_increase_rules(previous: Option<f64>, current: f64, expected: bool) {
        assert_eq!(
            super::Rule::MaxIncrease { ratio: 0.1 }.satisfies(previous, current),
            expected
        );
    }

    #[test_case::test_case(None, 100.0, true; "without previous value")]
    #[test_case::test_case(Some(100.0), 110.0, true; "with increase")]
    #[test_case::test_case(Some(100.0), 99.0, true; "with small decrease")]
    #[test_case::test_case(Some(100.0), 90.0, true; "with same decrease")]
    #[test_case::test_case(Some(100.0), 80.0, false; "with big decrease")]
    fn should_comply_max_decrease_rules(previous: Option<f64>, current: f64, expected: bool) {
        assert_eq!(
            super::Rule::MaxDecrease { ratio: 0.1 }.satisfies(previous, current),
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
        assert!(config.satisfies(&header, None, 15.0));
        assert!(!config.satisfies(&header, None, 8.0));
        assert!(!config.satisfies(&header, None, 22.0));
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
        assert!(config.satisfies(&header, None, 100.0));
        assert!(config.satisfies(&header, Some(100.0), 100.0));
        assert!(!config.satisfies(&header, Some(100.0), 80.0));
        assert!(!config.satisfies(&header, Some(100.0), 120.0));
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
        assert!(config.satisfies(&first, None, 100.0));
        assert!(config.satisfies(&first, Some(100.0), 100.0));
        assert!(!config.satisfies(&first, Some(100.0), 80.0));
        assert!(!config.satisfies(&first, Some(100.0), 120.0));

        let first_linux = crate::entity::MetricHeader {
            name: "first".into(),
            tags: {
                let mut tags: IndexMap<String, String> = Default::default();
                tags.insert("platform.os".into(), "linux".into());
                tags
            },
        };
        assert!(config.satisfies(&first_linux, None, 100.0));
        assert!(config.satisfies(&first_linux, Some(100.0), 100.0));
        assert!(!config.satisfies(&first_linux, Some(100.0), 80.0));
        assert!(config.satisfies(&first_linux, Some(100.0), 104.0));
        assert!(!config.satisfies(&first_linux, Some(100.0), 106.0));

        let second = crate::entity::MetricHeader {
            name: "second".into(),
            tags: Default::default(),
        };
        assert!(config.satisfies(&second, None, 100.0));
    }
}
