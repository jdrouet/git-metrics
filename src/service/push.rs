use crate::backend::Backend;

#[derive(Debug)]
pub(crate) struct Options {
    pub remote: String,
}

impl<B: Backend> super::Service<B> {
    pub(crate) fn push(&self, opts: &Options) -> Result<(), super::Error> {
        self.backend.push(opts.remote.as_str())?;
        Ok(())
    }
}
