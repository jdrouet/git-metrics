use super::prelude::PrettyWriter;
use crate::backend::Backend;
use crate::service::Service;
use crate::ExitCode;

/// Pushes the metrics
#[derive(clap::Parser, Debug, Default)]
pub struct CommandPush {
    /// Remote name, default to origin
    #[clap(default_value = "origin")]
    remote: String,
}

impl super::Executor for CommandPush {
    #[tracing::instrument(name = "push", skip_all, fields(remote = self.remote.as_str()))]
    fn execute<B: Backend, Out: PrettyWriter>(
        self,
        backend: B,
        _stdout: Out,
    ) -> Result<ExitCode, crate::service::Error> {
        Service::new(backend).push(&crate::service::push::Options {
            remote: self.remote.as_str(),
        })?;
        Ok(ExitCode::Success)
    }
}
