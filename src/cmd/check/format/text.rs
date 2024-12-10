use human_number::Formatter;

use crate::cmd::format::text::{TextMetricHeader, TAB};
use crate::cmd::prelude::{PrettyDisplay, PrettyWriter};
use crate::entity::check::{CheckList, MetricCheck, RuleCheck, Status};
use crate::entity::config::Config;
use crate::formatter::difference::ShortTextComparison;
use crate::formatter::metric::TextMetricTags;
use crate::formatter::rule::TextRule;

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

pub struct TextFormatter<'a> {
    params: &'a super::Params,
}

impl<'a> TextFormatter<'a> {
    pub fn new(params: &'a super::Params) -> Self {
        Self { params }
    }

    fn format_check<W: PrettyWriter>(
        &self,
        check: &RuleCheck,
        numeric_formatter: &Formatter<'_>,
        stdout: &mut W,
    ) -> std::io::Result<()> {
        match check.status {
            Status::Success if !self.params.show_success_rules => Ok(()),
            Status::Skip if !self.params.show_skipped_rules => Ok(()),
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
        stdout.write_element(ShortTextComparison::new(
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
                || (self.params.show_skipped_rules && subset.status.neutral > 0)
                || (self.params.show_success_rules && subset.status.success > 0)
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
        mut stdout: W,
    ) -> std::io::Result<W> {
        for entry in res.list.iter() {
            let formatter: Formatter = config.formatter(entry.diff.header.name.as_str());
            self.format_metric(entry, formatter, &mut stdout)?;
        }
        Ok(stdout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::check::format::Params;
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
        let text_formatter = TextFormatter::new(&Params {
            show_skipped_rules: false,
            show_success_rules: false,
        });
        let list = complete_checklist();
        let writter = BasicWriter::from(Vec::<u8>::new());
        let writter = text_formatter.format(&list, &config, writter).unwrap();
        let stdout = writter.into_string();
        similar_asserts::assert_eq!(stdout, include_str!("./format_text_by_default.txt"));
    }

    #[test]
    fn should_format_to_text_with_success_showed() {
        let config = Config::default().with_metric(
            "with-unit",
            MetricConfig::default().with_unit(Unit::binary().with_suffix("B")),
        );
        let formatter = TextFormatter::new(&Params {
            show_success_rules: true,
            show_skipped_rules: true,
        });
        let list = complete_checklist();
        let writter = BasicWriter::from(Vec::<u8>::new());
        let writter = formatter.format(&list, &config, writter).unwrap();
        let stdout = writter.into_string();
        similar_asserts::assert_eq!(
            stdout,
            include_str!("./format_text_with_success_showed.txt")
        );
    }
}
