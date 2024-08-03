use crate::entity::metric::Metric;

#[derive(Debug)]
pub struct Error(std::io::Error);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

pub trait Importer {
    fn import(self) -> Result<Vec<Metric>, Error>;
}
