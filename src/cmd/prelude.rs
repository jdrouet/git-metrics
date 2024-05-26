use std::str::FromStr;

#[derive(Clone, Debug)]
pub(super) struct Tag {
    pub name: String,
    pub value: String,
}

impl FromStr for Tag {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split_once(':')
            .map(|(name, value)| Tag {
                name: name.trim().to_string(),
                value: value.trim().to_string(),
            })
            .ok_or_else(|| "unable to decode tag name and value")
    }
}
