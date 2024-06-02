use crate::{backend::Backend, service::Service};
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
    #[tracing::instrument(name = "remove", skip_all, fields(target = self.target.as_str(), index = self.index))]
    fn execute<B: Backend, Out: Write>(
        self,
        backend: B,
        _stdout: &mut Out,
    ) -> Result<(), crate::service::Error> {
        Service::new(backend).remove(
            self.index,
            &crate::service::remove::Options {
                target: self.target,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::entity::Metric;

    struct MockBackend;

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
            assert_eq!(target, "HEAD");
            Ok(Vec::new())
        }

        fn set_metrics(
            &self,
            target: &str,
            metrics: Vec<Metric>,
        ) -> Result<(), crate::backend::Error> {
            assert_eq!(target, "HEAD");
            assert!(metrics.is_empty());
            Ok(())
        }
    }

    #[test]
    fn should_remove_metric() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let code = crate::Args::parse_from(["_", "remove", "0"])
            .command
            .execute(MockBackend, &mut stdout, &mut stderr);

        assert!(code.is_success());
        assert!(stdout.is_empty());
        assert!(stderr.is_empty());
    }
}
