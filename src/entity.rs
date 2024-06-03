use std::fmt::{Display, Write};
use std::hash::{Hash, Hasher};

use indexmap::IndexMap;

#[derive(Debug)]
#[cfg_attr(test, derive(Clone))]
pub(crate) struct Commit {
    pub sha: String,
    pub summary: String,
}

pub(crate) struct MetricStackIterator {
    inner: indexmap::map::IntoIter<MetricHeader, f64>,
}

impl Iterator for MetricStackIterator {
    type Item = Metric;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(header, value)| Metric { header, value })
    }
}

#[derive(Debug, Default)]
pub(crate) struct MetricStack {
    inner: IndexMap<MetricHeader, f64>,
}

impl MetricStack {
    #[inline]
    pub(crate) fn from_iter(iter: impl Iterator<Item = Metric>) -> Self {
        Self {
            inner: IndexMap::from_iter(iter.map(|Metric { header, value }| (header, value))),
        }
    }

    pub(crate) fn with_change(mut self, change: MetricChange) -> Self {
        match change {
            MetricChange::Add(Metric { header, value }) => {
                self.inner.insert(header, value);
            }
            MetricChange::Remove(Metric { header, value }) => match self.inner.get(&header) {
                Some(existing) if *existing == value => {
                    self.inner.swap_remove(&header);
                }
                _ => {}
            },
        };
        self
    }

    pub(crate) fn with_changes(self, iter: impl Iterator<Item = MetricChange>) -> Self {
        iter.fold(self, |this, change| this.with_change(change))
    }

    pub(crate) fn into_metric_iter(self) -> MetricStackIterator {
        MetricStackIterator {
            inner: self.inner.into_iter(),
        }
    }

    pub(crate) fn into_vec(self) -> Vec<Metric> {
        self.into_metric_iter().collect()
    }

    pub(crate) fn at(&self, index: usize) -> Option<(&MetricHeader, f64)> {
        self.inner
            .get_index(index)
            .map(|(header, value)| (header, *value))
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(tag = "action", rename_all = "lowercase")]
pub(crate) enum MetricChange {
    Add(Metric),
    Remove(Metric),
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
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
        if !self.tags.is_empty() {
            f.write_char('{')?;
            for (index, (key, value)) in self.tags.iter().enumerate() {
                if index > 0 {
                    f.write_str(", ")?;
                }
                write!(f, "{key}={value:?}")?;
            }
            f.write_char('}')?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(test, derive(Clone))]
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
        write!(f, "{} {:?}", self.header, self.value)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn should_display_metric_with_single_tag() {
        let item = super::Metric::new("name", 12.34).with_tag("foo", "bar");
        assert_eq!(item.to_string(), "name{foo=\"bar\"} 12.34");
    }

    #[test]
    fn should_display_metric_with_multiple_tags() {
        let item = super::Metric::new("name", 12.34)
            .with_tag("foo", "bar")
            .with_tag("ab", "cd");
        assert_eq!(item.to_string(), "name{foo=\"bar\", ab=\"cd\"} 12.34");
    }

    #[test]
    fn should_display_metric_with_empty_tags() {
        let item = super::Metric::new("name", 12.34);
        assert_eq!(item.to_string(), "name 12.34");
    }
}
