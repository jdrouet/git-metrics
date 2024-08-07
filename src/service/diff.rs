use crate::backend::{Backend, RevParse};
use crate::entity::difference::MetricDiffList;
use crate::entity::metric::MetricStack;

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
    use crate::entity::difference::{Comparison, Delta};
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
