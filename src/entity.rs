use std::{
    collections::HashSet,
    fmt::{Display, Write},
    hash::{Hash, Hasher},
};

use indexmap::IndexMap;

#[derive(Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct MetricHeader {
    pub name: String,
    #[serde(default)]
    pub tags: IndexMap<String, String>,
}

impl Hash for MetricHeader {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        for (key, value) in self.tags.iter() {
            key.hash(state);
            value.hash(state);
        }
    }
}

impl Display for MetricHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name.as_str())?;
        f.write_char('{')?;
        for (index, (key, value)) in self.tags.iter().enumerate() {
            if index > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{key}={value:?}")?;
        }
        f.write_char('}')?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Metric {
    #[serde(flatten)]
    pub header: MetricHeader,
    pub value: f64,
}

#[cfg(test)]
impl Metric {
    pub(crate) fn new<N>(name: N, value: f64) -> Self
    where
        N: Into<String>,
    {
        Self {
            header: MetricHeader {
                name: name.into(),
                tags: Default::default(),
            },
            value,
        }
    }

    pub(crate) fn with_tag<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.header.tags.insert(key.into(), value.into());
        self
    }
}

impl Display for Metric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = {:?}", self.header, self.value)
    }
}

pub(crate) fn merge_metrics(mut remote: Vec<Metric>, local: Vec<Metric>) -> Vec<Metric> {
    let existing: HashSet<&MetricHeader, std::hash::RandomState> =
        HashSet::from_iter(remote.iter().map(|m| &m.header));
    let missing = local
        .into_iter()
        .filter(|item| !existing.contains(&item.header))
        .collect::<Vec<_>>();
    remote.extend(missing);
    remote
}

#[cfg(test)]
mod tests {
    #[test]
    fn should_display_metric_with_single_tag() {
        let item = super::Metric::new("name", 12.34).with_tag("foo", "bar");
        assert_eq!(item.to_string(), "name{foo=\"bar\"} = 12.34");
    }

    #[test]
    fn should_display_metric_with_multiple_tags() {
        let item = super::Metric::new("name", 12.34)
            .with_tag("foo", "bar")
            .with_tag("ab", "cd");
        assert_eq!(item.to_string(), "name{foo=\"bar\", ab=\"cd\"} = 12.34");
    }

    #[test]
    fn should_display_metric_with_empty_tags() {
        let item = super::Metric::new("name", 12.34);
        assert_eq!(item.to_string(), "name{} = 12.34");
    }
}
