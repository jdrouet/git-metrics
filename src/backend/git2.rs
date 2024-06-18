use std::path::PathBuf;

use super::{Backend, Note, NoteRef, REMOTE_METRICS_REF};
use crate::backend::RevParse;
use crate::entity::Commit;

macro_rules! with_git2_error {
    ($msg:expr) => {
        |err| {
            tracing::error!(concat!($msg, ": {:?}"), err);
            Error::Git2 {
                message: $msg,
                source: err,
            }
        }
    };
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("{message}\n\n{source}")]
    Git2 {
        message: &'static str,
        #[source]
        source: git2::Error,
    },
    #[error("{message}")]
    Race { message: &'static str },
    #[error("unable to deserialize metrics\n\n{source}")]
    Deserialize {
        #[source]
        source: toml::de::Error,
    },
    #[error("unable to serialize metrics\n\n{source}")]
    Serialize {
        #[source]
        source: toml::ser::Error,
    },
}

impl Error {
    #[inline]
    fn git2(message: &'static str, source: git2::Error) -> Self {
        Self::Git2 { message, source }
    }

    #[inline]
    fn race(message: &'static str) -> Self {
        Self::Race { message }
    }
}

#[derive(Default)]
pub(crate) struct GitCredentials {
    username: Option<String>,
    password: Option<String>,
}

impl From<crate::cmd::GitCredentials> for GitCredentials {
    fn from(value: crate::cmd::GitCredentials) -> Self {
        Self {
            username: value.username,
            password: value.password,
        }
    }
}

pub(crate) struct Git2Backend {
    repo: git2::Repository,
    credentials: GitCredentials,
}

impl Git2Backend {
    pub(crate) fn new(root: Option<PathBuf>) -> Result<Self, String> {
        let repo = match root {
            Some(path) => {
                tracing::debug!("opening repository in {path:?}");
                git2::Repository::open(path)
            }
            None => {
                tracing::debug!("opening repository based on environment");
                git2::Repository::open_from_env()
            }
        }
        .map_err(|err| format!("unable to open repository: {err:?}"))?;
        Ok(Git2Backend {
            repo,
            credentials: GitCredentials::default(),
        })
    }

    pub(crate) fn with_credentials(mut self, creds: impl Into<GitCredentials>) -> Self {
        self.credentials = creds.into();
        self
    }

    fn revision_id(&self, target: &str) -> Result<git2::Oid, Error> {
        tracing::trace!("fetching revision id for target {target:?}");
        self.repo
            .revparse_single(target)
            .map(|rev| rev.id())
            .map_err(|err| {
                tracing::error!("unable to find revision id for target {target:?}: {err:?}");
                Error::git2("unable to find revision id", err)
            })
    }

    fn signature(&self) -> Result<git2::Signature, Error> {
        tracing::trace!("fetching signature");
        self.repo.signature().map_err(|err| {
            tracing::error!("unable to get signature: {err:?}");
            Error::git2("unable to get signature", err)
        })
    }

    fn authenticator(&self) -> auth_git2::GitAuthenticator {
        let auth = auth_git2::GitAuthenticator::new();
        match (
            self.credentials.username.as_deref(),
            self.credentials.password.as_deref(),
        ) {
            (Some(username), Some(password)) => {
                auth.add_plaintext_credentials("*", username, password)
            }
            (Some(username), None) => auth.add_username("*", username),
            _ => auth,
        }
    }
}

impl Backend for Git2Backend {
    type Err = Error;

    fn rev_list(&self, range: &str) -> Result<Vec<String>, Error> {
        tracing::trace!("listing revisions in range {range:?}");
        let mut revwalk = self
            .repo
            .revwalk()
            .map_err(with_git2_error!("unable to lookup commits"))?;
        revwalk
            .set_sorting(git2::Sort::TOPOLOGICAL)
            .map_err(with_git2_error!("unable to set sorting direction"))?;
        let revspec = self
            .repo
            .revparse(range.as_ref())
            .map_err(with_git2_error!("unable to parse commit range"))?;
        if revspec.mode().contains(git2::RevparseMode::SINGLE) {
            let from = revspec.from().ok_or_else(|| {
                tracing::error!("unable to get range beginning");
                Error::race("unable to get range beginning: revspec.from is None")
            })?;
            tracing::trace!("using from {:?}", from.id());
            revwalk
                .push(from.id())
                .map_err(with_git2_error!("unable to push commit id in revwalk"))?;
        } else {
            let from = revspec.from().ok_or_else(|| {
                tracing::error!("unable to get range beginning");
                Error::race("unable to get range beginning: revspec.from is None")
            })?;
            let to = revspec.to().ok_or_else(|| {
                tracing::error!("unable to get range ending");
                Error::race("unable to get range ending: revspec.to is None")
            })?;
            tracing::trace!("using range {:?}..{:?}", from.id(), to.id());
            revwalk
                .push(to.id())
                .map_err(with_git2_error!("unable to push commit id in revwalk"))?;
            if revspec.mode().contains(git2::RevparseMode::MERGE_BASE) {
                tracing::trace!("using mode MERGE_BASE");
                let base = self
                    .repo
                    .merge_base(from.id(), to.id())
                    .map_err(with_git2_error!("unable to get merge base"))?;
                let o = self
                    .repo
                    .find_object(base, Some(git2::ObjectType::Commit))
                    .map_err(with_git2_error!("unable to get commit"))?;
                revwalk
                    .push(o.id())
                    .map_err(with_git2_error!("unable to push commit id in revwalk"))?;
            }
            revwalk
                .hide(from.id())
                .map_err(with_git2_error!("unable to hide commit id in revwalk"))?;
        }

        let mut res = Vec::new();
        for commit in revwalk {
            let commit_id =
                commit.map_err(with_git2_error!("unable to get commit from revwalk"))?;
            res.push(commit_id.to_string());
        }

        Ok(res)
    }

