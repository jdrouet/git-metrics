use crate::repository::Repository;
use std::io::Write;

fn parse_tag(input: String) -> Option<(String, String)> {
    input
        .split_once(':')
        .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
        .filter(|(key, _)| !key.is_empty())
}

/// Add a metric related to the target
#[derive(clap::Parser, Debug, Default)]
pub(crate) struct CommandAdd {
    /// Commit target, default to HEAD
    #[clap(long, short, default_value = "HEAD")]
    target: String,
    /// Name of the metric
    name: String,
    /// Tag given to the metric
    #[clap(long)]
    tag: Vec<String>,
    /// Value of the metric
    value: f64,
}

impl super::Executor for CommandAdd {
    #[tracing::instrument(name = "add", skip_all, fields(target = self.target.as_str(), name = self.name.as_str()))]
    fn execute<Repo: Repository, Out: Write, Err: Write>(
        self,
        repo: Repo,
        _stdout: &mut Out,
        _stderr: &mut Err,
    ) -> Result<(), super::Error> {
        let mut metrics = repo.get_metrics(&self.target)?;
        metrics.push(crate::entity::Metric {
            header: crate::entity::MetricHeader {
                name: self.name,
                tags: self.tag.into_iter().filter_map(parse_tag).collect(),
            },
            value: self.value,
        });
        repo.set_metrics(&self.target, metrics)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::repository::MockRepository;

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
                    && metrics[0].header.name == "my-metric"
                    && metrics[0].header.tags.len() == 1
                    && metrics[0].header.tags["foo"] == "bar"
                    && metrics[0].value == 12.34
            })
            .return_once(|_, _| Ok(()));

        let code = crate::Args::parse_from(["_", "add", "my-metric", "--tag", "foo: bar", "12.34"])
            .command
            .execute(repo, &mut stdout, &mut stderr);

        assert!(code.is_success());
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
                    && metrics[0].header.name == "my-metric"
                    && metrics[0].header.tags.len() == 2
                    && metrics[0].header.tags["foo"] == "bar"
                    && metrics[0].header.tags["yolo"] == "pouwet"
                    && metrics[0].value == 12.34
            })
            .return_once(|_, _| Ok(()));

        let code = crate::Args::parse_from([
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
        .execute(repo, &mut stdout, &mut stderr);

        assert!(code.is_success());
        assert!(stdout.is_empty());
        assert!(stderr.is_empty());
    }
}
