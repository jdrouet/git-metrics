use human_number::Formatter;

use crate::cmd::prelude::{PrettyDisplay, PrettyWriter};
use crate::entity::metric::{Metric, MetricHeader};
use crate::formatter::metric::TextMetricTags;

pub const TAB: &str = "    ";

pub struct TextMetricHeader<'a> {
    value: &'a MetricHeader,
}

impl<'a> TextMetricHeader<'a> {
    #[inline]
    pub const fn new(value: &'a MetricHeader) -> Self {
        Self { value }
    }
}

impl PrettyDisplay for TextMetricHeader<'_> {
    fn print<W: PrettyWriter>(&self, writer: &mut W) -> std::io::Result<()> {
        let style = nu_ansi_term::Style::new().bold();
        writer.set_style(style.prefix())?;
        writer.write_str(self.value.name.as_str())?;
        writer.set_style(style.suffix())?;
        TextMetricTags::new(&self.value.tags).print(writer)
    }
}

pub struct TextMetric<'a> {
    value: &'a Metric,
    formatter: &'a Formatter<'a>,
}

impl<'a> TextMetric<'a> {
    #[inline]
    pub const fn new(formatter: &'a Formatter<'a>, value: &'a Metric) -> Self {
        Self { value, formatter }
    }
}

impl PrettyDisplay for TextMetric<'_> {
    fn print<W: PrettyWriter>(&self, writer: &mut W) -> std::io::Result<()> {
        TextMetricHeader::new(&self.value.header).print(writer)?;
        write!(writer, " {}", self.formatter.format(self.value.value))
    }
}

#[cfg(test)]
mod tests {
    use human_number::Formatter;

    use crate::cmd::prelude::PrettyDisplay;

    #[test]
    fn should_display_metric_with_single_tag() {
        let item = super::Metric::new("name", 12.34).with_tag("foo", "bar");
        let formatter = Formatter::si();
        assert_eq!(
            super::TextMetric::new(&formatter, &item)
                .to_basic_string()
                .unwrap(),
            "name{foo=\"bar\"} 12.34"
        );
    }

    #[test]
    fn should_display_metric_with_multiple_tags() {
        let formatter = Formatter::si();
        let item = super::Metric::new("name", 12.34)
            .with_tag("foo", "bar")
            .with_tag("ab", "cd");
        assert_eq!(
            super::TextMetric::new(&formatter, &item)
                .to_basic_string()
                .unwrap(),
            "name{foo=\"bar\", ab=\"cd\"} 12.34"
        );
    }

    #[test]
    fn should_display_metric_with_empty_tags() {
        let formatter = Formatter::si();
        let item = super::Metric::new("name", 12.34);
        assert_eq!(
            super::TextMetric::new(&formatter, &item)
                .to_basic_string()
                .unwrap(),
            "name 12.34"
        );
    }
}
