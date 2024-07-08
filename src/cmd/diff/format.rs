pub(super) mod text {
    use std::io::Write;

    use crate::service::diff::{Comparison, MetricDiff, MetricDiffList};

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
        for entry in list
            .inner()
            .iter()
            .filter(|entry| entry.comparison.has_current())
        {
            format_entry(entry, stdout)?;
        }
        Ok(())
    }
}
