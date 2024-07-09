use std::io::Write;

use crate::backend::Backend;
use crate::entity::difference::MetricDiffList;
use crate::service::Service;

mod format;

#[derive(clap::ValueEnum, Clone, Copy, Debug, Default)]
pub(crate) enum Format {
    #[default]
    Text,
}

/// Show metrics changes
#[derive(clap::Parser, Debug, Default)]
pub(crate) struct CommandDiff {
    /// When enabled, the metrics prior the provided range will be displayed
    #[clap(long)]
    keep_previous: bool,

    /// Output format
    #[clap(long, default_value = "text")]
    format: Format,

    /// Commit range, default to HEAD
    ///
    /// Can use ranges like HEAD~2..HEAD
    #[clap(default_value = "HEAD")]
    target: String,
}

impl CommandDiff {
    fn display<Out: Write>(&self, list: &MetricDiffList, stdout: &mut Out) -> std::io::Result<()> {
        match self.format {
            Format::Text => format::TextFormatter::format(list, stdout),
        }
    }
}

impl super::Executor for CommandDiff {
    #[tracing::instrument(name = "diff", skip_all, fields(target = self.target.as_str()))]
    fn execute<B: Backend, Out: Write>(
        self,
        backend: B,
        stdout: &mut Out,
    ) -> Result<(), crate::service::Error> {
        let opts = crate::service::diff::Options {
            remote: "origin",
            target: self.target.as_str(),
        };
        let diff = Service::new(backend).diff(&opts)?;
        let diff = if self.keep_previous {
            diff
        } else {
            diff.remove_missing()
        };
        self.display(&diff, stdout)?;
        Ok(())
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
