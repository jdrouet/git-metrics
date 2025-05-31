use super::prelude::PrettyWriter;
use crate::backend::Backend;
use crate::entity::config::Config;
use crate::entity::log::LogEntry;
use crate::service::Service;
use crate::ExitCode;

#[cfg(feature = "exporter-json")]
mod json;
#[cfg(feature = "exporter-markdown")]
mod markdown;

#[derive(Debug, clap::Subcommand)]
enum ExportFormat {
    Json(json::CommandExportJson),
    Markdown(markdown::CommandExportMarkdown),
}

impl ExportFormat {
    fn execute<W: std::io::Write>(
        self,
        output: W,
        config: Config,
        payload: &crate::exporter::Payload,
    ) -> Result<ExitCode, crate::service::Error> {
        match self {
            Self::Json(inner) => inner.execute(output, payload),
            Self::Markdown(inner) => inner.execute(output, config, payload),
        }
    }
}

/// Report check result and diff
#[derive(clap::Parser, Debug)]
pub struct CommandExport {
    /// Remote name, default to origin
    #[clap(default_value = "origin")]
    remote: String,
    /// Commit range, default to HEAD
    ///
    /// Can use ranges like HEAD~2..HEAD
    #[clap(default_value = "HEAD")]
    target: String,
    /// Output format
    #[command(subcommand)]
    format: ExportFormat,
}

impl super::Executor for CommandExport {
    #[tracing::instrument(name = "export", skip_all, fields(
        remote = self.remote.as_str(),
        target = self.target.as_str(),
    ))]
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

        let checks = svc.check(
            &config,
            &crate::service::check::Options {
                remote: self.remote.as_str(),
                target: self.target.as_str(),
            },
        )?;

        let logs = svc.log(&crate::service::log::Options {
            remote: self.remote.as_str(),
            target: self.target.as_str(),
        })?;

        let payload = crate::exporter::Payload::new(
            self.target,
            checks,
            logs.into_iter().map(LogEntry::from).collect(),
        );
        self.format.execute(stdout, config, &payload)
    }
}
