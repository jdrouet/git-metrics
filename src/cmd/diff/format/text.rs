use human_number::Formatter;

use crate::cmd::format::text::PrettyTextMetricHeader;
use crate::cmd::prelude::PrettyWriter;
use crate::entity::config::Config;
use crate::entity::difference::{Comparison, MetricDiff, MetricDiffList};
use crate::formatter::percent::TextPercent;

pub struct TextFormatter<'a>(pub &'a super::Params);

impl TextFormatter<'_> {
    fn format_entry<W: PrettyWriter>(
        &self,
        entry: &MetricDiff,
        formatter: Formatter,
        stdout: &mut W,
    ) -> std::io::Result<()> {
        match &entry.comparison {
            Comparison::Created { current } => {
                stdout.write_str("+ ")?;
                stdout.write_element(PrettyTextMetricHeader::new(&entry.header))?;
                writeln!(stdout, " {}", formatter.format(*current))
            }
            Comparison::Missing { previous } if self.0.show_previous => {
                stdout.write_str("  ")?;
                stdout.write_element(PrettyTextMetricHeader::new(&entry.header))?;
                writeln!(stdout, " {}", formatter.format(*previous))
            }
            Comparison::Matching {
                previous,
                current,
                delta: _,
            } if previous == current => {
                stdout.write_str("= ")?;
                stdout.write_element(PrettyTextMetricHeader::new(&entry.header))?;
                writeln!(stdout, " {}", formatter.format(*current))
            }
            Comparison::Matching {
                previous,
                current,
                delta,
            } => {
                stdout.write_str("- ")?;
                stdout.write_element(PrettyTextMetricHeader::new(&entry.header))?;
                writeln!(stdout, " {}", formatter.format(*previous))?;
                stdout.write_str("+ ")?;
                stdout.write_element(PrettyTextMetricHeader::new(&entry.header))?;
                write!(stdout, " {}", formatter.format(*current))?;
                if let Some(relative) = delta.relative {
                    stdout.write_str(" (")?;
                    stdout.write_element(TextPercent::new(relative).with_sign(true))?;
                    stdout.write_str(")")?;
                }
                writeln!(stdout)
            }
            _ => Ok(()),
        }
    }

    pub fn format<W: PrettyWriter>(
        &self,
        list: &MetricDiffList,
        config: &Config,
        mut stdout: W,
    ) -> std::io::Result<W> {
        for entry in list.inner().iter() {
            let formatter: Formatter = config.formatter(entry.header.name.as_str());
            self.format_entry(entry, formatter, &mut stdout)?;
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
        let writer = BasicWriter::from(Vec::<u8>::new());
        let config = Config::default();
        let writer = super::TextFormatter(&Params {
            show_previous: true,
        })
        .format(&list, &config, writer)
        .unwrap();
        let stdout = writer.into_string();
        similar_asserts::assert_eq!(
            stdout,
            r#"+ first 10.00
- second 10.00
+ second 12.00 (+20.00 %)
  third 10.00
"#
        );
    }
}
