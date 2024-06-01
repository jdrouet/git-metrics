use crate::{backend::Backend, service::Service};
use std::io::Write;

/// Pulls the metrics
#[derive(clap::Parser, Debug, Default)]
pub(crate) struct CommandPull {
    /// Remote name, default to origin
    #[clap(default_value = "origin")]
    remote: String,
}

impl super::Executor for CommandPull {
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
