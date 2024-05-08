/// Remove a metric related to the target
#[derive(clap::Parser, Debug, Default)]
pub(crate) struct CommandRemove {
    /// Index of the metric to remove
    index: usize,
}

impl super::Executor for CommandRemove {
    fn execute<Repo: crate::repository::Repository, Out: std::io::Write, Err: std::io::Write>(
        self,
        repo: Repo,
        _stdout: Out,
        mut stderr: Err,
    ) -> std::io::Result<()> {
        let mut metrics = match repo.get_metrics("HEAD") {
            Ok(inner) => inner,
            Err(err) => return stderr.write_all(err.as_bytes()),
        };
        if self.index < metrics.len() {
            metrics.remove(self.index);
        }
        if let Err(err) = repo.set_metrics("HEAD", metrics) {
            return stderr.write_all(err.as_bytes());
        }
        Ok(())
    }
}
