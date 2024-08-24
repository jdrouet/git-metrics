use std::path::Path;

use human_number::Formatter;

use crate::entity::check::{CheckList, MetricCheck, RuleCheck, Status};
use crate::entity::config::Config;
use crate::entity::metric::MetricHeader;
use crate::formatter::difference::LongTextComparison;
use crate::formatter::metric::TextMetricTags;
use crate::formatter::rule::TextRule;

impl Status {
    const fn emoji(&self) -> &str {
        match self {
            Status::Success => "✅",
            Status::Skip => "🆗",
            Status::Failed => "⛔️",
        }
    }
}

struct MetricCheckTitle<'a>(&'a MetricHeader);

impl<'a> std::fmt::Display for MetricCheckTitle<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.name)?;
        TextMetricTags::new(&self.0.tags).fmt(f)?;
        Ok(())
    }
}

struct RuleCheckSection<'a> {
    formatter: &'a Formatter<'a>,
    rule: &'a RuleCheck,
}

impl<'a> RuleCheckSection<'a> {
    const fn new(formatter: &'a Formatter<'a>, rule: &'a RuleCheck) -> Self {
        Self { formatter, rule }
    }
}

impl<'a> std::fmt::Display for RuleCheckSection<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "- {} {}",
            self.rule.status.emoji(),
            TextRule::new(self.formatter, &self.rule.rule)
        )?;
        Ok(())
    }
}

struct CheckSection<'a> {
    config: &'a Config,
    check: &'a MetricCheck,
}

impl<'a> CheckSection<'a> {
    const fn new(config: &'a Config, check: &'a MetricCheck) -> Self {
        Self { config, check }
    }
}

impl<'a> std::fmt::Display for CheckSection<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let formatter = self.config.formatter(&self.check.diff.header.name);
        writeln!(
            f,
            "### {} `{}`",
            self.check.status.status().emoji(),
            MetricCheckTitle(&self.check.diff.header),
        )?;
        writeln!(f)?;
        LongTextComparison::new(&formatter, &self.check.diff.comparison).fmt(f)?;
        writeln!(f)?;
        writeln!(f)?;
        if self.check.checks.is_empty() {
            writeln!(f, "No rules defined for this metric. Passing.")?;
        } else {
            for rule in self.check.checks.iter() {
                RuleCheckSection::new(&formatter, rule).fmt(f)?;
            }
        }
        writeln!(f)?;
        Ok(())
    }
}

struct ChecklistSection<'a> {
    config: &'a Config,
    checklist: &'a CheckList,
}

impl<'a> ChecklistSection<'a> {
    const fn new(config: &'a Config, checklist: &'a CheckList) -> Self {
        Self { config, checklist }
    }
}

impl<'a> std::fmt::Display for ChecklistSection<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "## Check conclusion")?;
        writeln!(f)?;
        match self.checklist.status.status() {
            Status::Success => {
                writeln!(f, "The current commit is successful ✅")?;
            }
            Status::Skip => {}
            Status::Failed => {
                writeln!(f, "The current commit failed the checklist ⛔️")?;
            }
        }
        writeln!(f)?;
        writeln!(f, "| Success    | Skipped    | Failed     |")?;
        writeln!(f, "|:----------:|:----------:|:----------:|")?;
        writeln!(
            f,
            "| {: >10} | {: >10} | {: >10} |",
            self.checklist.status.success,
            self.checklist.status.neutral,
            self.checklist.status.failed
        )?;
        writeln!(f)?;
        for check in self.checklist.list.iter() {
            CheckSection::new(self.config, check).fmt(f)?;
        }
        Ok(())
    }
}

struct MainView<'a> {
    config: &'a Config,
    payload: &'a super::Payload,
}

impl<'a> MainView<'a> {
    const fn new(config: &'a Config, payload: &'a super::Payload) -> Self {
        Self { config, payload }
    }
}

impl<'a> std::fmt::Display for MainView<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "# Git metrics report")?;
        writeln!(f)?;
        writeln!(f, "Generated for the target `{}`.", self.payload.target)?;
        writeln!(f)?;
        ChecklistSection::new(self.config, &self.payload.checks).fmt(f)?;
        Ok(())
    }
}

pub(crate) fn to_file(
    path: &Path,
    config: &Config,
    payload: &super::Payload,
) -> Result<(), super::Error> {
    let mut file = super::with_file(path)?;
    to_writer(&mut file, config, payload)?;
    Ok(())
}

pub(crate) fn to_writer<W: std::io::Write>(
    mut output: W,
    config: &Config,
    payload: &super::Payload,
) -> Result<(), super::Error> {
    write!(output, "{}", MainView::new(config, payload))?;
    Ok(())
}
