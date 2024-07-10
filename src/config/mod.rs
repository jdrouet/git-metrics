use std::path::Path;
use std::str::FromStr;

use indexmap::IndexMap;

use crate::entity::metric::MetricHeader;

#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub(crate) enum Rule {
    Max { value: f64 },
    Min { value: f64 },
    MaxIncrease { ratio: f64 },
    MaxDecrease { ratio: f64 },
}

#[cfg(test)]
impl Rule {
    pub fn max(value: f64) -> Self {
        Self::Max { value }
    }

    pub fn max_increase(ratio: f64) -> Self {
        Self::MaxIncrease { ratio }
    }

    pub fn min(value: f64) -> Self {
        Self::Min { value }
    }

    pub fn max_decrease(ratio: f64) -> Self {
        Self::MaxDecrease { ratio }
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct SubsetConfig {
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    pub(crate) matching: IndexMap<String, String>,
    #[serde(default)]
    pub rules: Vec<Rule>,
}

impl SubsetConfig {
    pub fn matches(&self, header: &MetricHeader) -> bool {
        self.matching
            .iter()
            .all(|(key, value)| header.tags.get(key).map(|v| v == value).unwrap_or(false))
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct MetricConfig {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub subsets: IndexMap<String, SubsetConfig>,
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub(crate) struct Config {
    #[serde(default)]
    pub metrics: IndexMap<String, MetricConfig>,
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
}
