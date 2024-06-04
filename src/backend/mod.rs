use std::fmt::Display;

#[cfg(feature = "impl-command")]
mod command;
#[cfg(feature = "impl-git2")]
mod git2;
#[cfg(test)]
pub(crate) mod mock;

#[cfg(feature = "impl-command")]
pub(crate) use command::CommandBackend;
#[cfg(feature = "impl-git2")]
pub(crate) use git2::Git2Backend;
use serde::Serializer;

use crate::entity::{Commit, Metric};

const REMOTE_METRICS_REF: &str = "refs/notes/metrics";

#[derive(Clone, Debug)]
pub(crate) enum NoteRef {
    Changes,
    RemoteMetrics { name: String },
}

impl NoteRef {
    pub(crate) fn remote_metrics(name: impl Into<String>) -> Self {
        Self::RemoteMetrics { name: name.into() }
    }
}

impl std::fmt::Display for NoteRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Changes => write!(f, "refs/notes/metrics-changes"),
            Self::RemoteMetrics { name } => write!(f, "refs/notes/metrics-remote-{name}"),
        }
    }
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct NoteContent {
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

#[derive(Debug)]
pub(crate) struct Note {
    #[allow(dead_code)]
    pub note_id: String,
    pub commit_id: String,
}

#[derive(Clone, Debug)]
pub(crate) enum RevParse {
    Single(String),
    Range(String, String),
}

impl Display for RevParse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Single(inner) => f.write_str(inner.as_str()),
            Self::Range(first, second) => write!(f, "{first}..{second}"),
        }
    }
}

pub(crate) trait Backend {
    fn rev_parse(&self, range: &str) -> Result<RevParse, Error>;
    fn rev_list(&self, range: &str) -> Result<Vec<String>, Error>;
    fn pull(&self, remote: &str, local_ref: &NoteRef) -> Result<(), Error>;
    fn push(&self, remote: &str, local_ref: &NoteRef) -> Result<(), Error>;
    fn read_note<T: serde::de::DeserializeOwned>(
        &self,
        target: &str,
        note_ref: &NoteRef,
    ) -> Result<Option<T>, Error>;
    fn write_note<T: serde::Serialize>(
        &self,
        target: &str,
        note_ref: &NoteRef,
        value: &T,
    ) -> Result<(), Error>;
    fn remove_note(&self, target: &str, note_ref: &NoteRef) -> Result<(), Error>;
    fn list_notes(&self, note_ref: &NoteRef) -> Result<Vec<Note>, Error>;
    fn get_commits(&self, range: &str) -> Result<Vec<Commit>, Error>;
}
