use crate::backend::Backend;
use crate::entity::metric::{Metric, MetricChange};

#[derive(Debug)]
pub(crate) struct Options {
    pub target: String,
}

impl<B: Backend> super::Service<B> {
    pub(crate) fn add(&self, metric: Metric, opts: &Options) -> Result<(), super::Error> {
        let mut changes = self.get_metric_changes(&opts.target)?;
        changes.push(MetricChange::Add(metric));
        self.set_metric_changes(&opts.target, changes)?;

        Ok(())
    }
}
