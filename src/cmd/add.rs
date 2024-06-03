use std::io::Write;

use super::prelude::Tag;
use crate::backend::Backend;
use crate::service::Service;

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
    ) -> Result<(), crate::service::Error> {
        let metric = crate::entity::Metric {
            header: crate::entity::MetricHeader {
                name: self.name,
                tags: self
                    .tag
                    .into_iter()
                    .map(|tag| (tag.name, tag.value))
                    .collect(),
            },
            value: self.value,
        };
        let opts = crate::service::add::Options {
            target: self.target,
        };

        Service::new(backend).add(metric, &opts)
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::backend::mock::MockBackend;

    #[test]
    fn should_add_metric_with_one_attribute() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let repo = MockBackend::default();

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

        let repo = MockBackend::default();

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
        .execute(repo.clone(), &mut stdout, &mut stderr);

        assert!(code.is_success());
        assert!(stdout.is_empty());
        assert!(stderr.is_empty());

        assert_eq!(
            repo.get_note("HEAD", crate::backend::NoteRef::Changes),
            Some(String::from(
                r#"[[changes]]
action = "add"
name = "my-metric"
value = 12.34

[changes.tags]
foo = "bar"
yolo = "pouwet"
"#
            ))
        );
    }
}
