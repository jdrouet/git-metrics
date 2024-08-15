use std::path::PathBuf;

use crate::ExitCode;

/// Report check result and diff
#[derive(clap::Parser, Debug)]
pub struct CommandExportJson {
    /// Path to write the json output
    #[clap()]
    output: Option<PathBuf>,
}

impl CommandExportJson {
    pub(super) fn execute<W: std::io::Write>(
        self,
        stdout: W,
        payload: &crate::exporter::Payload,
    ) -> Result<ExitCode, crate::service::Error> {
        if let Some(path) = self.output {
            crate::exporter::to_json_file(&path, payload)?;
        } else {
            crate::exporter::to_json_writer(stdout, payload)?;
        }
        Ok(ExitCode::Success)
    }
}
