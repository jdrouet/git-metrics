use super::format::text::TextMetric;
use super::format::undefined_unit_formatter;
use super::prelude::PrettyWriter;
use crate::backend::Backend;
use crate::entity::config::Config;
use crate::service::Service;
use crate::ExitCode;

/// Display the metrics related to the target
#[derive(clap::Parser, Debug, Default)]
pub struct CommandShow {
    /// Commit target, default to HEAD
    #[clap(long, short, default_value = "HEAD")]
    target: String,
}

impl super::Executor for CommandShow {
    #[tracing::instrument(name = "show", skip_all, fields(target = self.target.as_str()))]
    fn execute<B: Backend, Out: PrettyWriter>(
        self,
        backend: B,
        stdout: &mut Out,
    ) -> Result<ExitCode, crate::service::Error> {
        let root = backend.root_path()?;
        let config = Config::from_root_path(&root)?;
        let metrics = Service::new(backend).show(&crate::service::show::Options {
            target: self.target,
        })?;
        let default_formatter = undefined_unit_formatter();
        for metric in metrics.into_metric_iter() {
            let formatter = config
                .metrics
                .get(metric.header.name.as_str())
                .map(|m| m.unit.formater())
                .unwrap_or_else(|| default_formatter.clone());
            stdout.write_element(TextMetric::new(&formatter, &metric))?;
            stdout.write_str("\n")?;
        }
        Ok(ExitCode::Success)
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::backend::NoteRef;

    #[test]
    fn should_read_head_and_return_nothing() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let repo = crate::backend::mock::MockBackend::default();
        repo.set_note("HEAD", NoteRef::remote_metrics("origin"), String::new());

        let code = crate::Args::parse_from(["_", "show"]).command.execute(
            repo,
            false,
            &mut stdout,
            &mut stderr,
        );

        assert!(code.is_success());
        assert!(stdout.is_empty());
        assert!(stderr.is_empty());
    }

    #[test]
    fn should_read_hash_and_return_a_list() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let sha = "aaaaaaa";

        let repo = crate::backend::mock::MockBackend::default();
        repo.set_note(
            sha,
            NoteRef::remote_metrics("origin"),
            String::from(
                r#"[[metrics]]
name = "foo"
value = 1.0
"#,
            ),
        );
        repo.set_note(
            sha,
            crate::backend::NoteRef::Changes,
            String::from(
                r#"[[changes]]
action = "add"
name = "foo"
tags = { bar = "baz" }
value = 1.0
"#,
            ),
        );

        let code = crate::Args::parse_from(["_", "show", "--target", sha])
            .command
            .execute(repo, false, &mut stdout, &mut stderr);

        assert!(code.is_success(), "{:?}", String::from_utf8_lossy(&stderr));
        assert!(!stdout.is_empty());
        assert!(stderr.is_empty());

        let stdout = String::from_utf8_lossy(&stdout);
        assert_eq!(stdout, "foo 1.00\nfoo{bar=\"baz\"} 1.00\n");
    }
}
