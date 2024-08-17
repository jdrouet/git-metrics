use human_number::Formatter;

use crate::cmd::format::text::{TextMetricHeader, TextMetricTags, TAB};
use crate::cmd::prelude::{PrettyDisplay, PrettyWriter};
use crate::entity::check::{CheckList, MetricCheck, RuleCheck, Status};
use crate::entity::config::{Config, Rule, RuleAbsolute, RuleChange, RuleRelative};
use crate::entity::difference::{Comparison, Delta};
use crate::formatter::percent::TextPercent;

impl Status {
    const fn big_label(&self) -> &'static str {
        match self {
            Status::Failed => "[FAILURE]",
            Status::Skip => "[SKIP]",
            Status::Success => "[SUCCESS]",
        }
    }

    fn style(&self) -> nu_ansi_term::Style {
        match self {
            Status::Failed => nu_ansi_term::Style::new()
                .bold()
                .fg(nu_ansi_term::Color::Red),
            Status::Skip => nu_ansi_term::Style::new()
                .italic()
                .fg(nu_ansi_term::Color::LightGray),
            Status::Success => nu_ansi_term::Style::new()
                .bold()
                .fg(nu_ansi_term::Color::Green),
        }
    }

    const fn small_label(&self) -> &'static str {
        match self {
            Status::Failed => "failed",
            Status::Skip => "skip",
            Status::Success => "check",
        }
    }
}

struct TextStatus {
    value: Status,
}

impl TextStatus {
    #[inline]
    pub const fn new(value: Status) -> Self {
        Self { value }
    }
}

impl PrettyDisplay for TextStatus {
    fn print<W: PrettyWriter>(&self, writer: &mut W) -> std::io::Result<()> {
        let style = self.value.style();
        writer.set_style(style.prefix())?;
        writer.write_str(self.value.big_label())?;
        writer.set_style(style.suffix())?;
        Ok(())
    }
}

struct SmallTextStatus {
    value: Status,
}

impl SmallTextStatus {
    #[inline]
    pub const fn new(value: Status) -> Self {
        Self { value }
    }
}

impl PrettyDisplay for SmallTextStatus {
    fn print<W: PrettyWriter>(&self, writer: &mut W) -> std::io::Result<()> {
        let style = self.value.style();
        writer.set_style(style.prefix())?;
        writer.write_str(self.value.small_label())?;
        writer.set_style(style.suffix())?;
        Ok(())
    }
}

struct TextRule<'a> {
    formatter: &'a Formatter<'a>,
    value: &'a Rule,
}

impl<'a> TextRule<'a> {
    #[inline]
    pub const fn new(formatter: &'a Formatter<'a>, value: &'a Rule) -> Self {
        Self { formatter, value }
    }
}

impl<'a> std::fmt::Display for TextRule<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value {
            Rule::Max(RuleAbsolute { value }) => {
                write!(f, "should be lower than {}", self.formatter.format(*value))
            }
            Rule::Min(RuleAbsolute { value }) => write!(
                f,
                "should be greater than {}",
                self.formatter.format(*value)
            ),
            Rule::MaxIncrease(RuleChange::Relative(RuleRelative { ratio })) => {
                write!(
                    f,
                    "increase should be less than {}",
                    TextPercent::new(*ratio)
                )
            }
            Rule::MaxIncrease(RuleChange::Absolute(RuleAbsolute { value })) => {
                write!(
                    f,
                    "increase should be less than {}",
                    self.formatter.format(*value)
                )
            }
            Rule::MaxDecrease(RuleChange::Relative(RuleRelative { ratio })) => {
                write!(
                    f,
                    "decrease should be less than {}",
                    TextPercent::new(*ratio)
                )
            }
            Rule::MaxDecrease(RuleChange::Absolute(RuleAbsolute { value })) => {
                write!(
                    f,
                    "decrease should be less than {}",
                    self.formatter.format(*value)
                )
            }
        }
    }
}

struct TextComparison<'a> {
    formatter: &'a Formatter<'a>,
    value: &'a Comparison,
}

impl<'a> TextComparison<'a> {
    #[inline]
    pub const fn new(formatter: &'a Formatter<'a>, value: &'a Comparison) -> Self {
        Self { formatter, value }
    }
}

impl<'a> std::fmt::Display for TextComparison<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let diff_formatter = self.formatter.clone().with_force_sign(true);
        match self.value {
            Comparison::Created { current } => {
                write!(f, "{} (new)", self.formatter.format(*current))
            }
            Comparison::Missing { previous } => {
                write!(f, "{} (old)", self.formatter.format(*previous))
            }
            Comparison::Matching {
                previous,
                current,
                delta:
                    Delta {
                        absolute,
                        relative: _,
                    },
            } if *absolute == 0.0 => {
                write!(
                    f,
                    "{} => {}",
                    self.formatter.format(*previous),
                    self.formatter.format(*current)
                )
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
                    "{} => {} Δ {} ({})",
                    self.formatter.format(*previous),
                    self.formatter.format(*current),
                    diff_formatter.format(*absolute),
                    TextPercent::new(*relative).with_sign(true)
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
                write!(
                    f,
                    "{} => {} Δ {}",
                    self.formatter.format(*previous),
                    self.formatter.format(*current),
                    diff_formatter.format(*absolute),
                )
            }
        }
    }
}

