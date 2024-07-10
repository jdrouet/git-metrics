use super::metric::{MetricHeader, MetricStack};

#[derive(Default)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub(crate) struct Delta {
    #[allow(dead_code)]
    pub(crate) absolute: f64,
    pub(crate) relative: Option<f64>,
}

impl Delta {
    pub fn new(previous: f64, current: f64) -> Self {
        let absolute = current - previous;
        let relative = if previous == 0.0 {
            None
        } else {
            Some(absolute / previous)
        };

        Self { absolute, relative }
    }
}

#[cfg_attr(test, derive(Debug, PartialEq))]
pub(crate) enum Comparison {
    Created {
        current: f64,
    },
    Missing {
        previous: f64,
    },
    Matching {
        #[allow(dead_code)]
        previous: f64,
        current: f64,
        delta: Delta,
    },
}

#[cfg(test)]
impl Comparison {
    #[inline]
    pub fn matching(previous: f64, current: f64) -> Self {
        Self::Matching {
            previous,
            current,
            delta: Delta::new(previous, current),
        }
    }
}

impl Comparison {
    pub fn has_current(&self) -> bool {
        matches!(self, Self::Created { .. } | Self::Matching { .. })
    }

    pub fn created(current: f64) -> Self {
        Self::Created { current }
    }

    pub fn new(previous: f64, current: Option<f64>) -> Self {
        if let Some(current) = current {
            Self::Matching {
                previous,
                current,
                delta: Delta::new(previous, current),
            }
        } else {
            Self::Missing { previous }
        }
    }
}

#[cfg_attr(test, derive(Debug, PartialEq))]
pub(crate) struct MetricDiff {
    pub header: MetricHeader,
    pub comparison: Comparison,
}

#[cfg(test)]
impl MetricDiff {
    pub fn new(header: MetricHeader, comparison: Comparison) -> Self {
        Self { header, comparison }
    }
}

pub(crate) struct MetricDiffList(pub(crate) Vec<MetricDiff>);

impl MetricDiffList {
    pub fn new(previous: MetricStack, mut current: MetricStack) -> Self {
        let mut result = Vec::new();
        for (header, previous_value) in previous.into_inner().into_iter() {
            let current_value = current.remove_entry(&header).map(|(_, value)| value);
            result.push(MetricDiff {
                header,
                comparison: Comparison::new(previous_value, current_value),
            });
        }
        for (header, value) in current.into_inner().into_iter() {
            result.push(MetricDiff {
                header,
                comparison: Comparison::created(value),
            });
        }
        Self(result)
    }

    pub fn remove_missing(self) -> Self {
        Self(
            self.0
                .into_iter()
                .filter(|m| m.comparison.has_current())
                .collect(),
        )
    }

    pub fn inner(&self) -> &[MetricDiff] {
        &self.0
    }

    pub fn into_inner(self) -> Vec<MetricDiff> {
        self.0
    }
}
