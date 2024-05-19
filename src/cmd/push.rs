use crate::repository::Repository;
use std::io::Write;

/// Pushes the metrics
#[derive(clap::Parser, Debug, Default)]
pub(crate) struct CommandPush {
    /// Remote name, default to origin
    #[clap(default_value = "origin")]
    remote: String,
}

impl super::Executor for CommandPush {
    #[tracing::instrument(name = "push", skip_all, fields(remote = self.remote.as_str()))]
    fn execute<Repo: Repository, Out: Write, Err: Write>(
        self,
        repo: Repo,
        _stdout: &mut Out,
        _stderr: &mut Err,
    ) -> Result<(), super::Error> {
        repo.push(self.remote.as_str())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::{cmd::Executor, repository::MockRepository};

    #[test]
    fn should_add_metric_with_one_attribute() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let mut repo = MockRepository::new();
        repo.expect_get_metrics()
            .with(mockall::predicate::eq("HEAD"))
            .return_once(|_| Ok(Vec::new()));
        repo.expect_set_metrics()
            .withf_st(|target, metrics| {
                target == "HEAD"
                    && metrics.len() == 1
                    && metrics[0].name == "my-metric"
                    && metrics[0].tags.len() == 1
                    && metrics[0].tags["foo"] == "bar"
                    && metrics[0].value == 12.34
            })
            .return_once(|_, _| Ok(()));

        crate::Args::parse_from(["_", "add", "my-metric", "--tag", "foo: bar", "12.34"])
            .command
            .execute(repo, &mut stdout, &mut stderr)
            .unwrap();

        assert!(stdout.is_empty());
        assert!(stderr.is_empty());
    }

    #[test]
    fn should_add_metric_with_multiple_attributes() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let mut repo = MockRepository::new();
        repo.expect_get_metrics()
            .with(mockall::predicate::eq("HEAD"))
            .return_once(|_| Ok(Vec::new()));
        repo.expect_set_metrics()
            .withf_st(|target, metrics| {
                target == "HEAD"
                    && metrics.len() == 1
                    && metrics[0].name == "my-metric"
                    && metrics[0].tags.len() == 2
                    && metrics[0].tags["foo"] == "bar"
                    && metrics[0].tags["yolo"] == "pouwet"
                    && metrics[0].value == 12.34
            })
            .return_once(|_, _| Ok(()));

        crate::Args::parse_from([
            "_",
            "add",
            "my-metric",
            "--tag",
            "foo: bar",
            "--tag",
            "yolo: pouwet",
            "12.34",
        ])
        .command
        .execute(repo, &mut stdout, &mut stderr)
        .unwrap();

        assert!(stdout.is_empty());
        assert!(stderr.is_empty());
    }
}
