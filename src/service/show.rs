use crate::backend::Backend;
use crate::entity::metric::MetricStack;

#[derive(Debug)]
pub(crate) struct Options {
    pub target: String,
}

impl<B: Backend> super::Service<B> {
    pub(crate) fn show(&self, opts: &Options) -> Result<MetricStack, super::Error> {
        self.get_metrics(&opts.target, "origin")
    }
}
