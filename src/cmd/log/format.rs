use std::io::Write;

use crate::cmd::format::text::TextMetric;
use crate::entity::git::Commit;
use crate::entity::metric::{Metric, MetricStack};

const TAB: &str = "    ";

#[derive(Default)]
pub struct TextFormatter {
    pub filter_empty: bool,
}

impl TextFormatter {
    fn format_metric<W: Write>(&self, item: &Metric, stdout: &mut W) -> std::io::Result<()> {
        writeln!(stdout, "{TAB}{}", TextMetric(item))
    }

    fn format_commit<W: Write>(&self, item: &Commit, stdout: &mut W) -> std::io::Result<()> {
        writeln!(stdout, "* {} {}", &item.sha.as_str()[..7], item.summary)
    }

    pub(crate) fn format<W: Write>(
        &self,
        list: Vec<(Commit, MetricStack)>,
        stdout: &mut W,
    ) -> std::io::Result<()> {
        for (commit, metrics) in list {
            if metrics.is_empty() && self.filter_empty {
                continue;
            }

            self.format_commit(&commit, stdout)?;
            for metric in metrics.into_metric_iter() {
                self.format_metric(&metric, stdout)?;
            }
        }
        Ok(())
    }
}
