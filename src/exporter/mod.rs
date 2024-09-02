use std::path::Path;

use crate::entity::check::CheckList;
use crate::entity::log::LogEntry;

#[cfg(feature = "exporter-json")]
pub(crate) mod json;
#[cfg(feature = "exporter-markdown")]
pub(crate) mod markdown;

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[cfg(feature = "exporter-json")]
    #[error("unable to write to json file")]
    Json(
        #[from]
        #[source]
        serde_json::Error,
    ),
    #[cfg(feature = "exporter")]
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

fn with_file(path: &Path) -> std::io::Result<std::fs::File> {
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
}
