use human_number::Formatter;

use crate::entity::config::Config;
use crate::entity::difference::{Comparison, MetricDiff, MetricDiffList};
use crate::formatter::difference::TextDelta;
use crate::formatter::metric::TextMetricHeader;

pub struct MarkdownFormatter<'a>(pub &'a super::Params);

impl MarkdownFormatter<'_> {
    fn format_entry<W: std::io::Write>(
        &self,
        entry: &MetricDiff,
        formatter: &Formatter<'_>,
        stdout: &mut W,
    ) -> std::io::Result<()> {
        match &entry.comparison {
            Comparison::Created { current } => {
                writeln!(
                    stdout,
                    "| 🆕 | {} |   | {} |   |",
                    TextMetricHeader::new(&entry.header),
                    formatter.format(*current)
                )
            }
            Comparison::Missing { previous } if self.0.show_previous => {
                writeln!(
                    stdout,
                    "|   | {} | {} |   |   |",
                    TextMetricHeader::new(&entry.header),
                    formatter.format(*previous)
                )
            }
            Comparison::Matching {
                previous,
                current,
                delta: _,
            } if previous == current => {
                writeln!(
                    stdout,
                    "| ➡️ | {} | {} | {} |   |",
                    TextMetricHeader::new(&entry.header),
                    formatter.format(*previous),
                    formatter.format(*current),
                )
            }
            Comparison::Matching {
                previous,
                current,
                delta,
            } => {
                let icon = if delta.absolute > 0.0 {
                    "⬆️"
                } else {
                    "⬇️"
                };
                writeln!(
                    stdout,
                    "| {icon} | {} | {} | {} | {} |",
                    TextMetricHeader::new(&entry.header),
                    formatter.format(*previous),
                    formatter.format(*current),
                    TextDelta::new(formatter, delta)
                )
            }
            _ => Ok(()),
        }
    }

    pub fn format<W: std::io::Write>(
        &self,
        list: &MetricDiffList,
        config: &Config,
        mut stdout: W,
    ) -> std::io::Result<W> {
        writeln!(
            &mut stdout,
            "|     | Metric | Previous value | Current value | Change |"
        )?;
        writeln!(
            &mut stdout,
            "|:---:|:-------|---------------:|--------------:|-------:|"
        )?;
        for entry in list.inner().iter() {
            let formatter: Formatter = config.formatter(entry.header.name.as_str());
            self.format_entry(entry, &formatter, &mut stdout)?;
        }
        Ok(stdout)
    }
}

#[cfg(test)]
mod tests {
    use crate::cmd::diff::format::Params;
    use crate::cmd::prelude::BasicWriter;
    use crate::entity::config::Config;
    use crate::entity::difference::{Comparison, MetricDiff, MetricDiffList};
    use crate::entity::metric::MetricHeader;

    #[test]
    fn should_format_text() {
        let list = MetricDiffList(vec![
            MetricDiff::new(MetricHeader::new("first"), Comparison::created(10.0)),
            MetricDiff::new(
                MetricHeader::new("second"),
                Comparison::new(10.0, Some(12.0)),
            ),
            MetricDiff::new(MetricHeader::new("third"), Comparison::new(10.0, None)),
        ]);
        let mut writer = BasicWriter::from(Vec::<u8>::new());
        let config = Config::default();
        super::MarkdownFormatter(&Params {
            show_previous: true,
        })
        .format(&list, &config, &mut writer)
        .unwrap();
        let stdout = writer.into_string();
        similar_asserts::assert_eq!(
            stdout,
            r#"|     | Metric | Previous value | Current value | Change |
|:---:|:-------|---------------:|--------------:|-------:|
| 🆕 | first |   | 10.00 |   |
| ⬆️ | second | 10.00 | 12.00 | 2.00 (+20.00 %) |
|   | third | 10.00 |   |   |
"#
        );
    }
}
