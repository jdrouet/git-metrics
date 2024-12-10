use crate::entity::check::CheckList;
use crate::entity::config::Config;

#[derive(Default)]
pub struct MarkdownFormatter {
    pub show_success_rules: bool,
    pub show_skipped_rules: bool,
}

impl MarkdownFormatter {
    pub fn format<W: std::io::Write>(
        &self,
        res: &CheckList,
        config: &Config,
        stdout: W,
    ) -> std::io::Result<W> {
        Ok(super::html::MetricCheckTable::new(config, &res.list).render(stdout))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::prelude::BasicWriter;
    use crate::entity::check::{CheckList, MetricCheck, Status, SubsetCheck};
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
        let markdown_formatter = MarkdownFormatter::default();
        let list = complete_checklist();
        let mut writter = BasicWriter::from(Vec::<u8>::new());
        markdown_formatter
            .format(&list, &config, &mut writter)
            .unwrap();
        let stdout = writter.into_string();
        similar_asserts::assert_eq!(stdout, include_str!("./format_md_by_default.md"));
    }

    #[test]
    fn should_format_to_text_with_success_showed() {
        let config = Config::default().with_metric(
            "with-unit",
            MetricConfig::default().with_unit(Unit::binary().with_suffix("B")),
        );
        let formatter = MarkdownFormatter {
            show_success_rules: true,
            show_skipped_rules: true,
        };
        let list = complete_checklist();
        let mut writter = BasicWriter::from(Vec::<u8>::new());
        formatter.format(&list, &config, &mut writter).unwrap();
        let stdout = writter.into_string();
        similar_asserts::assert_eq!(stdout, include_str!("./format_md_with_success_showed.md"));
    }
}
