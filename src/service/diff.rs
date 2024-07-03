use std::io::Write;

use crate::backend::{Backend, RevParse};
use crate::entity::MetricStack;

#[derive(Debug)]
pub(crate) struct Options {
    pub keep_previous: bool,
    pub remote: String,
    pub target: String,
}

fn show_diff<Out: Write>(
    keep_previous: bool,
    output: &mut Out,
    before: MetricStack,
    mut after: MetricStack,
) -> Result<(), super::Error> {
    for previous in before.into_metric_iter() {
        match after.remove_entry(&previous.header) {
            Some(next) if next.value == previous.value => {
                writeln!(output, "= {previous}")?;
            }
            Some(next) if next.value != previous.value => {
                if previous.value != 0.0 {
                    let delta = (next.value - previous.value) * 100.0 / previous.value;
                    writeln!(output, "- {previous}")?;
                    writeln!(output, "+ {next} ({delta:+.2} %)")?;
                } else {
                    writeln!(output, "- {previous}")?;
                    writeln!(output, "+ {next}")?;
                }
            }
            _ if keep_previous => {
                writeln!(output, "  {previous}")?;
            }
            _ => {}
        }
    }
    for metric in after.into_metric_iter() {
        writeln!(output, "+ {metric}")?;
    }
    Ok(())
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

    pub(crate) fn diff<Out: Write>(
        &self,
        stdout: &mut Out,
        opts: &Options,
    ) -> Result<(), super::Error> {
        let rev_parse = self.backend.rev_parse(&opts.target)?;
        let (before, after) = match rev_parse {
            RevParse::Range(ref first, _) => {
                let before = self.stack_metrics(&opts.remote, first.as_str())?;
                let after = self.stack_metrics(&opts.remote, &rev_parse.to_string())?;
                (before, after)
            }
            RevParse::Single(single) => {
                let before = self.stack_metrics(&opts.remote, &format!("{single}~1"))?;
                let after = self.get_metrics(single.as_str(), &opts.remote)?;
                (before, after)
            }
        };

        show_diff(opts.keep_previous, stdout, before, after)
    }
}

#[cfg(test)]
mod tests {
    use crate::backend::mock::MockBackend;
    use crate::backend::{NoteRef, RevParse};
    use crate::service::Service;

    #[test]
    fn should_render_diff_with_single_target_keeping_previous() {
        let mut stdout = Vec::new();
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
        Service::new(backend)
            .diff(
                &mut stdout,
                &super::Options {
                    keep_previous: true,
                    remote: "origin".into(),
                    target: "HEAD".into(),
                },
            )
            .unwrap();
        assert_eq!(
            String::from_utf8_lossy(&stdout),
            r#"- first 1.0
+ first 2.0 (+100.00 %)
  second 1.0
"#
        );
    }

    #[test]
    fn should_render_diff_with_single_target_without_previous() {
        let mut stdout = Vec::new();
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
        Service::new(backend)
            .diff(
                &mut stdout,
                &super::Options {
                    keep_previous: false,
                    remote: "origin".into(),
                    target: "HEAD".into(),
                },
            )
            .unwrap();
        assert_eq!(
            String::from_utf8_lossy(&stdout),
            r#"- first 1.0
+ first 2.0 (+100.00 %)
"#
        );
    }

    #[test]
    fn should_render_diff_with_range_target() {
        let mut stdout = Vec::new();
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
value = 0.8

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
        Service::new(backend)
            .diff(
                &mut stdout,
                &super::Options {
                    keep_previous: true,
                    remote: "origin".into(),
                    target: "HEAD~3..HEAD".into(),
                },
            )
            .unwrap();
        assert_eq!(
            String::from_utf8_lossy(&stdout),
            r#"- first 0.8
+ first 2.0 (+150.00 %)
= second 1.0
  third 0.1
"#
        );
    }
}
