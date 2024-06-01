use crate::{backend::Backend, entity::Metric};

#[derive(Debug)]
pub(crate) struct Options {
    pub target: String,
}

impl<B: Backend> super::Service<B> {
    pub(crate) fn add(&self, metric: Metric, opts: &Options) -> Result<(), super::Error> {
        let mut metrics = self.backend.get_metrics(&opts.target)?;
        metrics.push(metric);
        self.backend.set_metrics(&opts.target, metrics)?;

        Ok(())
    }
}
