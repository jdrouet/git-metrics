use std::hash::{Hash, Hasher};

use indexmap::IndexMap;

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

#[derive(Debug, Default, serde::Serialize)]
pub(crate) struct MetricStack {
    #[serde(flatten)]
    inner: IndexMap<MetricHeader, f64>,
}

impl MetricStack {
    #[inline]
    pub(crate) fn from_iter(iter: impl Iterator<Item = Metric>) -> Self {
        Self {
            inner: IndexMap::from_iter(iter.map(|Metric { header, value }| (header, value))),
        }
    }

    pub(crate) fn extend(&mut self, other: Self) {
        self.inner.extend(other.inner);
    }

    pub(crate) fn remove_entry(&mut self, header: &MetricHeader) -> Option<(MetricHeader, f64)> {
        self.inner.shift_remove_entry(header)
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

    pub(crate) fn into_inner(self) -> IndexMap<MetricHeader, f64> {
        self.inner
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

#[cfg(test)]
impl MetricHeader {
    pub fn new<N: Into<String>>(name: N) -> Self {
        Self {
            name: name.into(),
            tags: Default::default(),
        }
    }

    pub fn with_tag<N: Into<String>, V: Into<String>>(mut self, name: N, value: V) -> Self {
        self.tags.insert(name.into(), value.into());
        self
    }
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

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(test, derive(Clone))]
pub struct Metric {
    #[serde(flatten)]
    pub header: MetricHeader,
    pub value: f64,
}

impl Metric {
    #[cfg(any(test, feature = "importer-lcov"))]
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

    #[cfg(test)]
    pub(crate) fn with_tag<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.header.tags.insert(key.into(), value.into());
        self
    }
}
