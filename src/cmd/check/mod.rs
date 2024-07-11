use std::io::Write;

use crate::backend::Backend;
use crate::service::Service;
use crate::ExitCode;

mod format;

/// Show metrics changes
#[derive(clap::Parser, Debug, Default)]
pub(crate) struct CommandCheck {
    /// Show the successful rules
    #[clap(long)]
    show_success_rules: bool,
    /// Show the skipped rules
    #[clap(long)]
    show_skipped_rules: bool,
    /// Commit range, default to HEAD
    ///
    /// Can use ranges like HEAD~2..HEAD
    #[clap(default_value = "HEAD")]
    target: String,
}

impl super::Executor for CommandCheck {
    #[tracing::instrument(name = "check", skip_all, fields(target = self.target.as_str()))]
    fn execute<B: Backend, Out: Write>(
        self,
        backend: B,
        stdout: &mut Out,
    ) -> Result<crate::ExitCode, crate::service::Error> {
        let opts = crate::service::check::Options {
            remote: "origin",
            target: self.target.as_str(),
        };
        let checklist = Service::new(backend).check(&opts)?;
        format::TextFormatter {
            show_success_rules: self.show_success_rules,
            show_skipped_rules: self.show_skipped_rules,
        }
        .format(&checklist, stdout)?;

        if checklist.status.is_failed() {
            Ok(ExitCode::Failure)
        } else {
            Ok(ExitCode::Success)
        }
    }
}
