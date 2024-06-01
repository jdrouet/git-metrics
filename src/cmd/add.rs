use std::io::Write;

use super::prelude::Tag;
use crate::backend::Backend;

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
    tag: Vec<Tag>,
    /// Value of the metric
    value: f64,
}

impl super::Executor for CommandAdd {
    #[tracing::instrument(name = "add", skip_all, fields(target = self.target.as_str(), name = self.name.as_str()))]
    fn execute<B: Backend, Out: Write>(
        self,
        backend: B,
        _stdout: &mut Out,
    ) -> Result<(), super::Error> {
        let mut metrics = backend.get_metrics(&self.target)?;
        metrics.push(crate::entity::Metric {
            header: crate::entity::MetricHeader {
                name: self.name,
                tags: self
                    .tag
                    .into_iter()
                    .map(|tag| (tag.name, tag.value))
                    .collect(),
            },
            value: self.value,
        });
        backend.set_metrics(&self.target, metrics)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::backend::MockBackend;

    #[test]
    fn should_add_metric_with_one_attribute() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let mut repo = MockBackend::new();
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

        let mut repo = MockBackend::new();
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
