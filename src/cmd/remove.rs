use crate::repository::Repository;
use std::io::Write;

/// Remove a metric related to the target
#[derive(clap::Parser, Debug, Default)]
pub(crate) struct CommandRemove {
    /// Commit target, default to HEAD
    #[clap(long, short, default_value = "HEAD")]
    target: String,
    /// Index of the metric to remove
    index: usize,
}

impl super::Executor for CommandRemove {
    fn execute<Repo: Repository, Out: Write, Err: Write>(
        self,
        repo: Repo,
        _stdout: &mut Out,
        _stderr: &mut Err,
    ) -> Result<(), super::Error> {
        let mut metrics = repo.get_metrics(&self.target)?;
        if self.index < metrics.len() {
            metrics.remove(self.index);
        }
        repo.set_metrics(&self.target, metrics)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::{cmd::Executor, metric::Metric, repository::MockRepository};

    #[test]
    fn should_remove_metric() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let mut repo = MockRepository::new();
        repo.expect_get_metrics()
            .with(mockall::predicate::eq("HEAD"))
            .return_once(|_| Ok(Vec::new()));
        repo.expect_set_metrics()
            .with(
                mockall::predicate::eq("HEAD"),
                mockall::predicate::function(|v: &Vec<Metric>| v.is_empty()),
            )
            .return_once(|_, _| Ok(()));

        crate::Args::parse_from(&["_", "remove", "0"])
            .command
            .execute(repo, &mut stdout, &mut stderr)
            .unwrap();

        assert!(stdout.is_empty());
        assert!(stderr.is_empty());
    }
}
