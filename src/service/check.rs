use std::io::Write;

use crate::backend::{Backend, RevParse};

#[derive(Debug)]
pub(crate) struct Options {
    pub remote: String,
    pub target: String,
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

    pub(crate) fn check<Out: Write>(
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

        let mut failed_metrics: usize = 0;
        let mut success_metrics: usize = 0;

        let config = self.open_config()?;
        let mut before = before.into_inner();
        for (header, current) in after.into_inner().into_iter() {
            let previous = before.swap_remove(&header);
            let failed = config.check(&header, previous, current);
            if failed.is_empty() {
                writeln!(stdout, "[SUCCESS] {header}")?;
                success_metrics += 1;
            } else {
                writeln!(stdout, "[FAILURE] {header} ({} errors)", failed.len())?;
                for error in failed {
                    writeln!(stdout, "\t- {error}")?;
                }
                failed_metrics += 1;
            }
        }

        if failed_metrics > 0 {
            Err(super::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "{failed_metrics} metrics failed and {success_metrics} metrics are successful"
                ),
            )))
        } else {
            Ok(())
        }
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
            .check(
                &mut stdout,
                &super::Options {
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
            .check(
                &mut stdout,
                &super::Options {
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
            .check(
                &mut stdout,
                &super::Options {
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
