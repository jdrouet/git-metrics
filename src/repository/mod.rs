use std::fmt::Display;

#[cfg(feature = "impl-command")]
mod command;
#[cfg(feature = "impl-git2")]
mod git2;

use crate::metric::Metric;
#[cfg(feature = "impl-command")]
pub(crate) use command::CommandRepository;
#[cfg(feature = "impl-git2")]
pub(crate) use git2::GitRepository;
use serde::Serializer;

const HEAD: &str = "HEAD";
const LOCAL_METRICS_REF: &str = "refs/notes/local-metrics";
const REMOTE_METRICS_REF: &str = "refs/notes/metrics";
const REMOTE_METRICS_MAP: &str = "refs/notes/metrics:refs/notes/metrics";
const REMOTE_METRICS_MAP_FORCE: &str = "+refs/notes/metrics:refs/notes/metrics";

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

#[cfg_attr(test, mockall::automock)]
pub(crate) trait Repository {
    fn pull(&self, remote: &str) -> Result<(), Error>;
    fn push(&self, remote: &str) -> Result<(), Error>;
    fn get_metrics(&self, target: &str) -> Result<Vec<Metric>, Error>;
    fn set_metrics(&self, target: &str, metrics: Vec<Metric>) -> Result<(), Error>;
}
