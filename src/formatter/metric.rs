use indexmap::IndexMap;

use crate::entity::metric::MetricHeader;

pub struct TextMetricTags<'a> {
    value: &'a IndexMap<String, String>,
}

impl<'a> TextMetricTags<'a> {
    #[inline]
    pub const fn new(value: &'a IndexMap<String, String>) -> Self {
        Self { value }
    }
}

impl std::fmt::Display for TextMetricTags<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.value.is_empty() {
            f.write_str("{")?;
            for (index, (key, value)) in self.value.iter().enumerate() {
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

pub struct TextMetricHeader<'a> {
    value: &'a MetricHeader,
}

impl<'a> TextMetricHeader<'a> {
    #[inline]
    pub const fn new(value: &'a MetricHeader) -> Self {
        Self { value }
    }
}

impl std::fmt::Display for TextMetricHeader<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.name.fmt(f)?;
        TextMetricTags::new(&self.value.tags).fmt(f)
    }
}
