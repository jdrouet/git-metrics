use crate::entity::{check::CheckList, log::LogEntry};
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(feature = "exporter-json")]
    #[error("unable to write to json file")]
    Json(
        #[from]
        #[source]
        serde_json::Error,
    ),
    #[cfg(feature = "exporter-json")]
    #[error("unable to open file")]
    Io(
        #[from]
        #[source]
        std::io::Error,
    ),
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct Payload {
    target: String,
    checks: CheckList,
    logs: Vec<LogEntry>,
}

impl Payload {
    pub(crate) fn new(target: String, checks: CheckList, logs: Vec<LogEntry>) -> Self {
        Self {
            target,
            checks,
            logs,
        }
    }
}

#[cfg(feature = "exporter-json")]
pub(crate) fn to_json_file(path: &Path, payload: &Payload) -> Result<(), Error> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    to_json_writer(&mut file, payload)?;
    Ok(())
}

#[cfg(feature = "exporter-json")]
pub(crate) fn to_json_writer<W: std::io::Write>(output: W, payload: &Payload) -> Result<(), Error> {
    serde_json::to_writer(output, payload)?;
    Ok(())
}
