use indexmap::IndexMap;

use super::config::{
    Config, MetricConfig, Rule, RuleAbsolute, RuleChange, RuleRelative, SubsetConfig,
};
use super::difference::{Comparison, Delta, MetricDiff};

#[derive(Clone, Copy)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub(crate) enum Status {
    Success,
    Skip,
    Failed,
}

#[cfg(test)]
impl Default for Status {
    fn default() -> Self {
        Self::Skip
    }
}

#[derive(Default)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub(crate) struct StatusCount {
    pub success: usize,
    pub neutral: usize,
    pub failed: usize,
}

impl StatusCount {
    pub fn push(&mut self, status: &Status) {
        match status {
            Status::Success => self.success += 1,
            Status::Skip => self.neutral += 1,
            Status::Failed => self.failed += 1,
        }
    }

    pub fn extend(&mut self, other: &StatusCount) {
        self.success += other.success;
        self.neutral += other.neutral;
        self.failed += other.failed;
    }

    pub fn is_failed(&self) -> bool {
        self.failed > 0
    }

    pub fn status(&self) -> Status {
        if self.failed > 0 {
            Status::Failed
        } else if self.success > 0 {
            Status::Success
        } else {
            Status::Skip
        }
    }
}

impl Rule {
    fn check(&self, comparison: &Comparison) -> Status {
        match self {
            Self::Max(RuleAbsolute { value }) => match comparison {
                Comparison::Created { current } | Comparison::Matching { current, .. }
                    if current > value =>
                {
                    Status::Failed
                }
                Comparison::Missing { .. } => Status::Skip,
                _ => Status::Success,
            },
            Self::Min(RuleAbsolute { value }) => match comparison {
                Comparison::Created { current } | Comparison::Matching { current, .. }
                    if current < value =>
                {
                    Status::Failed
                }
                Comparison::Missing { .. } => Status::Skip,
                _ => Status::Success,
            },
            Self::MaxIncrease(RuleChange::Absolute(RuleAbsolute { value })) => match comparison {
                Comparison::Matching {
                    delta: Delta { absolute, .. },
                    ..
                } if absolute > value => Status::Failed,
                _ => Status::Success,
            },
            Self::MaxIncrease(RuleChange::Relative(RuleRelative { ratio })) => match comparison {
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
                _ => Status::Skip,
            },
            Self::MaxDecrease(RuleChange::Relative(RuleRelative { ratio })) => match comparison {
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
                _ => Status::Skip,
            },
            Self::MaxDecrease(RuleChange::Absolute(RuleAbsolute { value })) => match comparison {
                Comparison::Matching {
                    delta: Delta { absolute, .. },
                    ..
                } if absolute < value => Status::Failed,
                _ => Status::Success,
            },
        }
    }
}

#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct RuleCheck {
    pub rule: Rule,
    pub status: Status,
}

#[cfg_attr(test, derive(Debug, Default, PartialEq))]
pub(crate) struct SubsetCheck {
    pub matching: IndexMap<String, String>,
    pub checks: Vec<RuleCheck>,
    pub status: StatusCount,
}

#[cfg(test)]
impl SubsetCheck {
    pub fn with_matching<N: Into<String>, V: Into<String>>(mut self, name: N, value: V) -> Self {
        self.matching.insert(name.into(), value.into());
        self
    }

    pub fn with_check(mut self, rule: Rule, status: Status) -> Self {
        self.status.push(&status);
        self.checks.push(RuleCheck { rule, status });
        self
    }
}

