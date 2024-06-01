use crate::{backend::Backend, service::Service};
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
    ) -> Result<(), crate::service::Error> {
        let opts = crate::service::log::Options {
            target: self.target,
            hide_empty: self.filter_empty,
        };
        Service::new(backend).log(stdout, &opts)
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
