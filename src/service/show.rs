// TODO extract display out of service

use std::io::Write;

use crate::{backend::Backend, cmd::format::text::TextMetric};

#[derive(Debug)]
pub(crate) struct Options {
    pub target: String,
}

impl<B: Backend> super::Service<B> {
    pub(crate) fn show<Out: Write>(
        &self,
        stdout: &mut Out,
        opts: &Options,
    ) -> Result<(), super::Error> {
        let metrics = self.get_metrics(&opts.target, "origin")?;
        for m in metrics.into_metric_iter() {
            writeln!(stdout, "{}", TextMetric(&m))?;
        }
        Ok(())
    }
}
