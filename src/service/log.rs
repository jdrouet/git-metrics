use crate::backend::Backend;
use crate::entity::git::Commit;
use crate::entity::metric::MetricStack;

#[derive(Debug)]
pub(crate) struct Options<'a> {
    pub remote: &'a str,
    pub target: &'a str,
}

impl<B: Backend> super::Service<B> {
    pub(crate) fn log(&self, opts: &Options) -> Result<Vec<(Commit, MetricStack)>, super::Error> {
        let commits = self.backend.get_commits(opts.target)?;
        let mut result = Vec::with_capacity(commits.len());
        for commit in commits {
            let metrics = self.get_metrics(&commit.sha, opts.remote)?;
            result.push((commit, metrics));
        }
        Ok(result)
    }
}
