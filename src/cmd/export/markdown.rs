use std::path::PathBuf;

use crate::entity::config::Config;
use crate::ExitCode;

/// Report check result and diff
#[derive(clap::Parser, Debug)]
pub struct CommandExportMarkdown {
    /// Path to write the json output
    #[clap()]
    output: Option<PathBuf>,
}

impl CommandExportMarkdown {
    pub(super) fn execute<W: std::io::Write>(
        self,
        stdout: W,
        config: Config,
        payload: &crate::exporter::Payload,
    ) -> Result<ExitCode, crate::service::Error> {
        if let Some(path) = self.output {
            crate::exporter::markdown::to_file(&path, &config, payload)?;
        } else {
            crate::exporter::markdown::to_writer(stdout, &config, payload)?;
        }
        Ok(ExitCode::Success)
    }
}
