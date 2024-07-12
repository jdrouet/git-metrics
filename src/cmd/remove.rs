use super::prelude::PrettyWriter;
use crate::backend::Backend;
use crate::service::Service;
use crate::ExitCode;

/// Remove a metric related to the target
#[derive(clap::Parser, Debug, Default)]
pub struct CommandRemove {
    /// Commit target, default to HEAD
    #[clap(long, short, default_value = "HEAD")]
    target: String,
    /// Index of the metric to remove
    index: usize,
}

impl super::Executor for CommandRemove {
    #[tracing::instrument(name = "remove", skip_all, fields(target = self.target.as_str(), index = self.index))]
    fn execute<B: Backend, Out: PrettyWriter>(
        self,
        backend: B,
        _stdout: &mut Out,
    ) -> Result<ExitCode, crate::service::Error> {
        Service::new(backend).remove(
            self.index,
            &crate::service::remove::Options {
                target: self.target,
            },
        )?;
        Ok(ExitCode::Success)
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::backend::mock::MockBackend;

    #[test]
    fn should_remove_metric() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let backend = MockBackend::default();

        let code = crate::Args::parse_from(["_", "remove", "0"])
            .command
            .execute(backend, false, &mut stdout, &mut stderr);

        assert!(code.is_success());
        assert!(stdout.is_empty());
        assert!(stderr.is_empty());
    }
}
