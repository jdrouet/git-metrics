pub mod text;

pub const NO_SCALE: human_number::Scales<'static> = human_number::Scales::new(&[], &[]);

#[inline]
pub fn undefined_unit_formatter() -> human_number::Formatter<'static> {
    human_number::Formatter::new(NO_SCALE, human_number::Options::default())
}
