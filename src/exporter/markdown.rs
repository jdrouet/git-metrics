use std::path::Path;

use human_number::Formatter;

use crate::entity::check::{CheckList, MetricCheck, RuleCheck, Status};
use crate::entity::config::Config;
use crate::entity::log::LogEntry;
use crate::entity::metric::MetricHeader;
use crate::formatter::difference::LongTextComparison;
use crate::formatter::metric::TextMetricTags;
use crate::formatter::rule::TextRule;

struct MetricCheckTitle<'a>(&'a MetricHeader);

impl std::fmt::Display for MetricCheckTitle<'_> {
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

impl std::fmt::Display for RuleCheckSection<'_> {
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

impl std::fmt::Display for CheckSection<'_> {
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
            writeln!(f, "_No rules defined for this metric. Passing._")?;
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

impl std::fmt::Display for ChecklistSection<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "## Check conclusion")?;
        writeln!(f)?;
        match self.checklist.status.status() {
            Status::Success => {
                writeln!(f, "The current target is successful ✅")?;
            }
            Status::Skip => {
                writeln!(f, "All the elements from the checklist were skipped.")?;
            }
            Status::Failed => {
                writeln!(f, "The current target failed the checklist ⛔️")?;
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

struct LogEntrySection<'a> {
    config: &'a Config,
    entry: &'a LogEntry,
}

impl<'a> LogEntrySection<'a> {
    const fn new(config: &'a Config, entry: &'a LogEntry) -> Self {
        Self { config, entry }
    }
}

impl std::fmt::Display for LogEntrySection<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "- `{}` {}",
            self.entry.commit.short_sha(),
            self.entry.commit.summary
        )?;
        writeln!(f)?;
        if self.entry.metrics.is_empty() {
            writeln!(f, "_There were no metric for this commit._")?;
        } else {
            writeln!(f, r#"```"#)?;
            for metric in self.entry.metrics.iter() {
                let formatter = self.config.formatter(&metric.header.name);
                writeln!(
                    f,
                    "{} {}",
                    MetricCheckTitle(&metric.header),
                    formatter.format(metric.value)
                )?;
            }
            writeln!(f, r#"```"#)?;
        }
        writeln!(f)?;
        Ok(())
    }
}

struct LogHistorySection<'a> {
    config: &'a Config,
    entries: &'a [LogEntry],
}

impl<'a> LogHistorySection<'a> {
    const fn new(config: &'a Config, entries: &'a [LogEntry]) -> Self {
        Self { config, entries }
    }
}

impl std::fmt::Display for LogHistorySection<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "## Log history")?;
        writeln!(f)?;
        for entry in self.entries.iter() {
            LogEntrySection::new(self.config, entry).fmt(f)?;
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

impl std::fmt::Display for MainView<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "# Git metrics report")?;
        writeln!(f)?;
        writeln!(f, "Generated for the target `{}`.", self.payload.target)?;
        writeln!(f)?;
        ChecklistSection::new(self.config, &self.payload.checks).fmt(f)?;
        LogHistorySection::new(self.config, &self.payload.logs).fmt(f)?;
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

#[cfg(test)]
mod tests {
    use crate::{
        entity::{
            check::{CheckList, MetricCheck, Status},
            config::{Config, Rule},
            difference::{Comparison, MetricDiff},
            git::Commit,
            log::LogEntry,
            metric::{Metric, MetricHeader},
        },
        exporter::Payload,
    };

    #[test]
    fn should_display_log_history() {
        let config = Config::default();
        let entries = vec![
            LogEntry {
                commit: Commit {
                    sha: "aaaaaaaaaaaaa".into(),
                    summary: "first commit".into(),
                },
                metrics: Vec::new(),
            },
            LogEntry {
                commit: Commit {
                    sha: "bbbbbbbbbbbbb".into(),
                    summary: "second commit".into(),
                },
                metrics: vec![Metric::new("name", 42.0).with_tag("foo", "bar")],
            },
        ];
        let output = super::LogHistorySection::new(&config, &entries).to_string();
        similar_asserts::assert_eq!(
            r#"## Log history

- `aaaaaaa` first commit

_There were no metric for this commit._

- `bbbbbbb` second commit

```
name{foo="bar"} 42.00
```

"#,
            output
        );
    }

