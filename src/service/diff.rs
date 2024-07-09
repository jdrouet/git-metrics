use crate::backend::{Backend, RevParse};
use crate::entity::{MetricHeader, MetricStack};

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

#[derive(Debug)]
pub(crate) struct Options<'a> {
    pub remote: &'a str,
    pub target: &'a str,
}

impl<B: Backend> super::Service<B> {
    pub(super) fn stack_metrics(
        &self,
        remote_name: &str,
        range: &str,
    ) -> Result<MetricStack, super::Error> {
        let mut stack = MetricStack::default();
        let mut commits = self.backend.rev_list(range)?;
        commits.reverse();
        for commit_sha in commits {
            let metrics = self.get_metrics(commit_sha.as_str(), remote_name)?;
            stack.extend(metrics);
        }
        Ok(stack)
    }

    pub(crate) fn diff(&self, opts: &Options<'_>) -> Result<MetricDiffList, super::Error> {
        let rev_parse = self.backend.rev_parse(opts.target)?;
        let (before, after) = match rev_parse {
            RevParse::Range(ref first, _) => {
                let before = self.stack_metrics(opts.remote, first.as_str())?;
                let after = self.stack_metrics(opts.remote, &rev_parse.to_string())?;
                (before, after)
            }
            RevParse::Single(single) => {
                let before = self.stack_metrics(opts.remote, &format!("{single}~1"))?;
                let after = self.get_metrics(single.as_str(), opts.remote)?;
                (before, after)
            }
        };

        Ok(MetricDiffList::new(before, after))
    }
}

#[cfg(test)]
mod tests {
    use crate::backend::mock::MockBackend;
    use crate::backend::{NoteRef, RevParse};
    use crate::service::diff::{Comparison, Delta};
    use crate::service::Service;

    #[test]
    fn should_render_diff_with_single_target_keeping_previous() {
        let backend = MockBackend::default();
        backend.set_rev_parse("HEAD", RevParse::Single("aaaaaaa".into()));
        backend.set_rev_list("aaaaaaa~1", ["aaaaaab", "aaaaaac", "aaaaaad", "aaaaaae"]);
        backend.set_note(
            "aaaaaaa",
            NoteRef::remote_metrics("origin"),
            r#"[[metrics]]
name = "first"
tags = {}
value = 2.0
"#,
        );
        backend.set_note(
            "aaaaaac",
            NoteRef::remote_metrics("origin"),
            r#"[[metrics]]
name = "first"
tags = {}
value = 1.0

[[metrics]]
name = "second"
tags = {}
value = 1.0
"#,
        );
        let list = Service::new(backend)
            .diff(&super::Options {
                remote: "origin",
                target: "HEAD",
            })
            .unwrap();
        assert_eq!(list.0.len(), 2);
        assert_eq!(list.0[0].header.name, "first");
        assert_eq!(
            list.0[0].comparison,
            Comparison::Matching {
                previous: 1.0,
                current: 2.0,
                delta: Delta {
                    absolute: 1.0,
                    relative: Some(1.0),
                },
            }
        );
        assert_eq!(list.0[1].header.name, "second");
        assert_eq!(list.0[1].comparison, Comparison::Missing { previous: 1.0 });
        let list = list.remove_missing();
        assert_eq!(list.inner().len(), 1);
    }

    #[test]
    fn should_render_diff_with_range_target() {
        let backend = MockBackend::default();
        backend.set_rev_parse(
            "HEAD~3..HEAD",
            RevParse::Range("aaaaaad".into(), "aaaaaaa".into()),
        );
        backend.set_rev_list("aaaaaad", ["aaaaaad", "aaaaaae", "aaaaaaf"]);
        backend.set_rev_list("aaaaaad..aaaaaaa", ["aaaaaaa", "aaaaaab", "aaaaaac"]);
        backend.set_note(
            "aaaaaaa",
            NoteRef::remote_metrics("origin"),
            r#"[[metrics]]
name = "first"
tags = {}
value = 2.0
"#,
        );
        backend.set_note(
            "aaaaaac",
            NoteRef::remote_metrics("origin"),
            r#"[[metrics]]
name = "first"
tags = {}
value = 1.0

[[metrics]]
name = "second"
tags = {}
value = 1.0
"#,
        );
        backend.set_note(
            "aaaaaae",
            NoteRef::remote_metrics("origin"),
            r#"[[metrics]]
name = "first"
tags = {}
value = 0.5

[[metrics]]
name = "second"
tags = {}
value = 1.0

[[metrics]]
name = "third"
tags = {}
value = 0.1
"#,
        );
        let list = Service::new(backend)
            .diff(&super::Options {
                remote: "origin",
                target: "HEAD~3..HEAD",
            })
            .unwrap();
        assert_eq!(list.0.len(), 3);
        assert_eq!(list.0[0].header.name, "first");
        assert_eq!(
            list.0[0].comparison,
            Comparison::Matching {
                previous: 0.5,
                current: 2.0,
                delta: Delta {
                    absolute: 1.5,
                    relative: Some(3.0),
                },
            }
        );
        assert_eq!(list.0[1].header.name, "second");
        assert_eq!(
            list.0[1].comparison,
            Comparison::Matching {
                previous: 1.0,
                current: 1.0,
                delta: Delta {
                    absolute: 0.0,
                    relative: Some(0.0),
                },
            }
        );
        assert_eq!(list.0[2].header.name, "third");
        assert_eq!(
            list.0[2].comparison,
            Comparison::Missing { previous: 0.1 },
            "{:?}",
            list.0[2].comparison
        );
    }
}
