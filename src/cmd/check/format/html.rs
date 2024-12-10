use another_html_builder::prelude::WriterExt;
use another_html_builder::{Body, Buffer};

use crate::entity::check::{MetricCheck, RuleCheck, StatusCount};
use crate::entity::config::Config;
use crate::formatter::metric::TextMetricHeader;
use crate::formatter::percent::TextPercent;
use crate::formatter::rule::TextRule;

fn empty<W: WriterExt>(buf: Buffer<W, Body<'_>>) -> Buffer<W, Body<'_>> {
    buf
}

fn text<W: WriterExt>(
    value: &'static str,
) -> impl FnOnce(Buffer<W, Body<'_>>) -> Buffer<W, Body<'_>> {
    |buf: Buffer<W, Body<'_>>| buf.text(value)
}

fn write_thead<W: WriterExt>(buf: Buffer<W, Body<'_>>) -> Buffer<W, Body<'_>> {
    buf.node("thead").content(|buf| {
        buf.node("tr").content(|buf| {
            buf.node("th")
                .attr(("align", "center"))
                .content(text("Status"))
                .node("th")
                .attr(("align", "left"))
                .content(text("Metric"))
                .node("th")
                .attr(("align", "right"))
                .content(text("Previous value"))
                .node("th")
                .attr(("align", "right"))
                .content(text("Current value"))
                .node("th")
                .attr(("align", "right"))
                .content(text("Change"))
        })
    })
}

fn should_display_detailed(params: &super::Params, status: &StatusCount) -> bool {
    status.failed > 0
        || (status.neutral > 0 && params.show_skipped_rules)
        || (status.success > 0 && params.show_success_rules)
}

pub(super) struct MetricCheckTable<'a> {
    params: &'a super::Params,
    config: &'a Config,
    values: &'a [MetricCheck],
}

impl<'e> MetricCheckTable<'e> {
    pub fn new(params: &'e super::Params, config: &'e Config, values: &'e [MetricCheck]) -> Self {
        Self {
            params,
            config,
            values,
        }
    }

    fn write_rule_check<'a, W: WriterExt>(
        &self,
        buf: Buffer<W, Body<'a>>,
        check: &RuleCheck,
        formatter: &human_number::Formatter<'_>,
    ) -> Buffer<W, Body<'a>> {
        buf.cond(
            check.status.is_failed()
                || (self.params.show_skipped_rules && check.status.is_skip())
                || (self.params.show_success_rules && check.status.is_success()),
            |buf| {
                buf.raw(check.status.emoji())
                    .raw(" ")
                    .raw(TextRule::new(formatter, &check.rule))
                    .node("br")
                    .close()
            },
        )
    }

    fn write_metric_check<'a, W: WriterExt>(
        &self,
        buf: Buffer<W, Body<'a>>,
        check: &MetricCheck,
    ) -> Buffer<W, Body<'a>> {
        let formatter = self.config.formatter(&check.diff.header.name);

        let buf = buf.node("tr").content(|buf| {
            buf.node("td")
                .attr(("align", "center"))
                .content(|buf| buf.raw(check.status.status().emoji()))
                .node("td")
                .attr(("align", "left"))
                .content(|buf| buf.raw(TextMetricHeader::new(&check.diff.header)))
                .node("td")
                .attr(("align", "right"))
                .content(|buf| {
                    buf.optional(check.diff.comparison.previous(), |buf, value| {
                        buf.raw(formatter.format(value))
                    })
                })
                .node("td")
                .attr(("align", "right"))
                .content(|buf| {
                    buf.optional(check.diff.comparison.current(), |buf, value| {
                        buf.raw(formatter.format(value))
                    })
                })
                .node("td")
                .attr(("align", "right"))
                .content(|buf| {
                    buf.optional(check.diff.comparison.delta(), |buf, delta| {
                        let buf = buf.raw(formatter.format(delta.absolute));
                        buf.optional(delta.relative, |buf, rel| {
                            buf.node("br")
                                .close()
                                .raw("(")
                                .raw(TextPercent::new(rel).with_sign(true))
                                .raw(")")
                        })
                    })
                })
        });

        buf.cond(should_display_detailed(self.params, &check.status), |buf| {
            buf.node("tr").content(|buf| {
                buf.node("td")
                    .content(empty)
                    .node("td")
                    .attr(("colspan", "4"))
                    .content(|buf| {
                        let buf = check.checks.iter().fold(buf, |buf, rule_check| {
                            self.write_rule_check(buf, rule_check, &formatter)
                        });
                        check.subsets.iter().fold(buf, |buf, (title, subset)| {
                            buf.cond(
                                should_display_detailed(self.params, &subset.status),
                                |buf| {
                                    let buf = buf
                                        .node("i")
                                        .content(|buf| buf.text(title))
                                        .node("br")
                                        .close();

                                    subset.checks.iter().fold(buf, |buf, rule_check| {
                                        self.write_rule_check(buf, rule_check, &formatter)
                                    })
                                },
                            )
                        })
                    })
            })
        })
    }

    pub fn write<'a, W: WriterExt>(&self, buf: Buffer<W, Body<'a>>) -> Buffer<W, Body<'a>> {
        buf.node("table").content(|buf| {
            let buf = write_thead(buf);
            buf.node("tbody").content(|buf| {
                self.values
                    .iter()
                    .fold(buf, |buf, check| self.write_metric_check(buf, check))
            })
        })
    }

    pub fn render<W: std::io::Write>(&self, writer: W) -> W {
        let buf = Buffer::from(writer);
        let buf = self.write(buf);
        buf.into_inner()
    }
}