    fn rev_parse(&self, range: &str) -> Result<super::RevParse, Error> {
        tracing::trace!("parse revision range {range:?}");
        let revspec = self
            .repo
            .revparse(range.as_ref())
            .map_err(with_git2_error!("unable to parse commit range"))?;
        if revspec.mode().contains(git2::RevparseMode::SINGLE) {
            let commit = revspec.from().ok_or_else(|| {
                tracing::error!("unable to get range beginning");
                Error::race("unable to get range beginning: revspec.from is None")
            })?;
            tracing::trace!("using from {:?}", commit.id());
            Ok(super::RevParse::Single(commit.id().to_string()))
        } else {
            let first = revspec.from().ok_or_else(|| {
                tracing::error!("unable to get range beginning");
                Error::race("unable to get range beginning: revspec.from is None")
            })?;
            let second = revspec.to().ok_or_else(|| {
                tracing::error!("unable to get range ending");
                Error::race("unable to get range ending: revspec.to is None")
            })?;
            tracing::trace!("using range {:?}..{:?}", first.id(), second.id());
            Ok(RevParse::Range(
                first.id().to_string(),
                second.id().to_string(),
            ))
        }
    }

    fn list_notes(&self, note_ref: &NoteRef) -> Result<Vec<Note>, Error> {
        tracing::trace!("listing notes for ref {note_ref}");
        let notes = match self.repo.notes(Some(&note_ref.to_string())) {
            Ok(notes) => notes,
            Err(error) => {
                let not_found_msg = format!("reference '{note_ref}' not found");
                if error.message() == not_found_msg {
                    return Ok(Vec::with_capacity(0));
                }
                return Err(with_git2_error!("unable to list notes")(error));
            }
        };
        Ok(notes
            .filter_map(|note| note.ok())
            .map(|(note_id, commit_id)| super::Note {
                note_id: note_id.to_string(),
                commit_id: commit_id.to_string(),
            })
            .collect())
    }

    fn remove_note(&self, target: &str, note_ref: &NoteRef) -> Result<(), Error> {
        tracing::trace!("removing note for target {target:?} and {note_ref:?}");
        let rev_id = self.revision_id(target)?;
        let sig = self.signature()?;
        self.repo
            .note_delete(rev_id, Some(&note_ref.to_string()), &sig, &sig)
            .map_err(with_git2_error!("unable to remove note"))?;

        Ok(())
    }

    fn read_note<T: serde::de::DeserializeOwned>(
        &self,
        target: &str,
        note_ref: &NoteRef,
    ) -> Result<Option<T>, Error> {
        tracing::trace!("reading note for target {target:?} and ref {note_ref:?}");
        let rev_id = self.revision_id(target)?;

        let Ok(note) = self.repo.find_note(Some(&note_ref.to_string()), rev_id) else {
            tracing::debug!("no note found for revision {rev_id:?}");
            return Ok(None);
        };

        note.message()
            .map(|msg| {
                tracing::trace!("deserializing note content");
                toml::from_str::<T>(msg).map(Some).map_err(|err| {
                    tracing::error!("unable to deserialize metrics: {err:?}");
                    Error::Deserialize { source: err }
                })
            })
            .unwrap_or_else(|| {
                tracing::debug!("no message found for note {:?}", note.id());
                Ok(None)
            })
    }

