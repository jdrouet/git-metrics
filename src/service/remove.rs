use crate::backend::Backend;

#[derive(Debug)]
pub(crate) struct Options {
    pub target: String,
}

impl<B: Backend> super::Service<B> {
    pub(crate) fn remove(&self, index: usize, opts: &Options) -> Result<(), super::Error> {
        let mut metrics = self.backend.get_metrics(&opts.target)?;
        if index < metrics.len() {
            metrics.remove(index);
            self.backend.set_metrics(&opts.target, metrics)?;
        }

        Ok(())
    }
}
