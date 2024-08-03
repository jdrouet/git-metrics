use crate::entity::metric::Metric;

#[cfg(feature = "importer-lcov")]
pub(crate) mod lcov;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io {
        #[from]
        source: std::io::Error,
    },
    #[allow(dead_code)]
    #[error("invalid source file format")]
    InvalidFormat {
        #[source]
        source: Box<dyn std::error::Error>,
    },
}

pub trait Importer {
    fn import(self) -> Result<Vec<Metric>, Error>;
}
