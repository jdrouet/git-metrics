pub(crate) struct TextPercent {
    value: f64,
    sign: bool,
}

impl TextPercent {
    #[inline]
    pub(crate) const fn new(value: f64) -> Self {
        Self { value, sign: false }
    }

    #[inline]
    pub(crate) const fn with_sign(mut self, sign: bool) -> Self {
        self.sign = sign;
        self
    }
}

impl std::fmt::Display for TextPercent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.sign {
            write!(f, "{:+.2} %", self.value * 100.0)
        } else {
            write!(f, "{:.2} %", self.value * 100.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case::test_case(0.1234, false, "12.34 %"; "positive without sign")]
    #[test_case::test_case(0.1234, true, "+12.34 %"; "positive with sign")]
    #[test_case::test_case(-0.1234, false, "-12.34 %"; "negative without forcing sign")]
    #[test_case::test_case(-0.1234, true, "-12.34 %"; "negative with sign")]
    fn format(value: f64, sign: bool, expected: &str) {
        assert_eq!(
            expected,
            TextPercent::new(value).with_sign(sign).to_string()
        );
    }
}
