use crate::{
    backend::Backend,
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

#[derive(Debug, serde::Deserialize)]
struct TomlList<T> {
    content: Vec<T>,
}

pub(crate) struct Service<B> {
    backend: B,
}

impl<B: Backend> Service<B> {
    pub(crate) fn new(backend: B) -> Self {
        Self { backend }
    }

    pub(crate) fn get_metrics(&self, commit_sha: &str) -> Result<MetricStack, Error> {
        let remote_metrics = self
            .backend
            .read_note::<TomlList<Metric>>(commit_sha, crate::backend::REMOTE_METRICS_REF)?
            .map(|list| list.content)
            .unwrap_or_default();

        let diff_metrics = self
            .backend
            .read_note::<TomlList<MetricChange>>(commit_sha, crate::backend::LOCAL_METRICS_REF)?
            .map(|list| list.content)
            .unwrap_or_default();

        Ok(MetricStack::from_iter(remote_metrics.into_iter())
            .with_changes(diff_metrics.into_iter()))
    }
}
