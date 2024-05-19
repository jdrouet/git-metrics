use crate::repository::Repository;
use std::io::Write;

/// Pushes the metrics
#[derive(clap::Parser, Debug, Default)]
pub(crate) struct CommandPull {
    /// Remote name, default to origin
    #[clap(default_value = "origin")]
    remote: String,
}

impl super::Executor for CommandPull {
    fn execute<Repo: Repository, Out: Write, Err: Write>(
        self,
        repo: Repo,
        _stdout: &mut Out,
        _stderr: &mut Err,
    ) -> Result<(), super::Error> {
        repo.pull(self.remote.as_str())?;
        Ok(())
    }
}
