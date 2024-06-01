use crate::backend::Backend;
use std::io::Write;

/// Add a metric related to the target
#[derive(clap::Parser, Debug, Default)]
pub(crate) struct CommandLog {
    /// Commit range, default to HEAD
    ///
    /// Can use ranges like HEAD~2..HEAD
    #[clap(default_value = "HEAD")]
    target: String,

    /// If enabled, the empty commits will not be displayed
    #[clap(long)]
    filter_empty: bool,
}

impl super::Executor for CommandLog {
    fn execute<B: Backend, Out: Write>(
        self,
        backend: B,
        stdout: &mut Out,
    ) -> Result<(), super::Error> {
        let commits = backend.get_commits(&self.target)?;
        for commit in commits.iter() {
            let metrics = backend.get_remote_metrics(commit.sha.as_str())?;
            if self.filter_empty && metrics.is_empty() {
                continue;
            }
            writeln!(stdout, "* {} {}", &commit.sha.as_str()[..7], commit.summary)?;
            for metric in metrics {
                writeln!(stdout, "\t{metric}")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::CommandLog;

    #[test]
    fn should_parse_range() {
        let cmd = CommandLog::parse_from(["_", "HEAD~4..HEAD"]);
        assert_eq!(cmd.target, "HEAD~4..HEAD");
    }
}
