use std::collections::HashSet;

use crate::backend::{Backend, NoteRef};

impl<B: Backend> super::Service<B> {
    #[inline]
    fn prune_notes_in_ref(&self, note_ref: &NoteRef) -> Result<(), super::Error> {
        let notes = self.backend.list_notes(note_ref)?;
        for note in notes {
            self.backend.remove_note(&note.commit_id, note_ref)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct Options {
    pub remote: String,
}

impl<B: Backend> super::Service<B> {
    pub(crate) fn push(&self, opts: &Options) -> Result<(), super::Error> {
        self.prune_notes_in_ref(&NoteRef::Cache)?;
        let remote_notes = self
            .backend
            .list_notes(&NoteRef::remote_metrics(&opts.remote))?;
        let local_notes = self.backend.list_notes(&NoteRef::Changes)?;
        let commit_shas = remote_notes
            .iter()
            .map(|note| note.commit_id.as_str())
            .chain(local_notes.iter().map(|note| note.commit_id.as_str()))
            .collect::<HashSet<&str>>();
        for commit_sha in commit_shas {
            let metrics = self.get_metrics(commit_sha, &opts.remote)?;
            self.set_metrics_cache(commit_sha, metrics.into_vec())?;
        }

        self.backend.push(opts.remote.as_str(), &NoteRef::Cache)?;

        self.pull(&super::pull::Options {
            remote: opts.remote.clone(),
        })
    }
}
