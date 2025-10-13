use super::prelude::PrettyWriter;
use crate::entity::metric::Metric;
use crate::importer::Importer;
use crate::ExitCode;

#[cfg(feature = "importer-lcov")]
mod lcov;

#[derive(Debug, clap::Subcommand)]
enum CommandImporter {
    /// Just for testing, will import nothing.
    #[cfg(feature = "importer-noop")]
    Noop,
    #[cfg(feature = "importer-lcov")]
    Lcov(lcov::LcovImporter),
}

impl crate::importer::Importer for CommandImporter {
    fn import(self) -> Result<Vec<Metric>, crate::importer::Error> {
        match self {
            #[cfg(feature = "importer-noop")]
            Self::Noop => Ok(Vec::new()),
            #[cfg(feature = "importer-lcov")]
            Self::Lcov(inner) => inner.import(),
        }
    }
}

/// Import metrics in batch from source files.
#[derive(clap::Parser, Debug)]
pub struct CommandImport {
    /// Commit target, default to HEAD
    #[clap(long, short, default_value = "HEAD")]
    target: String,

    /// Load metrics without adding them to the repository
    #[clap(long, default_value = "false")]
    dry_run: bool,

    #[command(subcommand)]
    importer: CommandImporter,
}

impl crate::cmd::Executor for CommandImport {
    fn execute<B: crate::backend::Backend, Out: PrettyWriter>(
        self,
        backend: B,
        _stdout: Out,
        _alternative_config: Option<crate::entity::config::Config>,
    ) -> Result<ExitCode, crate::service::Error> {
        let metrics = self.importer.import()?;
        if metrics.is_empty() {
            tracing::debug!("no metrics found");
            return Ok(ExitCode::Success);
        }

        tracing::debug!("{} metrics found", metrics.len());

        if self.dry_run {
            for metric in metrics {
                tracing::info!("{metric:?}");
            }
            tracing::debug!("dry run aborting early");
            return Ok(ExitCode::Success);
        }

        let svc = crate::service::Service::new(backend);
        let opts = crate::service::add::Options {
            target: self.target,
        };

        for metric in metrics {
            tracing::trace!("importing {metric:?}");
            svc.add(metric, &opts)?;
        }
        tracing::debug!("import done");
        Ok(ExitCode::Success)
    }
}
