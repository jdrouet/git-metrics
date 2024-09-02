#[derive(Debug, serde::Serialize)]
#[cfg_attr(test, derive(Clone))]
pub(crate) struct Commit {
    pub sha: String,
    pub summary: String,
}

impl Commit {
    pub fn short_sha(&self) -> &str {
        &self.sha[..7]
    }
}
