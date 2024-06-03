use crate::backend::{Backend, NoteRef};

#[derive(Debug)]
pub(crate) struct Options {
    pub remote: String,
}

impl<B: Backend> super::Service<B> {
    pub(crate) fn pull(&self, opts: &Options) -> Result<(), super::Error> {
        let note_ref = NoteRef::remote_metrics(&opts.remote);
        self.backend.pull(opts.remote.as_str(), &note_ref)?;
        Ok(())
    }
}
