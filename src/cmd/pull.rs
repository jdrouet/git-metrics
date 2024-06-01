use crate::backend::Backend;
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
    ) -> Result<(), super::Error> {
        backend.pull(self.remote.as_str())?;
        Ok(())
    }
}
