use std::fmt::Display;

use crate::metric::Metric;

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct Note {
    metrics: Vec<Metric>,
}

#[derive(Debug)]
pub(crate) enum ErrorKind {
    TargetNotFound,
    SignatureNotFound,
    UnableToDecode,
    UnableToEncode,
    UnableToPersist,
}

#[derive(Debug)]
pub(crate) struct Error {
    kind: ErrorKind,
    source: Box<dyn std::error::Error + 'static>,
}

impl Error {
    #[inline]
    fn signature_not_found<E: std::error::Error + 'static>(err: E) -> Self {
        Self {
            kind: ErrorKind::SignatureNotFound,
            source: Box::new(err),
        }
    }
    #[inline]
    fn target_not_found<E: std::error::Error + 'static>(err: E) -> Self {
        Self {
            kind: ErrorKind::TargetNotFound,
            source: Box::new(err),
        }
    }
    #[inline]
    fn unable_to_decode<E: std::error::Error + 'static>(err: E) -> Self {
        Self {
            kind: ErrorKind::UnableToDecode,
            source: Box::new(err),
        }
    }
    #[inline]
    fn unable_to_encode<E: std::error::Error + 'static>(err: E) -> Self {
        Self {
            kind: ErrorKind::UnableToEncode,
            source: Box::new(err),
        }
    }
    #[inline]
    fn unable_to_persist<E: std::error::Error + 'static>(err: E) -> Self {
        Self {
            kind: ErrorKind::UnableToPersist,
            source: Box::new(err),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self.kind {
            ErrorKind::SignatureNotFound => "unable to get current signature",
            ErrorKind::TargetNotFound => "target not found",
            ErrorKind::UnableToDecode => "unable to decode metrics",
            ErrorKind::UnableToEncode => "unable to encode metrics",
            ErrorKind::UnableToPersist => "unable to persist metrics",
        })
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.source.as_ref())
    }
}

#[cfg_attr(test, mockall::automock)]
pub(crate) trait Repository {
    fn push(&self, remote: &str) -> Result<(), Error>;
    fn get_metrics(&self, target: &str) -> Result<Vec<Metric>, Error>;
    fn set_metrics(&self, target: &str, metrics: Vec<Metric>) -> Result<(), Error>;
}

const NOTES_REF: &str = "refs/notes/metrics";
const NOTES_REF_OPTS: Option<&str> = Some(NOTES_REF);

pub(crate) struct GitRepository {
    repo: git2::Repository,
}

impl GitRepository {
    pub(crate) fn from_env() -> Result<Self, String> {
        let repo = git2::Repository::open_from_env()
            .map_err(|err| format!("unable to open repository: {err:?}"))?;
        Ok(GitRepository { repo })
    }

    fn revision_id(&self, target: &str) -> Result<git2::Oid, Error> {
        self.repo
            .revparse_single(target)
            .map(|rev| rev.id())
            .map_err(Error::target_not_found)
    }

    fn signature(&self) -> Result<git2::Signature, Error> {
        self.repo.signature().map_err(Error::signature_not_found)
    }
}

fn git_credentials(
    _url: &str,
    username_from_url: Option<&str>,
    _allowed_types: git2::CredentialType,
) -> Result<git2::Cred, git2::Error> {
    let res = git2::Cred::default();
    if let Some(username) = username_from_url {
        let res = if let Some(id_rsa) = std::env::var("HOME")
            .ok()
            .map(|value| std::path::PathBuf::from(value).join(".ssh").join("id_rsa"))
            .filter(|value| value.exists())
        {
            git2::Cred::ssh_key(username, None, &id_rsa, None).or(res)
        } else {
            res
        };
        git2::Cred::ssh_key_from_agent(username).or(res)
    } else {
        res
    }
}

impl Repository for GitRepository {
    fn push(&self, remote: &str) -> Result<(), Error> {
        let mut remote_cb = git2::RemoteCallbacks::default();
        remote_cb.credentials(git_credentials);
        let mut push_opts = git2::PushOptions::default();
        push_opts.remote_callbacks(remote_cb);
        let mut remote = self.repo.find_remote(remote).unwrap();
        remote.push(&[NOTES_REF], Some(&mut push_opts)).unwrap();
        Ok(())
    }

    fn get_metrics(&self, target: &str) -> Result<Vec<Metric>, Error> {
        let rev_id = self.revision_id(target)?;

        let Ok(note) = self.repo.find_note(NOTES_REF_OPTS, rev_id) else {
            return Ok(Default::default());
        };

        note.message()
            .map(|msg| toml::from_str::<Note>(msg).map_err(Error::unable_to_decode))
            .unwrap_or(Ok(Note::default()))
            .map(|res| res.metrics)
    }

    fn set_metrics(&self, target: &str, metrics: Vec<Metric>) -> Result<(), Error> {
        let head_id = self.revision_id(target)?;
        let sig = self.signature()?;

        let note = toml::to_string_pretty(&Note { metrics }).map_err(Error::unable_to_encode)?;
        self.repo
            .note(&sig, &sig, NOTES_REF_OPTS, head_id, &note, true)
            .map_err(Error::unable_to_persist)?;

        Ok(())
    }
}