impl SubsetCheck {
    fn evaluate(config: &SubsetConfig, diff: &MetricDiff) -> Self {
        let mut status = StatusCount::default();
        let mut checks = Vec::with_capacity(config.rules.len());
        if config.matches(&diff.header) {
            for rule in config.rules.iter() {
                let res = rule.check(&diff.comparison);
                status.push(&res);
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
    pub checks: Vec<RuleCheck>,
    pub subsets: IndexMap<String, SubsetCheck>,
    pub status: StatusCount,
}

#[cfg(test)]
impl MetricCheck {
    pub fn new(diff: MetricDiff) -> Self {
        Self {
            diff,
            checks: Default::default(),
            subsets: Default::default(),
            status: StatusCount::default(),
        }
    }

    pub fn with_check(mut self, rule: Rule, status: Status) -> Self {
        self.status.push(&status);
        self.checks.push(RuleCheck { rule, status });
        self
    }

    pub fn with_subset<N: Into<String>>(mut self, name: N, subset: SubsetCheck) -> Self {
        self.status.extend(&subset.status);
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
            status: StatusCount::default(),
        }
    }

    fn evaluate(config: &MetricConfig, diff: MetricDiff) -> Self {
        let mut global_status = StatusCount::default();

        let mut checks = Vec::with_capacity(config.rules.len());
        for rule in config.rules.iter() {
            let status = rule.check(&diff.comparison);
            global_status.push(&status);
            checks.push(RuleCheck {
                rule: *rule,
                status,
            });
        }

        let mut subsets = IndexMap::with_capacity(config.subsets.len());
        for (name, subset) in config.subsets.iter() {
            let res = SubsetCheck::evaluate(subset, &diff);
            global_status.extend(&res.status);
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

#[cfg_attr(test, derive(Debug, Default, PartialEq))]
pub(crate) struct CheckList {
    pub status: StatusCount,
    pub list: Vec<MetricCheck>,
}

#[cfg(test)]
impl CheckList {
    pub fn with_check(mut self, check: MetricCheck) -> Self {
        self.status.extend(&check.status);
        self.list.push(check);
        self
    }
}

impl CheckList {
    pub fn evaluate(config: &Config, diff: Vec<MetricDiff>) -> Self {
        let mut list = Vec::with_capacity(diff.len());
        let mut status = StatusCount::default();

        for item in diff.into_iter() {
            if let Some(config) = config.metrics.get(&item.header.name) {
                let check = MetricCheck::evaluate(config, item);
                status.extend(&check.status);
                list.push(check);
            } else {
                list.push(MetricCheck::neutral(item));
            }
        }

        Self { status, list }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_check_max() {
        let rule = Rule::max(10.0);
        assert_eq!(rule.check(&Comparison::created(20.0)), Status::Failed);
        assert_eq!(rule.check(&Comparison::created(0.0)), Status::Success);
        assert_eq!(
            rule.check(&Comparison::new(0.0, Some(20.0))),
            Status::Failed
        );
        assert_eq!(
            rule.check(&Comparison::new(0.0, Some(5.0))),
            Status::Success
        );
        assert_eq!(rule.check(&Comparison::new(0.0, None)), Status::Skip);
    }

    #[test]
    fn should_check_min() {
        let rule = Rule::min(10.0);
        assert_eq!(rule.check(&Comparison::created(20.0)), Status::Success);
        assert_eq!(rule.check(&Comparison::created(0.0)), Status::Failed);
        assert_eq!(
            rule.check(&Comparison::new(0.0, Some(20.0))),
            Status::Success
        );
        assert_eq!(rule.check(&Comparison::new(0.0, Some(5.0))), Status::Failed);
        assert_eq!(rule.check(&Comparison::new(0.0, None)), Status::Skip);
    }

    #[test]
    fn should_check_max_increase() {
        let rule = Rule::max_relative_increase(0.1);
        assert_eq!(rule.check(&Comparison::created(0.0)), Status::Skip);
        assert_eq!(rule.check(&Comparison::new(0.0, Some(20.0))), Status::Skip);
        assert_eq!(
            rule.check(&Comparison::new(10.0, Some(20.0))),
            Status::Failed
        );
        assert_eq!(
            rule.check(&Comparison::new(10.0, Some(10.5))),
            Status::Success
        );
        assert_eq!(rule.check(&Comparison::new(10.0, None)), Status::Skip);
    }

    #[test]
    fn should_check_max_decrease() {
        let rule = Rule::max_relative_decrease(0.1);
        assert_eq!(rule.check(&Comparison::created(0.0)), Status::Skip);
        assert_eq!(rule.check(&Comparison::new(0.0, Some(20.0))), Status::Skip);
        assert_eq!(
            rule.check(&Comparison::new(10.0, Some(0.0))),
            Status::Failed
        );
        assert_eq!(
            rule.check(&Comparison::new(10.0, Some(9.5))),
            Status::Success
        );
        assert_eq!(rule.check(&Comparison::new(10.0, None)), Status::Skip);
    }
}
