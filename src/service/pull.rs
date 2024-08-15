use crate::backend::{Backend, NoteRef};

#[derive(Debug)]
pub(crate) struct Options<'a> {
    pub remote: &'a str,
}

impl<B: Backend> super::Service<B> {
    pub(crate) fn pull(&self, opts: &Options) -> Result<(), super::Error> {
        let note_ref = NoteRef::remote_metrics(opts.remote);
        self.backend.pull(opts.remote, &note_ref)?;
        Ok(())
    }
}
