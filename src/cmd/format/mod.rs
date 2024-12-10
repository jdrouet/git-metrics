#[derive(clap::ValueEnum, Clone, Copy, Debug, Default)]
pub enum Format {
    #[default]
    Text,
    Markdown,
}

pub mod text;