    #[test]
    fn should_display_checklist_with_nothing() {
        let config = Config::default();
        let checklist = CheckList::default();
        let output = super::ChecklistSection::new(&config, &checklist).to_string();
        similar_asserts::assert_eq!(
            r#"## Check conclusion

All the elements from the checklist were skipped.

| Success    | Skipped    | Failed     |
|:----------:|:----------:|:----------:|
|          0 |          0 |          0 |

"#,
            output
        );
    }

    #[test]
    fn should_display_checklist_with_failed_check() {
        let config = Config::default();
        let checklist = CheckList::default()
            .with_check(MetricCheck::new(MetricDiff::new(
                MetricHeader::new("created"),
                Comparison::created(42.0),
            )))
            .with_check(
                MetricCheck::new(MetricDiff::new(
                    MetricHeader::new("diff"),
                    Comparison::matching(21.0, 42.0),
                ))
                .with_check(Rule::max(30.0), Status::Failed),
            );
        let output = super::ChecklistSection::new(&config, &checklist).to_string();
        similar_asserts::assert_eq!(
            r#"## Check conclusion

The current target failed the checklist ⛔️

| Success    | Skipped    | Failed     |
|:----------:|:----------:|:----------:|
|          0 |          0 |          1 |

### ⏭️ `created`

This metric didn't exist before and was created with the value 42.00.

_No rules defined for this metric. Passing._

### ⛔️ `diff`

This metric changed from 21.00 to 42.00, with a difference of +21.00 (+100.00 %).

- ⛔️ should be lower than 30.00

"#,
            output
        );
    }

    #[test]
    fn should_display_checklist_with_success_check() {
        let config = Config::default();
        let checklist = CheckList::default()
            .with_check(MetricCheck::new(MetricDiff::new(
                MetricHeader::new("created"),
                Comparison::created(42.0),
            )))
            .with_check(
                MetricCheck::new(MetricDiff::new(
                    MetricHeader::new("diff"),
                    Comparison::matching(21.0, 42.0),
                ))
                .with_check(Rule::max(50.0), Status::Success),
            );
        let output = super::ChecklistSection::new(&config, &checklist).to_string();
        similar_asserts::assert_eq!(
            r#"## Check conclusion

The current target is successful ✅

| Success    | Skipped    | Failed     |
|:----------:|:----------:|:----------:|
|          1 |          0 |          0 |

### ⏭️ `created`

This metric didn't exist before and was created with the value 42.00.

_No rules defined for this metric. Passing._

### ✅ `diff`

This metric changed from 21.00 to 42.00, with a difference of +21.00 (+100.00 %).

- ✅ should be lower than 50.00

"#,
            output
        );
    }

    #[test]
    fn should_display_main_view() {
        let config = Config::default();
        let checklist = CheckList::default()
            .with_check(MetricCheck::new(MetricDiff::new(
                MetricHeader::new("created"),
                Comparison::created(42.0),
            )))
            .with_check(
                MetricCheck::new(MetricDiff::new(
                    MetricHeader::new("diff"),
                    Comparison::matching(21.0, 42.0),
                ))
                .with_check(Rule::max(50.0), Status::Success),
            );
        let entries = vec![
            LogEntry {
                commit: Commit {
                    sha: "aaaaaaaaaaaaa".into(),
                    summary: "first commit".into(),
                },
                metrics: Vec::new(),
            },
            LogEntry {
                commit: Commit {
                    sha: "bbbbbbbbbbbbb".into(),
                    summary: "second commit".into(),
                },
                metrics: vec![Metric::new("name", 42.0).with_tag("foo", "bar")],
            },
        ];
        let payload = Payload::new("HEAD~4..HEAD".to_string(), checklist, entries);
        let output = super::MainView::new(&config, &payload).to_string();

        assert_eq!(
            r#"# Git metrics report

Generated for the target `HEAD~4..HEAD`.

## Check conclusion

The current target is successful ✅

| Success    | Skipped    | Failed     |
|:----------:|:----------:|:----------:|
|          1 |          0 |          0 |

### ⏭️ `created`

This metric didn't exist before and was created with the value 42.00.

_No rules defined for this metric. Passing._

### ✅ `diff`

This metric changed from 21.00 to 42.00, with a difference of +21.00 (+100.00 %).

- ✅ should be lower than 50.00

## Log history

- `aaaaaaa` first commit

_There were no metric for this commit._

- `bbbbbbb` second commit

```
name{foo="bar"} 42.00
```

"#,
            output
        );
    }
}
