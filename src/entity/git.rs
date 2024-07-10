#[derive(Debug)]
#[cfg_attr(test, derive(Clone))]
pub(crate) struct Commit {
    pub sha: String,
    pub summary: String,
}
