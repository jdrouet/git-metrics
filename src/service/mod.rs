use crate::{
    backend::{Backend, NoteRef},
    entity::{Metric, MetricChange, MetricStack},
};

pub(crate) mod add;
pub(crate) mod log;
pub(crate) mod pull;
pub(crate) mod push;
pub(crate) mod remove;
pub(crate) mod show;

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("unable to write to stdout or stderr")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Backend(#[from] crate::backend::Error),
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

    pub(crate) fn set_metrics_cache(
        &self,
        commit_sha: &str,
        metrics: Vec<Metric>,
    ) -> Result<(), Error> {
        let payload = MetricList { metrics };
        self.backend
            .write_note(commit_sha, &NoteRef::Cache, &payload)?;
        Ok(())
    }
}
