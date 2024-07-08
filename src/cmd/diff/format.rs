use std::io::Write;

use crate::service::diff::{Comparison, MetricDiff, MetricDiffList};

pub(super) struct TextFormatter;

impl TextFormatter {
    fn format_entry<W: Write>(entry: &MetricDiff, stdout: &mut W) -> std::io::Result<()> {
        match &entry.comparison {
            Comparison::Created { current } => {
                writeln!(stdout, "+ {} {:.1}", entry.header, current)
            }
            Comparison::Missing { previous } => {
                writeln!(stdout, "  {} {:.1}", entry.header, previous)
            }
            Comparison::Matching {
                previous,
                current,
                delta: _,
            } if previous == current => {
                writeln!(stdout, "= {} {:.1}", entry.header, current)
            }
            Comparison::Matching {
                previous,
                current,
                delta,
            } => {
                writeln!(stdout, "- {} {:.1}", entry.header, previous)?;
                write!(stdout, "+ {} {:.1}", entry.header, current)?;
                if let Some(relative) = delta.relative {
                    write!(stdout, " ({:+.2} %)", relative * 100.0)?;
                }
                writeln!(stdout)
            }
        }
    }

    pub(crate) fn format<W: Write>(list: &MetricDiffList, stdout: &mut W) -> std::io::Result<()> {
        for entry in list.inner().iter() {
            Self::format_entry(entry, stdout)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        entity::MetricHeader,
        service::diff::{Comparison, MetricDiff, MetricDiffList},
    };

    #[test]
    fn should_format_text() {
        let list = MetricDiffList(vec![
            MetricDiff {
                header: MetricHeader {
                    name: "first".into(),
                    tags: Default::default(),
                },
                comparison: Comparison::created(10.0),
            },
            MetricDiff {
                header: MetricHeader {
                    name: "second".into(),
                    tags: Default::default(),
                },
                comparison: Comparison::new(10.0, Some(12.0)),
            },
            MetricDiff {
                header: MetricHeader {
                    name: "third".into(),
                    tags: Default::default(),
                },
                comparison: Comparison::new(10.0, None),
            },
        ]);
        let mut stdout = Vec::new();
        super::TextFormatter::format(&list, &mut stdout).unwrap();
        let stdout = String::from_utf8_lossy(&stdout);
        assert_eq!(
            stdout,
            "+ first 10.0\n- second 10.0\n+ second 12.0 (+20.00 %)\n  third 10.0\n"
        );
    }
}
