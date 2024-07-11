use crate::cmd::format::text::{TextMetricHeader, TextPercent};
use crate::cmd::prelude::PrettyWriter;
use crate::entity::difference::{Comparison, MetricDiff, MetricDiffList};

pub struct TextFormatter {
    pub show_previous: bool,
}

impl TextFormatter {
    fn format_entry<W: PrettyWriter>(
        &self,
        entry: &MetricDiff,
        stdout: &mut W,
    ) -> std::io::Result<()> {
        match &entry.comparison {
            Comparison::Created { current } => {
                stdout.write_str("+ ")?;
                stdout.write_element(TextMetricHeader(&entry.header))?;
                writeln!(stdout, "{:.1}", current)
            }
            Comparison::Missing { previous } if self.show_previous => {
                stdout.write_str("  ")?;
                stdout.write_element(TextMetricHeader(&entry.header))?;
                writeln!(stdout, "{:.1}", previous)
            }
            Comparison::Matching {
                previous,
                current,
                delta: _,
            } if previous == current => {
                stdout.write_str("= ")?;
                stdout.write_element(TextMetricHeader(&entry.header))?;
                writeln!(stdout, "{:.1}", current)
            }
            Comparison::Matching {
                previous,
                current,
                delta,
            } => {
                stdout.write_str("- ")?;
                stdout.write_element(TextMetricHeader(&entry.header))?;
                writeln!(stdout, "{:.1}", previous)?;
                stdout.write_str("+ ")?;
                stdout.write_element(TextMetricHeader(&entry.header))?;
                write!(stdout, "{:.1}", current)?;
                if let Some(relative) = delta.relative {
                    stdout.write_element(TextPercent(relative))?;
                }
                writeln!(stdout)
            }
            _ => Ok(()),
        }
    }

    pub fn format<W: PrettyWriter>(
        &self,
        list: &MetricDiffList,
        stdout: &mut W,
    ) -> std::io::Result<()> {
        for entry in list.inner().iter() {
            self.format_entry(entry, stdout)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
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
        let mut stdout = Vec::new();
        super::TextFormatter {
            show_previous: true,
        }
        .format(&list, &mut stdout)
        .unwrap();
        let stdout = String::from_utf8_lossy(&stdout);
        similar_asserts::assert_eq!(
            stdout,
            r#"+ first 10.0
- second 10.0
+ second 12.0 (+20.0 %)
  third 10.0
"#
        );
    }
}
