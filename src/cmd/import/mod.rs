use super::prelude::PrettyWriter;
use crate::entity::metric::Metric;
use crate::importer::Importer;
use crate::ExitCode;

#[derive(Debug, clap::Subcommand)]
enum CommandImporter {
    Noop,
}

impl crate::importer::Importer for CommandImporter {
    fn import(self) -> Result<Vec<Metric>, crate::importer::Error> {
        match self {
            Self::Noop => Ok(Vec::new()),
        }
    }
}

/// Import metrics in batch from source file
#[derive(clap::Parser, Debug)]
pub struct CommandImport {
    /// Commit target, default to HEAD
    #[clap(long, short, default_value = "HEAD")]
    target: String,

    #[command(subcommand)]
    importer: CommandImporter,
}

impl crate::cmd::Executor for CommandImport {
    fn execute<B: crate::backend::Backend, Out: PrettyWriter>(
        self,
        backend: B,
        _stdout: &mut Out,
    ) -> Result<ExitCode, crate::service::Error> {
        let metrics = self.importer.import()?;
        if metrics.is_empty() {
            tracing::debug!("no metrics found");
            return Ok(ExitCode::Success);
        }

        let svc = crate::service::Service::new(backend);
        let opts = crate::service::add::Options {
            target: self.target,
        };

        tracing::debug!("{} metrics found", metrics.len());
        for metric in metrics {
            svc.add(metric, &opts)?;
        }
        tracing::debug!("import done");
        Ok(ExitCode::Success)
    }
}
