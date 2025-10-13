use super::prelude::PrettyWriter;
use crate::backend::Backend;
use crate::service::Service;
use crate::ExitCode;

mod format;

/// Add a metric related to the target
#[derive(clap::Parser, Debug, Default)]
pub struct CommandLog {
    /// Remote name, default to origin
    #[clap(long, default_value = "origin")]
    remote: String,
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
        stdout: Out,
        alternative_config: Option<crate::entity::config::Config>,
    ) -> Result<ExitCode, crate::service::Error> {
        let svc = Service::new(backend);
        let config = if let Some(cfg) = alternative_config {
            cfg
        } else {
            svc.open_config()?
        };
        let result = svc.log(&crate::service::log::Options {
            remote: self.remote.as_str(),
            target: self.target.as_str(),
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
