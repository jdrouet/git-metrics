use super::prelude::PrettyWriter;
use crate::entity::metric::Metric;
use crate::importer::Importer;
use crate::ExitCode;

#[cfg(feature = "importer-lcov")]
/// Imports metrics from a lcov.info file
///
/// This can be obtained with the following commands
///
/// For Rust, use <https://github.com/taiki-e/cargo-llvm-cov> with the following command.
///
///     cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
///
/// For other languages, feel free to open a PR or an issue with the command.
#[derive(clap::Parser, Debug)]
struct LcovImporter {
    /// Path to the lcov.info file
    path: std::path::PathBuf,
    /// Skip importing branch coverage
    #[clap(long, default_value = "false")]
    disable_branches: bool,
    /// Skip importing function coverage
    #[clap(long, default_value = "false")]
    disable_functions: bool,
    /// Skip importing line coverage
    #[clap(long, default_value = "false")]
    disable_lines: bool,
}

#[derive(Debug, clap::Subcommand)]
enum CommandImporter {
    Noop,
    #[cfg(feature = "importer-lcov")]
    Lcov(LcovImporter),
}

impl crate::importer::Importer for CommandImporter {
    fn import(self) -> Result<Vec<Metric>, crate::importer::Error> {
        match self {
            Self::Noop => Ok(Vec::new()),
            #[cfg(feature = "importer-lcov")]
            Self::Lcov(LcovImporter {
                path,
                disable_branches,
                disable_functions,
                disable_lines,
            }) => crate::importer::lcov::LcovImporter::new(
                path,
                crate::importer::lcov::LcovImporterOptions {
                    branches: !disable_branches,
                    functions: !disable_functions,
                    lines: !disable_lines,
                },
            )
            .import(),
        }
    }
}

/// Import metrics in batch from source file
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
        _stdout: &mut Out,
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
