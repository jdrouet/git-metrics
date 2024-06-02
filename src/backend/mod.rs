use std::fmt::Display;

#[cfg(feature = "impl-command")]
mod command;
#[cfg(feature = "impl-git2")]
mod git2;

use crate::entity::{Commit, Metric};
#[cfg(feature = "impl-command")]
pub(crate) use command::CommandBackend;
#[cfg(feature = "impl-git2")]
pub(crate) use git2::Git2Backend;
use serde::Serializer;

const HEAD: &str = "HEAD";
pub(crate) const LOCAL_METRICS_REF: &str = "refs/notes/local-metrics";
pub(crate) const REMOTE_METRICS_REF: &str = "refs/notes/metrics";
pub(crate) const REMOTE_METRICS_MAP: &str = "refs/notes/metrics:refs/notes/metrics";
pub(crate) const REMOTE_METRICS_MAP_FORCE: &str = "+refs/notes/metrics:refs/notes/metrics";

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct Note {
    metrics: Vec<Metric>,
}

#[derive(Debug)]
pub(crate) struct Error {
    message: &'static str,
    source: Box<dyn std::error::Error + 'static>,
}

impl Error {
    #[inline]
    fn new<E: std::error::Error + 'static>(message: &'static str, err: E) -> Self {
        Self {
            message,
            source: Box::new(err),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.serialize_str(self.message)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.source.as_ref())
    }
}

pub(crate) trait Backend {
    fn pull(&self, remote: &str) -> Result<(), Error>;
    fn push(&self, remote: &str) -> Result<(), Error>;
    fn read_note<T: serde::de::DeserializeOwned>(
        &self,
        target: &str,
        note_ref: &str,
    ) -> Result<Option<T>, Error>;
    fn write_note<T: serde::Serialize>(
        &self,
        target: &str,
        note_ref: &str,
        value: &T,
    ) -> Result<(), Error>;
    fn get_remote_metrics(&self, target: &str) -> Result<Vec<Metric>, Error> {
        self.get_metrics_for_ref(target, REMOTE_METRICS_REF)
    }
    fn get_metrics(&self, target: &str) -> Result<Vec<Metric>, Error> {
        self.get_metrics_for_ref(target, LOCAL_METRICS_REF)
    }
    fn get_metrics_for_ref(&self, target: &str, note_ref: &str) -> Result<Vec<Metric>, Error> {
        self.read_note(target, note_ref)
            .map(|v: Option<Note>| v.unwrap_or_default())
            .map(|v| v.metrics)
    }
    fn set_metrics(&self, target: &str, metrics: Vec<Metric>) -> Result<(), Error> {
        self.set_metrics_for_ref(target, LOCAL_METRICS_REF, metrics)
    }
    fn set_metrics_for_ref(
        &self,
        target: &str,
        note_ref: &str,
        metrics: Vec<Metric>,
    ) -> Result<(), Error> {
        self.write_note(target, note_ref, &Note { metrics })
    }

    fn get_commits(&self, range: &str) -> Result<Vec<Commit>, Error>;
}
