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

    use crate::entity::Metric;

    struct MockBackend {
        get_metrics_expected: &'static str,
        get_metrics_returns: Vec<Metric>,
        set_metrics_expected_target: &'static str,
        set_metrics_expected_values: Vec<&'static str>,
    }

    impl crate::backend::Backend for MockBackend {
        fn pull(&self, _remote: &str) -> Result<(), crate::backend::Error> {
            todo!()
        }
        fn push(&self, _remote: &str) -> Result<(), crate::backend::Error> {
            todo!()
        }
        fn read_note<T: serde::de::DeserializeOwned>(
            &self,
            _target: &str,
            _note_ref: &str,
        ) -> Result<Option<T>, crate::backend::Error> {
            todo!()
        }
        fn write_note<T: serde::Serialize>(
            &self,
            _target: &str,
            _note_ref: &str,
            _value: &T,
        ) -> Result<(), crate::backend::Error> {
            todo!()
        }
        fn get_commits(
            &self,
            _range: &str,
        ) -> Result<Vec<crate::entity::Commit>, crate::backend::Error> {
            todo!()
        }

        fn get_metrics(&self, target: &str) -> Result<Vec<Metric>, crate::backend::Error> {
            assert_eq!(self.get_metrics_expected, target);
            Ok(self.get_metrics_returns.clone())
        }
        fn set_metrics(
            &self,
            target: &str,
            metrics: Vec<Metric>,
        ) -> Result<(), crate::backend::Error> {
            assert_eq!(self.set_metrics_expected_target, target);
            assert_eq!(self.set_metrics_expected_values.len(), metrics.len());
            self.set_metrics_expected_values
                .iter()
                .zip(metrics.iter().map(|v| v.to_string()))
                .for_each(|(left, right)| assert_eq!(left, &right));
            Ok(())
        }
    }

    #[test]
    fn should_add_metric_with_one_attribute() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let repo = MockBackend {
            get_metrics_expected: "HEAD",
            get_metrics_returns: Vec::new(),
            set_metrics_expected_target: "HEAD",
            set_metrics_expected_values: vec!["my-metric{foo=\"bar\"} 12.34"],
        };

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

        let repo = MockBackend {
            get_metrics_expected: "HEAD",
            get_metrics_returns: Vec::new(),
            set_metrics_expected_target: "HEAD",
            set_metrics_expected_values: vec!["my-metric{foo=\"bar\", yolo=\"pouwet\"} 12.34"],
        };

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
