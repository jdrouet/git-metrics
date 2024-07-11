use indexmap::IndexMap;

use crate::cmd::prelude::{PrettyDisplay, PrettyWriter};
use crate::entity::metric::{Metric, MetricHeader};

pub const TAB: &str = "    ";

pub struct TextPercent(pub f64);

impl std::fmt::Display for TextPercent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:+.1} %", self.0 * 100.0)
    }
}

pub struct TextMetricTags<'a>(pub &'a IndexMap<String, String>);

impl<'a> std::fmt::Display for TextMetricTags<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.0.is_empty() {
            f.write_str("{")?;
            for (index, (key, value)) in self.0.iter().enumerate() {
                if index > 0 {
                    f.write_str(", ")?;
                }
                write!(f, "{key}={value:?}")?;
            }
            f.write_str("}")?;
        }
        Ok(())
    }
}

pub struct TextMetricHeader<'a>(pub &'a MetricHeader);

impl<'a> PrettyDisplay for TextMetricHeader<'a> {
    fn print<W: PrettyWriter>(&self, writer: &mut W) -> std::io::Result<()> {
        write!(
            writer,
            "{}{}",
            self.0.name.as_str(),
            TextMetricTags(&self.0.tags)
        )
    }
}

pub struct TextMetric<'a>(pub &'a Metric);

impl<'a> PrettyDisplay for TextMetric<'a> {
    fn print<W: PrettyWriter>(&self, writer: &mut W) -> std::io::Result<()> {
        TextMetricHeader(&self.0.header).print(writer)?;
        write!(writer, " {:?}", self.0.value)
    }
}

#[cfg(test)]
mod tests {
    use crate::cmd::prelude::PrettyDisplay;

    #[test]
    fn should_display_metric_with_single_tag() {
        let item = super::Metric::new("name", 12.34).with_tag("foo", "bar");
        assert_eq!(
            super::TextMetric(&item).to_basic_string().unwrap(),
            "name{foo=\"bar\"} 12.34"
        );
    }

    #[test]
    fn should_display_metric_with_multiple_tags() {
        let item = super::Metric::new("name", 12.34)
            .with_tag("foo", "bar")
            .with_tag("ab", "cd");
        assert_eq!(
            super::TextMetric(&item).to_basic_string().unwrap(),
            "name{foo=\"bar\", ab=\"cd\"} 12.34"
        );
    }

    #[test]
    fn should_display_metric_with_empty_tags() {
        let item = super::Metric::new("name", 12.34);
        assert_eq!(
            super::TextMetric(&item).to_basic_string().unwrap(),
            "name 12.34"
        );
    }
}
