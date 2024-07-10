use std::io::Write;

use crate::cmd::format::text::{TextMetricHeader, TextPercent};
use crate::config::Rule;
use crate::entity::check::{CheckList, MetricCheck, RuleCheck, Status};
use crate::entity::difference::{Comparison, Delta};

const TAB: &str = "    ";

struct TextStatus<'a>(pub &'a Status);

impl<'a> std::fmt::Display for TextStatus<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Status::Failed => write!(f, "[FAILURE]"),
            Status::Skip => write!(f, "[SKIP]"),
            Status::Success => write!(f, "[SUCCESS]"),
        }
    }
}

struct SmallTextStatus<'a>(pub &'a Status);

impl<'a> std::fmt::Display for SmallTextStatus<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Status::Failed => write!(f, "failed"),
            Status::Skip => write!(f, "skip"),
            Status::Success => write!(f, "check"),
        }
    }
}

struct TextRule<'a>(pub &'a Rule);

impl<'a> std::fmt::Display for TextRule<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Rule::Max { value } => write!(f, "should be lower than {value:.1}"),
            Rule::Min { value } => write!(f, "should be greater than {value:.1}"),
            Rule::MaxIncrease { ratio } => {
                write!(f, "increase should be less than {:.1}%", *ratio * 100.0)
            }
            Rule::MaxDecrease { ratio } => {
                write!(f, "decrease should be less than {:.1}%", *ratio * 100.0)
            }
        }
    }
}

struct TextComparison<'a>(pub &'a Comparison);

impl<'a> std::fmt::Display for TextComparison<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Comparison::Created { current } => write!(f, "{current:.1} (new)"),
            Comparison::Missing { previous } => write!(f, "{previous:.1} (old)"),
            Comparison::Matching {
                previous,
                current,
                delta:
                    Delta {
                        absolute,
                        relative: _,
                    },
            } if *absolute == 0.0 => {
                write!(f, "{previous:.1} => {current:.1}")
            }
            Comparison::Matching {
                previous,
                current,
                delta:
                    Delta {
                        absolute,
                        relative: Some(relative),
                    },
            } => {
                write!(
                    f,
                    "{previous:.1} => {current:.1} Δ {absolute:+.1} ({})",
                    TextPercent(*relative)
                )
            }
            Comparison::Matching {
                previous,
                current,
                delta:
                    Delta {
                        absolute,
                        relative: None,
                    },
            } => {
                write!(f, "{previous:.1} => {current:.1} Δ {absolute:+.1}",)
            }
        }
    }
}

#[derive(Default)]
pub(crate) struct FormatOptions {
    show_success_rules: bool,
    show_skipped_rules: bool,
}

#[derive(Default)]
pub(super) struct TextFormatter(FormatOptions);

impl TextFormatter {
    fn format_check<W: Write>(&self, check: &RuleCheck, stdout: &mut W) -> std::io::Result<()> {
        match check.status {
            Status::Success if !self.0.show_success_rules => Ok(()),
            Status::Skip if !self.0.show_skipped_rules => Ok(()),
            _ => writeln!(
                stdout,
                "{TAB}{} ... {}",
                TextRule(&check.rule),
                SmallTextStatus(&check.status),
            ),
        }
    }

    fn format_metric<W: Write>(&self, item: &MetricCheck, stdout: &mut W) -> std::io::Result<()> {
        writeln!(
            stdout,
            "{} {} {}",
            TextStatus(&item.status.status()),
            TextMetricHeader(&item.diff.header),
            TextComparison(&item.diff.comparison)
        )?;
        for check in item.checks.iter() {
            self.format_check(check, stdout)?;
        }
        for check in item
            .subsets
            .iter()
            .flat_map(|(_name, subset)| subset.checks.iter())
        {
            self.format_check(check, stdout)?;
        }
        Ok(())
    }

    pub(crate) fn format<W: Write>(&self, res: &CheckList, stdout: &mut W) -> std::io::Result<()> {
        for entry in res.list.iter() {
            self.format_metric(entry, stdout)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Rule;
    use crate::entity::check::SubsetCheck;
    use crate::entity::difference::{Comparison, MetricDiff};
    use crate::entity::metric::MetricHeader;

    fn complete_checklist() -> CheckList {
        CheckList::default()
            .with_check(
                MetricCheck::new(MetricDiff::new(
                    MetricHeader::new("first")
                        .with_tag("platform.os", "linux")
                        .with_tag("platform.arch", "amd64")
                        .with_tag("unit", "byte"),
                    Comparison::matching(10.0, 20.0),
                ))
                .with_check(Rule::max(30.0), Status::Success)
                .with_subset(
                    "show_not_increase_too_much",
                    SubsetCheck::default()
                        .with_matching("platform.os", "linux")
                        .with_check(Rule::max_increase(0.2), Status::Failed),
                ),
            )
            .with_check(
                MetricCheck::new(MetricDiff::new(
                    MetricHeader::new("first")
                        .with_tag("platform.os", "linux")
                        .with_tag("platform.arch", "arm64")
                        .with_tag("unit", "byte"),
                    Comparison::matching(10.0, 11.0),
                ))
                .with_check(Rule::max(30.0), Status::Success)
                .with_subset(
                    "show_not_increase_too_much",
                    SubsetCheck::default()
                        .with_matching("platform.os", "linux")
                        .with_check(Rule::max_increase(0.2), Status::Success),
                ),
            )
            // metric not known in config
            .with_check(MetricCheck::new(MetricDiff::new(
                MetricHeader::new("unknown"),
                Comparison::matching(42.0, 28.0),
            )))
            // metric without general rule
            .with_check(
                MetricCheck::new(MetricDiff::new(
                    MetricHeader::new("noglobal"),
                    Comparison::matching(42.0, 28.0),
                ))
                .with_subset(
                    "show_pass",
                    SubsetCheck::default()
                        .with_matching("foo", "bar")
                        .with_check(Rule::max_increase(0.2), Status::Skip),
                ),
            )
            // metric that doesn't change
            .with_check(
                MetricCheck::new(MetricDiff::new(
                    MetricHeader::new("nochange"),
                    Comparison::matching(10.0, 10.0),
                ))
                .with_check(Rule::max(30.0), Status::Success),
            )
    }

    #[test]
    fn should_format_to_text_by_default() {
        let formatter = TextFormatter(FormatOptions::default());
        let list = complete_checklist();
        let mut stdout: Vec<u8> = Vec::new();
        formatter.format(&list, &mut stdout).unwrap();
        let stdout = String::from_utf8_lossy(&stdout);
        similar_asserts::assert_eq!(stdout, include_str!("./format_text_by_default.txt"));
    }

    #[test]
    fn should_format_to_text_with_success_showed() {
        let formatter = TextFormatter(FormatOptions {
            show_success_rules: true,
            show_skipped_rules: true,
        });
        let list = complete_checklist();
        let mut stdout: Vec<u8> = Vec::new();
        formatter.format(&list, &mut stdout).unwrap();
        let stdout = String::from_utf8_lossy(&stdout);
        similar_asserts::assert_eq!(
            stdout,
            include_str!("./format_text_with_success_showed.txt")
        );
    }
}
