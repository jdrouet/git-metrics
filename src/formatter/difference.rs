use human_number::Formatter;

use super::percent::TextPercent;
use crate::entity::difference::{Comparison, Delta};

pub(crate) struct TextDelta<'a> {
    formatter: &'a Formatter<'a>,
    value: &'a Delta,
}

impl<'a> TextDelta<'a> {
    pub fn new(formatter: &'a Formatter<'a>, value: &'a Delta) -> Self {
        Self { formatter, value }
    }
}

impl std::fmt::Display for TextDelta<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value.relative {
            Some(relative) => write!(
                f,
                "{} ({})",
                self.formatter.format(self.value.absolute),
                TextPercent::new(relative).with_sign(true)
            ),
            None => self.formatter.format(self.value.absolute).fmt(f),
        }
    }
}

pub(crate) struct ShortTextComparison<'a> {
    formatter: &'a Formatter<'a>,
    value: &'a Comparison,
}

impl<'a> ShortTextComparison<'a> {
    #[inline]
    pub const fn new(formatter: &'a Formatter<'a>, value: &'a Comparison) -> Self {
        Self { formatter, value }
    }
}

impl std::fmt::Display for ShortTextComparison<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let diff_formatter = self.formatter.clone().with_force_sign(true);
        match self.value {
            Comparison::Created { current } => {
                write!(f, "{} (new)", self.formatter.format(*current))
            }
            Comparison::Missing { previous } => {
                write!(f, "{} (old)", self.formatter.format(*previous))
            }
            Comparison::Matching {
                previous,
                current,
                delta:
                    Delta {
                        absolute,
                        relative: _,
                    },
            } if *absolute == 0.0 => {
                write!(
                    f,
                    "{} => {}",
                    self.formatter.format(*previous),
                    self.formatter.format(*current)
                )
            }
            Comparison::Matching {
                previous,
                current,
                delta,
            } => {
                write!(
                    f,
                    "{} => {} Δ {}",
                    self.formatter.format(*previous),
                    self.formatter.format(*current),
                    TextDelta::new(&diff_formatter, delta),
                )
            }
        }
    }
}

pub(crate) struct LongTextComparison<'a> {
    formatter: &'a Formatter<'a>,
    value: &'a Comparison,
}

impl<'a> LongTextComparison<'a> {
    #[inline]
    pub const fn new(formatter: &'a Formatter<'a>, value: &'a Comparison) -> Self {
        Self { formatter, value }
    }
}

impl std::fmt::Display for LongTextComparison<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let diff_formatter = self.formatter.clone().with_force_sign(true);
        match self.value {
            Comparison::Created { current } => {
                write!(
                    f,
                    "This metric didn't exist before and was created with the value {}.",
                    self.formatter.format(*current)
                )
            }
            Comparison::Missing { previous } => {
                write!(
                    f,
                    "This metric doesn't exist for the current target. The previous value was {}.",
                    self.formatter.format(*previous)
                )
            }
            Comparison::Matching {
                previous: _,
                current,
                delta:
                    Delta {
                        absolute,
                        relative: _,
                    },
            } if *absolute == 0.0 => {
                write!(
                    f,
                    "This metric didn't change and kept the value of {}.",
                    self.formatter.format(*current)
                )
            }
            Comparison::Matching {
                previous,
                current,
                delta,
            } => {
                write!(
                    f,
                    "This metric changed from {} to {}, with a difference of {}.",
                    self.formatter.format(*previous),
                    self.formatter.format(*current),
                    TextDelta::new(&diff_formatter, delta),
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use human_number::Formatter;

    use super::TextDelta;
    use crate::entity::difference::Delta;

    #[test_case::test_case(10.0, 20.0, "+10.00 B (+100.00 %)"; "with increase")]
    #[test_case::test_case(20.0, 10.0, "-10.00 B (-50.00 %)"; "with decrease")]
    #[test_case::test_case(10.0, 10.0, "+0.00 B (+0.00 %)"; "stable")]
    #[test_case::test_case(100_000_000.0, 100_000_001.0, "+1.00 B (+0.00 %)"; "tiny increase")]
    #[test_case::test_case(0.0, 10.0, "+10.00 B"; "increase from 0")]
    #[test_case::test_case(0.0, -10.0, "-10.00 B"; "decrease from 0")]
    fn format_delta(previous: f64, current: f64, expected: &str) {
        let fmt = Formatter::binary().with_unit("B").with_force_sign(true);
        let delta = Delta::new(previous, current);
        assert_eq!(expected, TextDelta::new(&fmt, &delta).to_string());
    }
}
