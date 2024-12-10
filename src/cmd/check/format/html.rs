use crate::{
    entity::{check::MetricCheck, config::Config, metric::MetricHeader},
    formatter::{metric::TextMetricHeader, percent::TextPercent},
};
use another_html_builder::{prelude::WriterExt, Body, Buffer};

fn empty<W: WriterExt>(buf: Buffer<W, Body<'_>>) -> Buffer<W, Body<'_>> {
    buf
}

fn text<W: WriterExt>(
    value: &'static str,
) -> impl FnOnce(Buffer<W, Body<'_>>) -> Buffer<W, Body<'_>> {
    |buf: Buffer<W, Body<'_>>| buf.text(value)
}

fn write_metric_header<'a, W: WriterExt>(
    header: &MetricHeader,
    buf: Buffer<W, Body<'a>>,
) -> Buffer<W, Body<'a>> {
    buf.text(&header.name)
}

fn write_thead<'a, W: WriterExt>(buf: Buffer<W, Body<'a>>) -> Buffer<W, Body<'a>> {
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

pub(super) struct MetricCheckTable<'a> {
    config: &'a Config,
    values: &'a [MetricCheck],
}

impl<'e> MetricCheckTable<'e> {
    pub fn new(config: &'e Config, values: &'e [MetricCheck]) -> Self {
        Self { config, values }
    }

    fn write_metric_check<'a, W: WriterExt>(
        &self,
        buf: Buffer<W, Body<'a>>,
        check: &MetricCheck,
    ) -> Buffer<W, Body<'a>> {
        let formatter = self.config.formatter(&check.diff.header.name);

        buf.node("tr").content(|buf| {
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
                            buf.raw(" (")
                                .raw(TextPercent::new(rel).with_sign(true))
                                .raw(")")
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
