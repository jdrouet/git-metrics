use crate::backend::Backend;

pub(crate) mod add;
pub(crate) mod log;
pub(crate) mod pull;
pub(crate) mod push;
pub(crate) mod remove;
pub(crate) mod show;

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("unable to write to stdout or stderr")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Backend(#[from] crate::backend::Error),
}

pub(crate) struct Service<B> {
    backend: B,
}

impl<B: Backend> Service<B> {
    pub(crate) fn new(backend: B) -> Self {
        Self { backend }
    }
}
