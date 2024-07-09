use crate::backend::Backend;
use crate::entity::metric::{Metric, MetricChange};

#[derive(Debug)]
pub(crate) struct Options {
    pub target: String,
}

impl<B: Backend> super::Service<B> {
    pub(crate) fn remove(&self, index: usize, opts: &Options) -> Result<(), super::Error> {
        let metrics = self.get_metrics(&opts.target, "remote")?;
        if let Some((header, value)) = metrics.at(index) {
            let mut changes = self.get_metric_changes(&opts.target)?;
            changes.push(MetricChange::Remove(Metric {
                header: header.clone(),
                value,
            }));
            self.set_metric_changes(&opts.target, changes)?;
        }

        Ok(())
    }
}
