use super::git::Commit;
use super::metric::{Metric, MetricStack};

#[derive(Debug, serde::Serialize)]
pub struct LogEntry {
    pub commit: Commit,
    pub metrics: Vec<Metric>,
}

impl From<(Commit, MetricStack)> for LogEntry {
    fn from((commit, metrics): (Commit, MetricStack)) -> Self {
        Self {
            commit,
            metrics: metrics.into_vec(),
        }
    }
}
