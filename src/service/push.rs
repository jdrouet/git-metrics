use super::MetricList;
use crate::backend::{Backend, NoteRef};
use crate::entity::metric::MetricStack;

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
        let remote_ref = NoteRef::remote_metrics(&opts.remote);
        let local_notes = self.backend.list_notes(&NoteRef::Changes)?;

        for commit_sha in local_notes.into_iter().map(|item| item.commit_id) {
            let remote_metrics = self
                .backend
                .read_note::<MetricList>(commit_sha.as_str(), &remote_ref)?
                .map(|list| list.metrics)
                .unwrap_or_default();

            let diff_metrics = self.get_metric_changes(commit_sha.as_str())?;

            if !diff_metrics.is_empty() {
                let new_metrics = MetricStack::from_iter(remote_metrics.into_iter())
                    .with_changes(diff_metrics.into_iter())
                    .into_vec();
                self.set_metrics_for_ref(commit_sha.as_str(), &remote_ref, new_metrics)?;
            }
        }

        self.backend.push(opts.remote.as_str(), &remote_ref)?;
        self.prune_notes_in_ref(&NoteRef::Changes)?;

        Ok(())
    }
}
