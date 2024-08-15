use super::prelude::PrettyWriter;
use crate::backend::Backend;
use crate::service::Service;
use crate::ExitCode;

mod format;

/// Show metrics changes
#[derive(clap::Parser, Debug, Default)]
pub struct CommandDiff {
    /// Remote name, default to origin
    #[clap(long, default_value = "origin")]
    remote: String,
    /// When enabled, the metrics prior the provided range will be displayed
    #[clap(long)]
    show_previous: bool,

    /// Output format
    #[clap(long, default_value = "text")]
    format: super::format::Format,

    /// Commit range, default to HEAD
    ///
    /// Can use ranges like HEAD~2..HEAD
    #[clap(default_value = "HEAD")]
    target: String,
}

impl super::Executor for CommandDiff {
    #[tracing::instrument(name = "diff", skip_all, fields(target = self.target.as_str()))]
    fn execute<B: Backend, Out: PrettyWriter>(
        self,
        backend: B,
        stdout: &mut Out,
    ) -> Result<ExitCode, crate::service::Error> {
        let root = backend.root_path()?;
        let config = crate::entity::config::Config::from_root_path(&root)?;
        let opts = crate::service::diff::Options {
            remote: self.remote.as_str(),
            target: self.target.as_str(),
        };
        let diff = Service::new(backend).diff(&opts)?;
        let diff = if self.show_previous {
            diff
        } else {
            diff.remove_missing()
        };
        match self.format {
            super::format::Format::Text => format::TextFormatter {
                show_previous: self.show_previous,
            }
            .format(&diff, &config, stdout),
        }?;
        Ok(ExitCode::Success)
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
