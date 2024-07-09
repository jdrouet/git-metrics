use crate::{
    backend::Backend,
    config::{Config, MetricConfig, Rule, SubsetConfig},
};
use indexmap::IndexMap;

use super::diff::{Comparison, Delta, MetricDiff};

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
pub(crate) struct SubsetCheck {
    pub matching: IndexMap<String, String>,
    pub checks: Vec<(Rule, Status)>,
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
        self.checks.push((rule, status));
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
                checks.push((*rule, res));
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
    pub checks: Vec<(Rule, Status)>,
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
        self.checks.push((rule, status));
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
            checks.push((*rule, status));
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
    fn evaluate(config: &Config, diff: Vec<MetricDiff>) -> Self {
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

#[derive(Debug)]
pub(crate) struct Options<'a> {
    pub remote: &'a str,
    pub target: &'a str,
}

impl<B: Backend> super::Service<B> {
    fn open_config(&self) -> Result<crate::config::Config, super::Error> {
        let root = self.backend.root_path()?;
        let config_path = root.join(".git-metrics.toml");
        let file = if config_path.is_file() {
            crate::config::Config::from_path(&config_path)?
        } else {
            Default::default()
        };
        Ok(file)
    }

    pub(crate) fn check(&self, opts: &Options) -> Result<CheckList, super::Error> {
        let diff = self
            .diff(&super::diff::Options {
                remote: opts.remote,
                target: opts.target,
            })?
            .remove_missing()
            .into_inner();

        let config = self.open_config()?;
        Ok(CheckList::evaluate(&config, diff))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::mock::MockBackend;
    use crate::backend::{NoteRef, RevParse};
    use crate::entity::MetricHeader;
    use crate::service::Service;

    #[test]
    fn should_success() {
        let backend = MockBackend::default();
        backend.set_config(
            r#"[[metrics.first.rules]]
type = "max"
value = 100.0

[[metrics.first.rules]]
type = "max-increase"
ratio = 0.1
"#,
        );
        backend.set_rev_parse(
            "main..HEAD",
            RevParse::Range("aaaaaab".into(), "aaaaaaa".into()),
        );
        backend.set_rev_list("aaaaaab", ["aaaaaac", "aaaaaad"]);
        backend.set_rev_list("aaaaaab..aaaaaaa", ["aaaaaaa"]);
        backend.set_note(
            "aaaaaaa",
            NoteRef::remote_metrics("origin"),
            r#"[[metrics]]
name = "first"
tags = {}
value = 120.0
"#,
        );
        backend.set_note(
            "aaaaaac",
            NoteRef::remote_metrics("origin"),
            r#"[[metrics]]
name = "first"
tags = {}
value = 80.0
"#,
        );
        let res = Service::new(backend)
            .check(&super::Options {
                remote: "origin",
                target: "main..HEAD",
            })
            .unwrap();
        similar_asserts::assert_eq!(
            res,
            CheckList {
                status: Status::Failed,
                list: vec![MetricCheck::new(
                    MetricDiff {
                        header: MetricHeader::new("first"),
                        comparison: Comparison::matching(80.0, 120.0),
                    },
                    Status::Failed
                )
                .with_check(Rule::Max { value: 100.0 }, Status::Failed)
                .with_check(Rule::MaxIncrease { ratio: 0.1 }, Status::Failed)]
            }
        );
    }

    #[test]
    fn should_success_with_subsets() {
        let backend = MockBackend::default();
        backend.set_config(
            r#"[[metrics.first.rules]]
type = "max"
value = 100.0

[metrics.first.subsets.foo]
matching = { foo = "bar" }

[[metrics.first.subsets.foo.rules]]
type = "max-increase"
ratio = 0.1
"#,
        );
        backend.set_rev_parse(
            "main..HEAD",
            RevParse::Range("aaaaaab".into(), "aaaaaaa".into()),
        );
        backend.set_rev_list("aaaaaab", ["aaaaaac", "aaaaaad"]);
        backend.set_rev_list("aaaaaab..aaaaaaa", ["aaaaaaa"]);
        backend.set_note(
            "aaaaaaa",
            NoteRef::remote_metrics("origin"),
            r#"[[metrics]]
name = "first"
tags = {}
value = 90.0

[[metrics]]
name = "first"
tags = { foo = "bar" }
value = 90.0
"#,
        );
        backend.set_note(
            "aaaaaac",
            NoteRef::remote_metrics("origin"),
            r#"[[metrics]]
name = "first"
tags = {}
value = 50.0

[[metrics]]
name = "first"
tags = { foo = "bar" }
value = 50.0
"#,
        );
        let res = Service::new(backend)
            .check(&super::Options {
                remote: "origin",
                target: "main..HEAD",
            })
            .unwrap();
        similar_asserts::assert_eq!(
            res,
            CheckList {
                status: Status::Failed,
                list: vec![
                    MetricCheck::new(
                        MetricDiff {
                            header: MetricHeader::new("first"),
                            comparison: Comparison::matching(50.0, 90.0),
                        },
                        Status::Success
                    )
                    .with_check(Rule::Max { value: 100.0 }, Status::Success)
                    .with_subset(
                        "foo",
                        SubsetCheck::new(Status::Neutral).with_matching("foo", "bar")
                    ),
                    MetricCheck::new(
                        MetricDiff {
                            header: MetricHeader::new("first").with_tag("foo", "bar"),
                            comparison: Comparison::matching(50.0, 90.0),
                        },
                        Status::Failed
                    )
                    .with_check(Rule::Max { value: 100.0 }, Status::Success)
                    .with_subset(
                        "foo",
                        SubsetCheck::new(Status::Failed)
                            .with_matching("foo", "bar")
                            .with_check(Rule::MaxIncrease { ratio: 0.1 }, Status::Failed)
                    ),
                ]
            }
        );
    }
}
