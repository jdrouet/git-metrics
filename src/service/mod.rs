use crate::backend::{Backend, NoteRef};
use crate::entity::{Metric, MetricChange, MetricStack};

pub(crate) mod add;
pub(crate) mod check;
pub(crate) mod diff;
pub(crate) mod log;
pub(crate) mod pull;
pub(crate) mod push;
pub(crate) mod remove;
pub(crate) mod show;

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("unable to write output")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Backend(crate::backend::Error),
}

impl<E: Into<crate::backend::Error>> From<E> for Error {
    fn from(value: E) -> Self {
        Self::Backend(value.into())
    }
}

impl crate::error::DetailedError for Error {
    fn details(&self) -> Option<String> {
        match self {
            Self::Io(inner) => Some(inner.to_string()),
            Self::Backend(inner) => inner.details(),
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct MetricList {
    #[serde(default)]
    metrics: Vec<Metric>,
}

impl From<Vec<Metric>> for MetricList {
    fn from(value: Vec<Metric>) -> Self {
        Self { metrics: value }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct ChangeList {
    #[serde(default)]
    changes: Vec<MetricChange>,
}

pub(crate) struct Service<B> {
    backend: B,
}

impl<B: Backend> Service<B> {
    pub(crate) fn new(backend: B) -> Self {
        Self { backend }
    }

    pub(crate) fn set_metric_changes(
        &self,
        commit_sha: &str,
        changes: Vec<MetricChange>,
    ) -> Result<(), Error> {
        let payload = ChangeList { changes };
        self.backend
            .write_note(commit_sha, &NoteRef::Changes, &payload)?;
        Ok(())
    }

    pub(crate) fn get_metric_changes(&self, commit_sha: &str) -> Result<Vec<MetricChange>, Error> {
        Ok(self
            .backend
            .read_note::<ChangeList>(commit_sha, &NoteRef::Changes)?
            .map(|list| list.changes)
            .unwrap_or_default())
    }

    pub(crate) fn get_metrics(
        &self,
        commit_sha: &str,
        remote_name: &str,
    ) -> Result<MetricStack, Error> {
        let remote_metrics = self
            .backend
            .read_note::<MetricList>(commit_sha, &NoteRef::remote_metrics(remote_name))?
            .map(|list| list.metrics)
            .unwrap_or_default();

        let diff_metrics = self.get_metric_changes(commit_sha)?;

        Ok(MetricStack::from_iter(remote_metrics.into_iter())
            .with_changes(diff_metrics.into_iter()))
    }

    pub(crate) fn set_metrics_for_ref(
        &self,
        commit_sha: &str,
        note_ref: &NoteRef,
        metrics: Vec<Metric>,
    ) -> Result<(), Error> {
        let payload = MetricList { metrics };
        self.backend.write_note(commit_sha, note_ref, &payload)?;
        Ok(())
    }
}
