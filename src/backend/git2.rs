use std::path::PathBuf;

use crate::entity::Commit;

use super::{Backend, Error};
use super::{
    HEAD, LOCAL_METRICS_REF, REMOTE_METRICS_MAP, REMOTE_METRICS_MAP_FORCE, REMOTE_METRICS_REF,
};

macro_rules! with_error {
    ($msg:expr) => {
        |err| {
            tracing::error!(concat!($msg, ": {:?}"), err);
            Error::new($msg, err)
        }
    };
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
                Error::new("target not found", err)
            })
    }

    fn signature(&self) -> Result<git2::Signature, Error> {
        tracing::trace!("fetching signature");
        self.repo.signature().map_err(|err| {
            tracing::error!("unable to get signature: {err:?}");
            Error::new("unable to get signature", err)
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
    fn read_note<T: serde::de::DeserializeOwned>(
        &self,
        target: &str,
        note_ref: &str,
    ) -> Result<Option<T>, Error> {
        tracing::trace!("reading note for target {target:?} and ref {note_ref:?}");
        let rev_id = self.revision_id(target)?;

        let Ok(note) = self.repo.find_note(Some(note_ref), rev_id) else {
            tracing::debug!("no note found for revision {rev_id:?}");
            return Ok(None);
        };

        note.message()
            .map(|msg| {
                tracing::trace!("deserializing note content");
                toml::from_str::<T>(msg)
                    .map(Some)
                    .map_err(with_error!("unable to deserialize not"))
            })
            .unwrap_or_else(|| {
                tracing::debug!("no message found for note {:?}", note.id());
                Ok(None)
            })
    }

    fn write_note<T: serde::Serialize>(
        &self,
        target: &str,
        note_ref: &str,
        value: &T,
    ) -> Result<(), Error> {
        tracing::trace!("setting note for target {target:?} and ref {note_ref:?}",);
        let head_id = self.revision_id(target)?;
        let sig = self.signature()?;

        tracing::trace!("serializing metrics");
        let note =
            toml::to_string_pretty(value).map_err(with_error!("unable to serialize metrics"))?;
        self.repo
            .note(&sig, &sig, Some(note_ref), head_id, &note, true)
            .map_err(with_error!("unable to persist metrics"))?;

        Ok(())
    }

    fn pull(&self, remote: &str) -> Result<(), Error> {
        let config = self
            .repo
            .config()
            .map_err(with_error!("unable to read config"))?;
        let mut remote = self
            .repo
            .find_remote(remote)
            .map_err(with_error!("unable to find remote"))?;

        let auth = self.authenticator();
        let mut remote_cb = git2::RemoteCallbacks::new();
        remote_cb.credentials(auth.credentials(&config));

        let mut fetch_opts = git2::FetchOptions::new();
        fetch_opts.remote_callbacks(remote_cb);
        remote
            .fetch(&[REMOTE_METRICS_MAP_FORCE], Some(&mut fetch_opts), None)
            .map_err(|err| {
                tracing::error!("unable to pull metrics: {err:?}");
                Error::new("unable to pull metrics", err)
            })?;

        let remote_metrics = self.get_metrics_for_ref(HEAD, REMOTE_METRICS_REF)?;
        let local_metrics = self.get_metrics_for_ref(HEAD, LOCAL_METRICS_REF)?;
        let metrics = crate::entity::merge_metrics(remote_metrics, local_metrics);

        self.set_metrics_for_ref(HEAD, LOCAL_METRICS_REF, metrics)?;

        Ok(())
    }

    fn push(&self, remote: &str) -> Result<(), Error> {
        let config = self.repo.config().map_err(|err| {
            tracing::error!("unable to read config: {err:?}");
            Error::new("unable to read config", err)
        })?;
        let mut remote = self
            .repo
            .find_remote(remote)
            .map_err(with_error!("unable to find remote"))?;
        let auth = self.authenticator();
        let mut remote_cb = git2::RemoteCallbacks::new();
        remote_cb.credentials(auth.credentials(&config));
        remote_cb.push_update_reference(|first, second| {
            tracing::trace!("first={first:?} second={second:?}");
            Ok(())
        });

        let mut push_opts = git2::PushOptions::new();
        push_opts.remote_callbacks(remote_cb);

        let remote_metrics = self.get_metrics_for_ref(HEAD, REMOTE_METRICS_REF)?;
        let local_metrics = self.get_metrics_for_ref(HEAD, LOCAL_METRICS_REF)?;
        let metrics = crate::entity::merge_metrics(remote_metrics, local_metrics);

        self.set_metrics_for_ref(HEAD, REMOTE_METRICS_REF, metrics)?;

        remote
            .push(&[REMOTE_METRICS_MAP], Some(&mut push_opts))
            .map_err(with_error!("unable to push metrics"))
    }

    fn get_commits(&self, range: &str) -> Result<Vec<Commit>, Error> {
        let mut revwalk = self
            .repo
            .revwalk()
            .map_err(with_error!("unable to lookup commits"))?;
        revwalk
            .set_sorting(git2::Sort::TOPOLOGICAL)
            .map_err(with_error!("unable to set sorting direction"))?;
        let revspec = self
            .repo
            .revparse(range.as_ref())
            .map_err(with_error!("unable to parse commit range"))?;
        if revspec.mode().contains(git2::RevparseMode::SINGLE) {
            let from = revspec.from().ok_or_else(|| {
                tracing::error!("unable to get range beginning");
                Error::new(
                    "unable to get range beginning",
                    git2::Error::from_str("revspec.from is None"),
                )
            })?;
            revwalk
                .push(from.id())
                .map_err(with_error!("unable to push commit id in revwalk"))?;
        } else {
            let from = revspec.from().ok_or_else(|| {
                tracing::error!("unable to get range beginning");
                Error::new(
                    "unable to get range beginning",
                    git2::Error::from_str("revspec.from is None"),
                )
            })?;
            let to = revspec.from().ok_or_else(|| {
                tracing::error!("unable to get range ending");
                Error::new(
                    "unable to get range ending",
                    git2::Error::from_str("revspec.to is None"),
                )
            })?;
            revwalk
                .push(from.id())
                .map_err(with_error!("unable to push commit id in revwalk"))?;
            if revspec.mode().contains(git2::RevparseMode::MERGE_BASE) {
                let base = self
                    .repo
                    .merge_base(from.id(), to.id())
                    .map_err(with_error!("unable to get merge base"))?;
                let o = self
                    .repo
                    .find_object(base, Some(git2::ObjectType::Commit))
                    .map_err(with_error!("unable to get commit"))?;
                revwalk
                    .push(o.id())
                    .map_err(with_error!("unable to push commit id in revwalk"))?;
            }
            revwalk
                .hide(from.id())
                .map_err(with_error!("unable to hide commit id in revwalk"))?;
        }

        let mut result = Vec::new();
        for commit_id in revwalk {
            let commit_id = commit_id.map_err(with_error!("unable to get commit from revwalk"))?;
            let commit = self
                .repo
                .find_commit(commit_id)
                .map_err(with_error!("unable to get commit"))?;
            let summary = commit.summary().map(String::from).unwrap_or_default();
            result.push(Commit {
                sha: commit_id.to_string(),
                summary,
            })
        }

        Ok(result)
    }
}