    fn write_note<T: serde::Serialize>(
        &self,
        target: &str,
        note_ref: &NoteRef,
        value: &T,
    ) -> Result<(), Error> {
        tracing::trace!("setting note for target {target:?} and ref {note_ref:?}",);
        let head_id = self.revision_id(target)?;
        let sig = self.signature()?;

        tracing::trace!("serializing metrics");
        let note = toml::to_string_pretty(value).map_err(|err| {
            tracing::error!("unable to serialize metrics: {err:?}");
            Error::Serialize { source: err }
        })?;
        self.repo
            .note(
                &sig,
                &sig,
                Some(&note_ref.to_string()),
                head_id,
                &note,
                true,
            )
            .map_err(with_git2_error!("unable to persist metrics"))?;

        Ok(())
    }

    fn pull(&self, remote_name: &str, local_ref: &NoteRef) -> Result<(), Error> {
        let config = self
            .repo
            .config()
            .map_err(with_git2_error!("unable to read config"))?;
        let mut remote = self
            .repo
            .find_remote(remote_name)
            .map_err(with_git2_error!("unable to find remote"))?;

        let auth = self.authenticator();
        let mut remote_cb = git2::RemoteCallbacks::new();
        remote_cb.credentials(auth.credentials(&config));

        let mut fetch_opts = git2::FetchOptions::new();
        fetch_opts.remote_callbacks(remote_cb);
        remote
            .fetch(
                &[format!("+{REMOTE_METRICS_REF}:{local_ref}",)],
                Some(&mut fetch_opts),
                None,
            )
            .map_err(with_git2_error!("unable to pull metrics"))?;

        Ok(())
    }

    fn push(&self, remote_name: &str, local_ref: &NoteRef) -> Result<(), Error> {
        let config = self
            .repo
            .config()
            .map_err(with_git2_error!("unable to read config"))?;
        let mut remote = self
            .repo
            .find_remote(remote_name)
            .map_err(with_git2_error!("unable to find remote"))?;
        let auth = self.authenticator();
        let mut remote_cb = git2::RemoteCallbacks::new();
        remote_cb.credentials(auth.credentials(&config));
        remote_cb.push_update_reference(|first, second| {
            tracing::trace!("first={first:?} second={second:?}");
            Ok(())
        });

        let mut push_opts = git2::PushOptions::new();
        push_opts.remote_callbacks(remote_cb);

        remote
            .push(
                &[format!("{local_ref}:{REMOTE_METRICS_REF}",)],
                Some(&mut push_opts),
            )
            .map_err(with_git2_error!("unable to push metrics"))
    }

    fn get_commits(&self, range: &str) -> Result<Vec<Commit>, Error> {
        let mut revwalk = self
            .repo
            .revwalk()
            .map_err(with_git2_error!("unable to lookup commits"))?;
        revwalk
            .set_sorting(git2::Sort::TOPOLOGICAL)
            .map_err(with_git2_error!("unable to set sorting direction"))?;
        let revspec = self
            .repo
            .revparse(range.as_ref())
            .map_err(with_git2_error!("unable to parse commit range"))?;
        if revspec.mode().contains(git2::RevparseMode::SINGLE) {
            let from = revspec.from().ok_or_else(|| {
                tracing::error!("unable to get range beginning");
                Error::Race {
                    message: "unable to get range beginning: revspec.from is None",
                }
            })?;
            revwalk
                .push(from.id())
                .map_err(with_git2_error!("unable to push commit id in revwalk"))?;
        } else {
            let from = revspec.from().ok_or_else(|| {
                tracing::error!("unable to get range beginning");
                Error::Race {
                    message: "unable to get range beginning",
                }
            })?;
            let to = revspec.to().ok_or_else(|| {
                tracing::error!("unable to get range ending");
                Error::race("unable to get range ending: revspec.to is None")
            })?;
            revwalk
                .push(to.id())
                .map_err(with_git2_error!("unable to push commit id in revwalk"))?;
            if revspec.mode().contains(git2::RevparseMode::MERGE_BASE) {
                let base = self
                    .repo
                    .merge_base(from.id(), to.id())
                    .map_err(with_git2_error!("unable to get merge base"))?;
                let o = self
                    .repo
                    .find_object(base, Some(git2::ObjectType::Commit))
                    .map_err(with_git2_error!("unable to get commit"))?;
                revwalk
                    .push(o.id())
                    .map_err(with_git2_error!("unable to push commit id in revwalk"))?;
            }
            revwalk
                .hide(from.id())
                .map_err(with_git2_error!("unable to hide commit id in revwalk"))?;
        }

        let mut result = Vec::new();
        for commit_id in revwalk {
            let commit_id =
                commit_id.map_err(with_git2_error!("unable to get commit from revwalk"))?;
            let commit = self
                .repo
                .find_commit(commit_id)
                .map_err(with_git2_error!("unable to get commit"))?;
            let summary = commit.summary().map(String::from).unwrap_or_default();
            result.push(Commit {
                sha: commit_id.to_string(),
                summary,
            });
        }

        Ok(result)
    }
}
