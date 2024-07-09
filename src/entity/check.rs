use crate::config::{Config, MetricConfig, Rule, SubsetConfig};
use indexmap::IndexMap;

use crate::entity::difference::{Comparison, Delta, MetricDiff};

#[cfg_attr(test, derive(Debug, PartialEq))]
pub(crate) enum Status {
    Success,
    Neutral,
    Failed,
}

impl Rule {
    fn check(&self, comparison: &Comparison) -> Status {
        match self {
            Self::Max { value } => match comparison {
                Comparison::Created { current } | Comparison::Matching { current, .. }
                    if current > value =>
                {
                    Status::Failed
                }
                Comparison::Missing { .. } => Status::Neutral,
                _ => Status::Success,
            },
            Self::Min { value } => match comparison {
                Comparison::Created { current } | Comparison::Matching { current, .. }
                    if current < value =>
                {
                    Status::Failed
                }
                Comparison::Missing { .. } => Status::Neutral,
                _ => Status::Success,
            },
            Self::MaxIncrease { ratio } => match comparison {
                Comparison::Matching {
                    delta:
                        Delta {
                            relative: Some(relative),
                            ..
                        },
                    ..
                } if relative > ratio => Status::Failed,
                Comparison::Matching {
                    delta:
                        Delta {
                            relative: Some(_), ..
                        },
                    ..
                } => Status::Success,
                _ => Status::Neutral,
            },
            Self::MaxDecrease { ratio } => match comparison {
                Comparison::Matching {
                    delta:
                        Delta {
                            relative: Some(relative),
                            ..
                        },
                    ..
                } if *relative < (*ratio) * -1.0 => Status::Failed,
                Comparison::Matching {
                    delta:
                        Delta {
                            relative: Some(_), ..
                        },
                    ..
                } => Status::Success,
                _ => Status::Neutral,
            },
        }
    }
}

impl Status {
    fn and(self, other: &Self) -> Self {
        match (self, other) {
            (Self::Failed, _) | (_, Self::Failed) => Self::Failed,
            (Self::Success, _) | (_, Self::Success) => Self::Success,
            _ => Self::Neutral,
        }
    }
}

#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct RuleCheck {
    pub rule: Rule,
    pub status: Status,
}

#[cfg_attr(test, derive(Debug, PartialEq))]
pub(crate) struct SubsetCheck {
    pub matching: IndexMap<String, String>,
    pub checks: Vec<RuleCheck>,
    pub status: Status,
}

#[cfg(test)]
impl SubsetCheck {
    pub fn new(status: Status) -> Self {
        Self {
            matching: Default::default(),
            checks: Default::default(),
            status,
        }
    }

    pub fn with_matching<N: Into<String>, V: Into<String>>(mut self, name: N, value: V) -> Self {
        self.matching.insert(name.into(), value.into());
        self
    }

    pub fn with_check(mut self, rule: Rule, status: Status) -> Self {
        self.checks.push(RuleCheck { rule, status });
        self
    }
}

impl SubsetCheck {
    fn evaluate(config: &SubsetConfig, diff: &MetricDiff) -> Self {
        let mut status = Status::Neutral;
        let mut checks = Vec::with_capacity(config.rules.len());
        if config.matches(&diff.header) {
            for rule in config.rules.iter() {
                let res = rule.check(&diff.comparison);
                status = status.and(&res);
                checks.push(RuleCheck {
                    rule: *rule,
                    status: res,
                });
            }
        }
        Self {
            matching: config.matching.clone(),
            checks,
            status,
        }
    }
}

#[cfg_attr(test, derive(Debug, PartialEq))]
pub(crate) struct MetricCheck {
    pub diff: MetricDiff,
    pub status: Status,
    pub checks: Vec<RuleCheck>,
    pub subsets: IndexMap<String, SubsetCheck>,
}

#[cfg(test)]
impl MetricCheck {
    pub fn new(diff: MetricDiff, status: Status) -> Self {
        Self {
            diff,
            checks: Default::default(),
            subsets: Default::default(),
            status,
        }
    }

    pub fn with_check(mut self, rule: Rule, status: Status) -> Self {
        self.checks.push(RuleCheck { rule, status });
        self
    }

    pub fn with_subset<N: Into<String>>(mut self, name: N, subset: SubsetCheck) -> Self {
        self.subsets.insert(name.into(), subset);
        self
    }
}

impl MetricCheck {
    #[inline]
    fn neutral(diff: MetricDiff) -> Self {
        Self {
            diff,
            checks: Vec::with_capacity(0),
            subsets: IndexMap::with_capacity(0),
            status: Status::Neutral,
        }
    }

    fn evaluate(config: &MetricConfig, diff: MetricDiff) -> Self {
        let mut global_status = Status::Neutral;

        let mut checks = Vec::with_capacity(config.rules.len());
        for rule in config.rules.iter() {
            let status = rule.check(&diff.comparison);
            global_status = global_status.and(&status);
            checks.push(RuleCheck {
                rule: *rule,
                status,
            });
        }

        let mut subsets = IndexMap::with_capacity(config.subsets.len());
        for (name, subset) in config.subsets.iter() {
            let res = SubsetCheck::evaluate(subset, &diff);
            global_status = global_status.and(&res.status);
            subsets.insert(name.to_owned(), res);
        }

        Self {
            diff,
            checks,
            subsets,
            status: global_status,
        }
    }
}

#[cfg_attr(test, derive(Debug, PartialEq))]
pub(crate) struct CheckList {
    pub status: Status,
    pub list: Vec<MetricCheck>,
}

impl CheckList {
    pub fn evaluate(config: &Config, diff: Vec<MetricDiff>) -> Self {
        let mut list = Vec::with_capacity(diff.len());
        let mut status = Status::Neutral;

        for item in diff.into_iter() {
            if let Some(config) = config.metrics.get(&item.header.name) {
                let check = MetricCheck::evaluate(config, item);
                status = status.and(&check.status);
                list.push(check);
            } else {
                list.push(MetricCheck::neutral(item));
            }
        }

        Self { status, list }
    }
}
