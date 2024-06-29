use std::{fmt::Display, path::PathBuf};

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

use crate::entity::{Commit, Metric};

const REMOTE_METRICS_REF: &str = "refs/notes/metrics";

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[cfg(feature = "impl-command")]
    #[error(transparent)]
    Command(#[from] crate::backend::command::Error),
    #[cfg(feature = "impl-git2")]
    #[error(transparent)]
    Git2(#[from] crate::backend::git2::Error),
    #[cfg(test)]
    #[error(transparent)]
    Mock(#[from] crate::backend::mock::Error),
}

impl crate::error::DetailedError for Error {
    fn details(&self) -> Option<String> {
        match self {
            #[cfg(feature = "impl-command")]
            Self::Command(inner) => inner.details(),
            #[cfg(feature = "impl-git2")]
            Self::Git2(inner) => inner.details(),
            #[cfg(test)]
            Self::Mock(inner) => inner.details(),
        }
    }
}

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
    type Err: Into<Error>;

    fn rev_parse(&self, range: &str) -> Result<RevParse, Self::Err>;
    fn rev_list(&self, range: &str) -> Result<Vec<String>, Self::Err>;
    fn pull(&self, remote: &str, local_ref: &NoteRef) -> Result<(), Self::Err>;
    fn push(&self, remote: &str, local_ref: &NoteRef) -> Result<(), Self::Err>;
    fn read_note<T: serde::de::DeserializeOwned>(
        &self,
        target: &str,
        note_ref: &NoteRef,
    ) -> Result<Option<T>, Self::Err>;
    fn write_note<T: serde::Serialize>(
        &self,
        target: &str,
        note_ref: &NoteRef,
        value: &T,
    ) -> Result<(), Self::Err>;
    fn remove_note(&self, target: &str, note_ref: &NoteRef) -> Result<(), Self::Err>;
    fn list_notes(&self, note_ref: &NoteRef) -> Result<Vec<Note>, Self::Err>;
    fn get_commits(&self, range: &str) -> Result<Vec<Commit>, Self::Err>;
    fn root_path(&self) -> Result<PathBuf, Self::Err>;
}
