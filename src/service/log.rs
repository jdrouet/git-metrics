use crate::backend::Backend;
use crate::entity::git::Commit;
use crate::entity::metric::MetricStack;

#[derive(Debug)]
pub(crate) struct Options {
    pub target: String,
}

impl<B: Backend> super::Service<B> {
    pub(crate) fn log(&self, opts: &Options) -> Result<Vec<(Commit, MetricStack)>, super::Error> {
        let commits = self.backend.get_commits(&opts.target)?;
        let mut result = Vec::with_capacity(commits.len());
        for commit in commits {
            let metrics = self.get_metrics(&commit.sha, "origin")?;
            result.push((commit, metrics));
        }
        Ok(result)
    }
}
