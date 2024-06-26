use std::io::Write;

use crate::backend::Backend;
use crate::service::Service;

/// Pulls the metrics
#[derive(clap::Parser, Debug, Default)]
pub(crate) struct CommandPull {
    /// Remote name, default to origin
    #[clap(default_value = "origin")]
    remote: String,
}

impl super::Executor for CommandPull {
    #[tracing::instrument(name = "pull", skip_all, fields(remote = self.remote.as_str()))]
    fn execute<B: Backend, Out: Write>(
        self,
        backend: B,
        _stdout: &mut Out,
    ) -> Result<(), crate::service::Error> {
        Service::new(backend).pull(&crate::service::pull::Options {
            remote: self.remote,
        })
    }
}
