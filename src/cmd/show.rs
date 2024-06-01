use crate::backend::Backend;
use std::io::Write;

/// Display the metrics related to the target
#[derive(clap::Parser, Debug, Default)]
pub(crate) struct CommandShow {
    /// Commit target, default to HEAD
    #[clap(long, short, default_value = "HEAD")]
    target: String,
}

impl super::Executor for CommandShow {
    #[tracing::instrument(name = "show", skip_all, fields(target = self.target.as_str()))]
    fn execute<Repo: Backend, Out: Write>(
        self,
        repo: Repo,
        stdout: &mut Out,
    ) -> Result<(), super::Error> {
        let metrics = repo.get_metrics(&self.target)?;
        for m in metrics.iter() {
            writeln!(stdout, "{m}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::backend::MockBackend;
    use crate::entity::Metric;

    #[test]
    fn should_read_head_and_return_nothing() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let mut repo = MockBackend::new();
        repo.expect_get_metrics()
            .with(mockall::predicate::eq("HEAD"))
            .return_once(|_| Ok(Vec::new()));

        let code =
            crate::Args::parse_from(["_", "show"])
                .command
                .execute(repo, &mut stdout, &mut stderr);

        assert!(code.is_success());
        assert!(stdout.is_empty());
        assert!(stderr.is_empty());
    }

    #[test]
    fn should_read_hash_and_return_a_list() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let sha = "aaaaaaa";

        let mut repo = MockBackend::new();
        repo.expect_get_metrics()
            .with(mockall::predicate::eq(sha))
            .return_once(|_| {
                Ok(vec![
                    Metric::new("foo", 1.0),
                    Metric::new("foo", 1.0).with_tag("bar", "baz"),
                ])
            });

        let code = crate::Args::parse_from(["_", "show", "--target", sha])
            .command
            .execute(repo, &mut stdout, &mut stderr);

        assert!(code.is_success());
        assert!(!stdout.is_empty());
        assert!(stderr.is_empty());

        let stdout = String::from_utf8_lossy(&stdout);
        assert_eq!(stdout, "foo{} = 1.0\nfoo{bar=\"baz\"} = 1.0\n");
    }
}
