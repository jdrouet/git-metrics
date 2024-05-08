/// Display the metrics related to the target
#[derive(clap::Parser, Debug, Default)]
pub(crate) struct CommandShow {
    /// Commit target, default to HEAD
    target: Option<String>,
}

impl super::Executor for CommandShow {
    fn execute<Repo: crate::repository::Repository, Out: std::io::Write, Err: std::io::Write>(
        self,
        repo: Repo,
        mut stdout: Out,
        mut stderr: Err,
    ) -> std::io::Result<()> {
        let target = self.target.as_deref().unwrap_or("HEAD");
        match repo.get_metrics(target) {
            Ok(metrics) => {
                for m in metrics.iter() {
                    writeln!(stdout, "{m:?}")?;
                }
                Ok(())
            }
            Err(err) => stderr.write_all(err.as_bytes()),
        }
    }
}
