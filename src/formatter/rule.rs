use human_number::Formatter;

use crate::entity::config::{Rule, RuleAbsolute, RuleChange, RuleRelative};
use crate::formatter::percent::TextPercent;

pub(crate) struct TextRule<'a> {
    formatter: &'a Formatter<'a>,
    value: &'a Rule,
}

impl<'a> TextRule<'a> {
    #[inline]
    pub const fn new(formatter: &'a Formatter<'a>, value: &'a Rule) -> Self {
        Self { formatter, value }
    }
}

impl<'a> std::fmt::Display for TextRule<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value {
            Rule::Max(RuleAbsolute { value }) => {
                write!(f, "should be lower than {}", self.formatter.format(*value))
            }
            Rule::Min(RuleAbsolute { value }) => write!(
                f,
                "should be greater than {}",
                self.formatter.format(*value)
            ),
            Rule::MaxIncrease(RuleChange::Relative(RuleRelative { ratio })) => {
                write!(
                    f,
                    "increase should be less than {}",
                    TextPercent::new(*ratio)
                )
            }
            Rule::MaxIncrease(RuleChange::Absolute(RuleAbsolute { value })) => {
                write!(
                    f,
                    "increase should be less than {}",
                    self.formatter.format(*value)
                )
            }
            Rule::MaxDecrease(RuleChange::Relative(RuleRelative { ratio })) => {
                write!(
                    f,
                    "decrease should be less than {}",
                    TextPercent::new(*ratio)
                )
            }
            Rule::MaxDecrease(RuleChange::Absolute(RuleAbsolute { value })) => {
                write!(
                    f,
                    "decrease should be less than {}",
                    self.formatter.format(*value)
                )
            }
        }
    }
}
