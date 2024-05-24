use crate::repository::Repository;
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
    fn execute<Repo: Repository, Out: Write, Err: Write>(
        self,
        repo: Repo,
        stdout: &mut Out,
        _stderr: &mut Err,
    ) -> Result<(), super::Error> {
        let metrics = repo.get_metrics(&self.target)?;
        for m in metrics.iter() {
            writeln!(stdout, "{m:?}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use indexmap::IndexMap;

    use crate::{metric::Metric, repository::MockRepository};

    #[test]
    fn should_read_head_and_return_nothing() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let mut repo = MockRepository::new();
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

        let mut repo = MockRepository::new();
        repo.expect_get_metrics()
            .with(mockall::predicate::eq(sha))
            .return_once(|_| {
                Ok(vec![
                    Metric {
                        name: "foo".into(),
                        tags: Default::default(),
                        value: 1.0,
                    },
                    Metric {
                        name: "foo".into(),
                        tags: IndexMap::from_iter([(String::from("bar"), String::from("baz"))]),
                        value: 1.0,
                    },
                ])
            });

        let code = crate::Args::parse_from(["_", "show", "--target", sha])
            .command
            .execute(repo, &mut stdout, &mut stderr);

        assert!(code.is_success());
        assert!(!stdout.is_empty());
        assert!(stderr.is_empty());

        let stdout = String::from_utf8_lossy(&stdout);
        assert_eq!(
            stdout,
            r#"Metric { name: "foo", tags: {}, value: 1.0 }
Metric { name: "foo", tags: {"bar": "baz"}, value: 1.0 }
"#
        );
    }
}
