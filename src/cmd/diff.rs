use std::io::Write;

use crate::backend::Backend;
use crate::service::Service;

/// Show metrics changes
#[derive(clap::Parser, Debug, Default)]
pub(crate) struct CommandDiff {
    /// When enabled, the metrics prior the provided range will be displayed
    #[clap(long)]
    keep_previous: bool,
    /// Commit range, default to HEAD
    ///
    /// Can use ranges like HEAD~2..HEAD
    #[clap(default_value = "HEAD")]
    target: String,
}

impl super::Executor for CommandDiff {
    #[tracing::instrument(name = "diff", skip_all, fields(target = self.target.as_str()))]
    fn execute<B: Backend, Out: Write>(
        self,
        backend: B,
        stdout: &mut Out,
    ) -> Result<(), crate::service::Error> {
        let opts = crate::service::diff::Options {
            keep_previous: self.keep_previous,
            remote: String::from("origin"),
            target: self.target,
        };
        Service::new(backend).diff(stdout, &opts)
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::CommandDiff;

    #[test]
    fn should_parse_range() {
        let cmd = CommandDiff::parse_from(["_", "HEAD~4..HEAD"]);
        assert_eq!(cmd.target, "HEAD~4..HEAD");
    }
}
