use super::prelude::PrettyWriter;
use crate::backend::Backend;
use crate::service::Service;
use crate::ExitCode;

mod format;

/// Add a metric related to the target
#[derive(clap::Parser, Debug, Default)]
pub struct CommandLog {
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
    fn execute<B: Backend, Out: PrettyWriter>(
        self,
        backend: B,
        stdout: &mut Out,
    ) -> Result<ExitCode, crate::service::Error> {
        let root = backend.root_path()?;
        let config = crate::entity::config::Config::from_root_path(&root)?;
        let result = Service::new(backend).log(&crate::service::log::Options {
            target: self.target,
        })?;
        format::TextFormatter {
            filter_empty: self.filter_empty,
        }
        .format(result, &config, stdout)?;
        Ok(ExitCode::Success)
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
