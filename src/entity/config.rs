use std::path::{Path, PathBuf};
use std::str::FromStr;

use human_number::Formatter;
use indexmap::IndexMap;

use super::metric::MetricHeader;

const NO_SCALE: human_number::Scales<'static> = human_number::Scales::new(&[], &[]);

#[inline]
fn undefined_unit_formatter() -> human_number::Formatter<'static> {
    human_number::Formatter::new(NO_SCALE, human_number::Options::default())
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub(crate) struct RuleAbsolute {
    pub value: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub(crate) struct RuleRelative {
    pub ratio: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub(crate) enum RuleChange {
    Absolute(RuleAbsolute),
    Relative(RuleRelative),
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub(crate) enum Rule {
    Max(RuleAbsolute),
    Min(RuleAbsolute),
    MaxIncrease(RuleChange),
    MaxDecrease(RuleChange),
}

#[cfg(test)]
impl Rule {
    pub fn max(value: f64) -> Self {
        Self::Max(RuleAbsolute { value })
    }

    pub fn max_absolute_increase(value: f64) -> Self {
        Self::MaxIncrease(RuleChange::Absolute(RuleAbsolute { value }))
    }

    pub fn max_relative_increase(ratio: f64) -> Self {
        Self::MaxIncrease(RuleChange::Relative(RuleRelative { ratio }))
    }

    pub fn min(value: f64) -> Self {
        Self::Min(RuleAbsolute { value })
    }

    pub fn max_absolute_decrease(value: f64) -> Self {
        Self::MaxDecrease(RuleChange::Absolute(RuleAbsolute { value }))
    }

    pub fn max_relative_decrease(ratio: f64) -> Self {
        Self::MaxDecrease(RuleChange::Relative(RuleRelative { ratio }))
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct SubsetConfig {
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

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum UnitScale {
    #[default]
    SI,
    Binary,
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub(crate) struct Unit {
    #[serde(default)]
    pub scale: Option<UnitScale>,
    #[serde(default)]
    pub suffix: Option<String>,
    #[serde(default)]
    pub decimals: Option<usize>,
}

#[cfg(test)]
impl Unit {
    pub fn new<S: Into<String>>(scale: UnitScale, suffix: Option<S>) -> Self {
        Unit {
            scale: Some(scale),
            suffix: suffix.map(|v| v.into()),
            decimals: None,
        }
    }

    pub fn binary() -> Self {
        Unit::new(UnitScale::Binary, None::<String>)
    }

    pub fn with_suffix<S: Into<String>>(mut self, value: S) -> Self {
        self.suffix = Some(value.into());
        self
    }
}

impl Unit {
    pub fn formater(&self) -> human_number::Formatter<'_> {
        let mut formatter = match self.scale {
            Some(UnitScale::SI) => human_number::Formatter::si(),
            Some(UnitScale::Binary) => human_number::Formatter::binary(),
            None => undefined_unit_formatter(),
        };
        if let Some(ref unit) = self.suffix {
            formatter.set_unit(unit.as_str());
        }
        if let Some(decimals) = self.decimals {
            formatter.set_decimals(decimals);
        }
        formatter
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[cfg_attr(test, derive(Default))]
pub(crate) struct MetricConfig {
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub subsets: IndexMap<String, SubsetConfig>,
    #[serde(default)]
    pub unit: Unit,
}

#[cfg(test)]
impl MetricConfig {
    pub fn with_unit(mut self, unit: Unit) -> Self {
        self.unit = unit;
        self
    }
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub(crate) struct Config {
    #[serde(default)]
    pub metrics: IndexMap<String, MetricConfig>,
}

#[cfg(test)]
impl Config {
    pub fn with_metric<N: Into<String>>(mut self, name: N, value: MetricConfig) -> Self {
        self.metrics.insert(name.into(), value);
        self
    }
}

const fn sample() -> &'static str {
    r#"# # For every metric you want to monitor, you need to add an entry
# [metrics.metric_name.unit]
# # This scale can be "si" for International System of Units or "binary" (optional)
# scale = "si"
# # Any string that will be added as a suffix (optional)
# suffix = "B"
# # Number of decimals for every number (optional)
# decimals = 3
#
# # Set of rules for the unit budget.
# [[metrics.metric_name.rules]]
# type = "max"
# value = 12.34
#
# [[metrics.metric_name.rules]]
# type = "min"
# value = 1.234
#
# [[metrics.metric_name.rules]]
# type = "max-decrease"
# # the metric cannot decrease of more than 5%
# ratio = 0.05
#
# [[metrics.metric_name.rules]]
# type = "max-decrease"
# # the metric cannot decrease of more than 1.234
# value = 1.234
#
# [[metrics.metric_name.rules]]
# type = "max-increase"
# # the metric cannot increase of more than 5%
# ratio = 0.05
#
# [[metrics.metric_name.rules]]
# type = "max-increase"
# # the metric cannot increase of more than 1.234
# value = 1.234
"#
}

impl Config {
    pub(crate) fn from_path(path: &Path) -> std::io::Result<Config> {
        let content = std::fs::read_to_string(path)?;

        Config::from_str(content.as_str())
    }

    fn config_path(root: &Path) -> PathBuf {
        root.join(".git-metrics.toml")
    }

    pub(crate) fn from_root_path(root: &Path) -> std::io::Result<Config> {
        let config_path = Self::config_path(root);
        if config_path.is_file() {
            Config::from_path(&config_path)
        } else {
            Ok(Default::default())
        }
    }

    pub(crate) fn formatter(&self, metric_name: &str) -> Formatter<'_> {
        if let Some(config) = self.metrics.get(metric_name) {
            config.unit.formater()
        } else {
            undefined_unit_formatter()
        }
    }

    pub(crate) fn write_sample(root: &Path) -> std::io::Result<()> {
        let config_path = Self::config_path(root);
        std::fs::write(&config_path, sample())
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

    #[test]
    fn should_parse_config() {
        let without_comment = super::sample()
            .split("\n")
            .filter_map(|line| line.strip_prefix("# "))
            .collect::<Vec<_>>()
            .join("\n");
        super::Config::from_str(&without_comment).unwrap();
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
    fn should_deserialize_with_relative_and_absolute() {
        should_deserialize(
            r#"[metrics.binary_size]
    rules = [{ type = "max-increase", ratio = 0.1 }, { type = "max-increase", value = 1.0 }, { type = "max-decrease", ratio = 0.1 }, { type = "max-decrease", value = 1.0 }]
    "#,
            &["binary_size"],
        );
    }
}
