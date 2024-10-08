use crate::backend::Backend;
use crate::entity::check::CheckList;
use crate::entity::config::Config;

#[derive(Debug)]
pub(crate) struct Options<'a> {
    pub remote: &'a str,
    pub target: &'a str,
}

impl<B: Backend> super::Service<B> {
    pub(crate) fn check(&self, config: &Config, opts: &Options) -> Result<CheckList, super::Error> {
        let diff = self
            .diff(&super::diff::Options {
                remote: opts.remote,
                target: opts.target,
            })?
            .remove_missing()
            .into_inner();

        Ok(CheckList::evaluate(config, diff))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::mock::MockBackend;
    use crate::backend::{NoteRef, RevParse};
    use crate::entity::check::{MetricCheck, Status, SubsetCheck};
    use crate::entity::config::Rule;
    use crate::entity::difference::{Comparison, MetricDiff};
    use crate::entity::metric::MetricHeader;
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
        let config = backend.get_config();
        let res = Service::new(backend)
            .check(
                &config,
                &super::Options {
                    remote: "origin",
                    target: "main..HEAD",
                },
            )
            .unwrap();
        similar_asserts::assert_eq!(
            res,
            CheckList::default().with_check(
                MetricCheck::new(MetricDiff::new(
                    MetricHeader::new("first"),
                    Comparison::matching(80.0, 120.0)
                ))
                .with_check(Rule::max(100.0), Status::Failed)
                .with_check(Rule::max_relative_increase(0.1), Status::Failed)
            )
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
        let config = backend.get_config();
        let res = Service::new(backend)
            .check(
                &config,
                &super::Options {
                    remote: "origin",
                    target: "main..HEAD",
                },
            )
            .unwrap();
        similar_asserts::assert_eq!(
            res,
            CheckList::default()
                .with_check(
                    MetricCheck::new(MetricDiff::new(
                        MetricHeader::new("first"),
                        Comparison::matching(50.0, 90.0)
                    ))
                    .with_check(Rule::max(100.0), Status::Success)
                    .with_subset("foo", SubsetCheck::default().with_matching("foo", "bar"))
                )
                .with_check(
                    MetricCheck::new(MetricDiff::new(
                        MetricHeader::new("first").with_tag("foo", "bar"),
                        Comparison::matching(50.0, 90.0)
                    ))
                    .with_check(Rule::max(100.0), Status::Success)
                    .with_subset(
                        "foo",
                        SubsetCheck::default()
                            .with_matching("foo", "bar")
                            .with_check(Rule::max_relative_increase(0.1), Status::Failed)
                    )
                )
        );
    }
}
