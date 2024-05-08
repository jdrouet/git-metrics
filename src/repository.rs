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
    fn get_metrics(&self, target: &str) -> Result<Vec<Metric>, Error>;
    fn set_metrics(&self, target: &str, metrics: Vec<Metric>) -> Result<(), Error>;
}

const NOTES_REF: Option<&str> = Some("refs/notes/metrics");

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

impl Repository for GitRepository {
    fn get_metrics(&self, target: &str) -> Result<Vec<Metric>, Error> {
        let rev_id = self.revision_id(target)?;

        let Ok(note) = self.repo.find_note(NOTES_REF, rev_id) else {
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
            .note(&sig, &sig, NOTES_REF, head_id, &note, true)
            .map_err(Error::unable_to_persist)?;

        Ok(())
    }
}
