use crate::backend::Backend;
use crate::entity::metric::MetricStack;

#[derive(Debug)]
pub(crate) struct Options<'a> {
    pub remote: &'a str,
    pub target: &'a str,
}

impl<B: Backend> super::Service<B> {
    pub(crate) fn show(&self, opts: &Options) -> Result<MetricStack, super::Error> {
        self.get_metrics(opts.target, opts.remote)
    }
}
