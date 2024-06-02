use crate::{backend::Backend, service::Service};
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
    fn execute<B: Backend, Out: Write>(
        self,
        backend: B,
        stdout: &mut Out,
    ) -> Result<(), crate::service::Error> {
        Service::new(backend).show(
            stdout,
            &crate::service::show::Options {
                target: self.target,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::entity::Metric;

    struct MockBackend {
        get_metrics_expected: &'static str,
        get_metrics_returns: Vec<Metric>,
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
    }

    #[test]
    fn should_read_head_and_return_nothing() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let repo = MockBackend {
            get_metrics_expected: "HEAD",
            get_metrics_returns: Vec::new(),
        };

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

        let repo = MockBackend {
            get_metrics_expected: sha,
            get_metrics_returns: vec![
                Metric::new("foo", 1.0),
                Metric::new("foo", 1.0).with_tag("bar", "baz"),
            ],
        };

        let code = crate::Args::parse_from(["_", "show", "--target", sha])
            .command
            .execute(repo, &mut stdout, &mut stderr);

        assert!(code.is_success());
        assert!(!stdout.is_empty());
        assert!(stderr.is_empty());

        let stdout = String::from_utf8_lossy(&stdout);
        assert_eq!(stdout, "foo 1.0\nfoo{bar=\"baz\"} 1.0\n");
    }
}
