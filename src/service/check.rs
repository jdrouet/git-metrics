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
    fn should_success() {
        let mut stdout = Vec::new();
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
        let _err = Service::new(backend)
            .check(
                &mut stdout,
                &super::Options {
                    remote: "origin".into(),
                    target: "main..HEAD".into(),
                },
            )
            .unwrap_err();
        assert_eq!(
            String::from_utf8_lossy(&stdout),
            "[FAILURE] first (2 errors)\n\t- 120 is greater than the max allowed 100\n\t- increased of 50.0%, with a limit at 10.0%\n"
        );
    }

    #[test]
    fn should_success_with_subsets() {
        let mut stdout = Vec::new();
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
        let _err = Service::new(backend)
            .check(
                &mut stdout,
                &super::Options {
                    remote: "origin".into(),
                    target: "main..HEAD".into(),
                },
            )
            .unwrap_err();
        assert_eq!(String::from_utf8_lossy(&stdout), "[SUCCESS] first\n[FAILURE] first{foo=\"bar\"} (1 errors)\n\t- increased of 80.0%, with a limit at 10.0%\n");
    }
}
