use human_number::Formatter;

/// The output format should be something like
/// ```
/// * aaaaaa commit_message
///     metric_name{key="value"} 12.34
///     metric_name{key="other"} 23.45
/// ```
use crate::cmd::format::text::TextMetric;
use crate::cmd::prelude::{PrettyDisplay, PrettyWriter};
use crate::entity::config::Config;
use crate::entity::git::Commit;
use crate::entity::metric::{Metric, MetricStack};

const TAB: &str = "    ";

struct TextCommit<'a> {
    value: &'a Commit,
}

impl<'a> TextCommit<'a> {
    #[inline]
    pub const fn new(value: &'a Commit) -> Self {
        Self { value }
    }
}

impl<'a> PrettyDisplay for TextCommit<'a> {
    fn print<W: PrettyWriter>(&self, writer: &mut W) -> std::io::Result<()> {
        let style = nu_ansi_term::Style::new().fg(nu_ansi_term::Color::Yellow);
        writer.write_str("* ")?;
        writer.set_style(style.prefix())?;
        writer.write_str(self.value.short_sha())?;
        writer.set_style(style.suffix())?;
        writer.write_str(" ")?;
        writer.write_str(self.value.summary.as_str())?;
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
        TextCommit::new(item).print(writer)?;
        writeln!(writer)
    }

    pub(crate) fn format<W: PrettyWriter>(
        &self,
        list: Vec<(Commit, MetricStack)>,
        config: &Config,
        stdout: &mut W,
    ) -> std::io::Result<()> {
        for (commit, metrics) in list {
            if metrics.is_empty() && self.filter_empty {
                continue;
            }

            self.format_commit(&commit, stdout)?;
            for metric in metrics.into_metric_iter() {
                let formatter = config.formatter(metric.header.name.as_str());
                self.format_metric(&metric, &formatter, stdout)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {}