#[derive(Default)]
pub struct TextFormatter {
    pub show_success_rules: bool,
    pub show_skipped_rules: bool,
}

impl TextFormatter {
    fn format_check<W: PrettyWriter>(
        &self,
        check: &RuleCheck,
        numeric_formatter: &Formatter<'_>,
        stdout: &mut W,
    ) -> std::io::Result<()> {
        match check.status {
            Status::Success if !self.show_success_rules => Ok(()),
            Status::Skip if !self.show_skipped_rules => Ok(()),
            _ => {
                stdout.write_str(TAB)?;
                stdout.write_element(TextRule::new(numeric_formatter, &check.rule))?;
                stdout.write_str(" ... ")?;
                stdout.write_element(SmallTextStatus::new(check.status))?;
                writeln!(stdout)
            }
        }
    }

    fn format_metric<W: PrettyWriter>(
        &self,
        item: &MetricCheck,
        numeric_formatter: Formatter<'_>,
        stdout: &mut W,
    ) -> std::io::Result<()> {
        stdout.write_element(TextStatus::new(item.status.status()))?;
        stdout.write_str(" ")?;
        stdout.write_element(TextMetricHeader::new(&item.diff.header))?;
        stdout.write_str(" ")?;
        stdout.write_element(TextComparison::new(
            &numeric_formatter,
            &item.diff.comparison,
        ))?;
        stdout.write_str("\n")?;
        for check in item.checks.iter() {
            self.format_check(check, &numeric_formatter, stdout)?;
        }
        let subset_style = nu_ansi_term::Style::new().fg(nu_ansi_term::Color::LightGray);
        for (name, subset) in item.subsets.iter() {
            if subset.status.is_failed()
                || (self.show_skipped_rules && subset.status.neutral > 0)
                || (self.show_success_rules && subset.status.success > 0)
            {
                stdout.set_style(subset_style.prefix())?;
                writeln!(
                    stdout,
                    "{TAB}# {name:?} matching tags {}",
                    TextMetricTags::new(&subset.matching)
                )?;
                stdout.set_style(subset_style.suffix())?;
                for check in subset.checks.iter() {
                    self.format_check(check, &numeric_formatter, stdout)?;
                }
            }
        }
        Ok(())
    }

    pub fn format<W: PrettyWriter>(
        &self,
        res: &CheckList,
        config: &Config,
        stdout: &mut W,
    ) -> std::io::Result<()> {
        for entry in res.list.iter() {
            let formatter: Formatter = config.formatter(entry.diff.header.name.as_str());
            self.format_metric(entry, formatter, stdout)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::prelude::BasicWriter;
    use crate::entity::check::SubsetCheck;
    use crate::entity::config::{MetricConfig, Rule, Unit};
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
                        .with_check(Rule::max_relative_increase(0.2), Status::Failed),
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
                        .with_check(Rule::max_relative_increase(0.2), Status::Success),
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
                        .with_check(Rule::max_relative_increase(0.2), Status::Skip),
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
            // metric that doesn't change
            .with_check(
                MetricCheck::new(MetricDiff::new(
                    MetricHeader::new("with-unit"),
                    Comparison::matching(1024.0 * 1024.0 * 20.0, 1024.0 * 1024.0 * 25.0),
                ))
                .with_check(Rule::max(1024.0 * 1024.0 * 30.0), Status::Success),
            )
            // with absolute change
            .with_check(
                MetricCheck::new(MetricDiff::new(
                    MetricHeader::new("with-change"),
                    Comparison::matching(1024.0 * 1024.0 * 20.0, 1024.0 * 1024.0 * 25.0),
                ))
                .with_check(
                    Rule::max_absolute_increase(1024.0 * 1024.0 * 10.0),
                    Status::Success,
                )
                .with_check(
                    Rule::max_absolute_increase(1024.0 * 1024.0 * 2.0),
                    Status::Failed,
                ),
            )
    }

    #[test]
    fn should_format_to_text_by_default() {
        let config = Config::default().with_metric(
            "with-unit",
            MetricConfig::default().with_unit(Unit::binary().with_suffix("B")),
        );
        let text_formatter = TextFormatter::default();
        let list = complete_checklist();
        let mut writter = BasicWriter::from(Vec::<u8>::new());
        text_formatter.format(&list, &config, &mut writter).unwrap();
        let stdout = writter.into_string();
        similar_asserts::assert_eq!(stdout, include_str!("./format_text_by_default.txt"));
    }

    #[test]
    fn should_format_to_text_with_success_showed() {
        let config = Config::default().with_metric(
            "with-unit",
            MetricConfig::default().with_unit(Unit::binary().with_suffix("B")),
        );
        let formatter = TextFormatter {
            show_success_rules: true,
            show_skipped_rules: true,
        };
        let list = complete_checklist();
        let mut writter = BasicWriter::from(Vec::<u8>::new());
        formatter.format(&list, &config, &mut writter).unwrap();
        let stdout = writter.into_string();
        similar_asserts::assert_eq!(
            stdout,
            include_str!("./format_text_with_success_showed.txt")
        );
    }
}
