use std::fmt::Display;

mod git2;

use crate::metric::Metric;
pub(crate) use git2::GitRepository;

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
    UnableToPush,
}

#[derive(Debug)]
pub(crate) struct Error {
    kind: ErrorKind,
    source: Box<dyn std::error::Error + 'static>,
}

impl Error {
    #[inline]
    fn new<E: std::error::Error + 'static>(kind: ErrorKind, err: E) -> Self {
        Self {
            kind,
            source: Box::new(err),
        }
    }

    #[inline]
    fn signature_not_found<E: std::error::Error + 'static>(err: E) -> Self {
        Self::new(ErrorKind::SignatureNotFound, err)
    }

    #[inline]
    fn target_not_found<E: std::error::Error + 'static>(err: E) -> Self {
        Self::new(ErrorKind::TargetNotFound, err)
    }

    #[inline]
    fn unable_to_decode<E: std::error::Error + 'static>(err: E) -> Self {
        Self::new(ErrorKind::UnableToDecode, err)
    }

    #[inline]
    fn unable_to_encode<E: std::error::Error + 'static>(err: E) -> Self {
        Self::new(ErrorKind::UnableToEncode, err)
    }

    #[inline]
    fn unable_to_persist<E: std::error::Error + 'static>(err: E) -> Self {
        Self::new(ErrorKind::UnableToPersist, err)
    }

    #[inline]
    fn unable_to_push<E: std::error::Error + 'static>(err: E) -> Self {
        Self::new(ErrorKind::UnableToPush, err)
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
            ErrorKind::UnableToPush => "unable to push metrics",
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
