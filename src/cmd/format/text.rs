use std::fmt::{Display, Write};

use indexmap::IndexMap;

use crate::entity::metric::{Metric, MetricHeader};

pub(crate) struct TextPercent(pub f64);

impl Display for TextPercent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:+.1} %", self.0 * 100.0)
    }
}

pub(crate) struct TextMetricTags<'a>(pub &'a IndexMap<String, String>);

impl<'a> Display for TextMetricTags<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.0.is_empty() {
            f.write_char('{')?;
            for (index, (key, value)) in self.0.iter().enumerate() {
                if index > 0 {
                    f.write_str(", ")?;
                }
                write!(f, "{key}={value:?}")?;
            }
            f.write_char('}')?;
        }
        Ok(())
    }
}

pub(crate) struct TextMetricHeader<'a>(pub &'a MetricHeader);

impl<'a> Display for TextMetricHeader<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.name.as_str())?;
        TextMetricTags(&self.0.tags).fmt(f)?;
        Ok(())
    }
}

pub(crate) struct TextMetric<'a>(pub &'a Metric);

impl<'a> Display for TextMetric<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {:?}", TextMetricHeader(&self.0.header), self.0.value)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn should_display_metric_with_single_tag() {
        let item = super::Metric::new("name", 12.34).with_tag("foo", "bar");
        assert_eq!(
            super::TextMetric(&item).to_string(),
            "name{foo=\"bar\"} 12.34"
        );
    }

    #[test]
    fn should_display_metric_with_multiple_tags() {
        let item = super::Metric::new("name", 12.34)
            .with_tag("foo", "bar")
            .with_tag("ab", "cd");
        assert_eq!(
            super::TextMetric(&item).to_string(),
            "name{foo=\"bar\", ab=\"cd\"} 12.34"
        );
    }

    #[test]
    fn should_display_metric_with_empty_tags() {
        let item = super::Metric::new("name", 12.34);
        assert_eq!(super::TextMetric(&item).to_string(), "name 12.34");
    }
}
