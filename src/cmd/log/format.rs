use human_number::Formatter;

/// The output format should be something like
/// ```
/// * aaaaaa commit_message
///     metric_name{key="value"} 12.34
///     metric_name{key="other"} 23.45
/// ```
use crate::cmd::format::text::TextMetric;
use crate::cmd::format::undefined_unit_formatter;
use crate::cmd::prelude::{Pretty, PrettyDisplay, PrettyWriter};
use crate::entity::config::Config;
use crate::entity::git::Commit;
use crate::entity::metric::{Metric, MetricStack};

const TAB: &str = "    ";

struct TextCommit<'a>(pub &'a Commit);

impl<'a> PrettyDisplay for TextCommit<'a> {
    fn print<W: PrettyWriter>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_str("* ")?;
        Pretty::new(
            nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Yellow),
            &self.0.sha.as_str()[..7],
        )
        .print(writer)?;
        writer.write_str(" ")?;
        writer.write_str(self.0.summary.as_str())?;
        Ok(())
    }
}

#[derive(Default)]
pub struct TextFormatter {
    pub filter_empty: bool,
}

impl TextFormatter {
    fn format_metric<W: PrettyWriter>(
        &self,
        item: &Metric,
        formatter: &Formatter,
        stdout: &mut W,
    ) -> std::io::Result<()> {
        stdout.write_str(TAB)?;
        stdout.write_element(TextMetric::new(formatter, item))?;
        stdout.write_str("\n")?;
        Ok(())
    }

    fn format_commit<W: PrettyWriter>(&self, item: &Commit, writer: &mut W) -> std::io::Result<()> {
        TextCommit(item).print(writer)?;
        writeln!(writer)
    }

    pub(crate) fn format<W: PrettyWriter>(
        &self,
        list: Vec<(Commit, MetricStack)>,
        config: &Config,
        stdout: &mut W,
    ) -> std::io::Result<()> {
        let default_formatter = undefined_unit_formatter();
        for (commit, metrics) in list {
            if metrics.is_empty() && self.filter_empty {
                continue;
            }

            self.format_commit(&commit, stdout)?;
            for metric in metrics.into_metric_iter() {
                let formatter = config
                    .metrics
                    .get(metric.header.name.as_str())
                    .map(|m| m.unit.formater())
                    .unwrap_or_else(|| default_formatter.clone());
                self.format_metric(&metric, &formatter, stdout)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {}
