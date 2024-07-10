// TODO extract the display out of service

use std::io::Write;

use crate::{backend::Backend, cmd::format::text::TextMetric};

#[derive(Debug)]
pub(crate) struct Options {
    pub target: String,
    pub hide_empty: bool,
}

impl<B: Backend> super::Service<B> {
    pub(crate) fn log<Out: Write>(
        &self,
        stdout: &mut Out,
        opts: &Options,
    ) -> Result<(), super::Error> {
        let commits = self.backend.get_commits(&opts.target)?;
        for commit in commits.iter() {
            let metrics = self.get_metrics(&commit.sha, "origin")?;
            if opts.hide_empty && metrics.is_empty() {
                continue;
            }
            writeln!(stdout, "* {} {}", &commit.sha.as_str()[..7], commit.summary)?;
            for metric in metrics.into_metric_iter() {
                writeln!(stdout, "\t{}", TextMetric(&metric))?;
            }
        }
        Ok(())
    }
}
