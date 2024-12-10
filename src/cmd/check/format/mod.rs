use crate::entity::check::Status;

mod html;
pub mod markdown;
pub mod text;

#[derive(Default)]
pub(crate) struct Params {
    pub show_success_rules: bool,
    pub show_skipped_rules: bool,
}

impl Status {
    const fn big_label(&self) -> &'static str {
        match self {
            Status::Failed => "[FAILURE]",
            Status::Skip => "[SKIP]",
            Status::Success => "[SUCCESS]",
        }
    }

    fn style(&self) -> nu_ansi_term::Style {
        match self {
            Status::Failed => nu_ansi_term::Style::new()
                .bold()
                .fg(nu_ansi_term::Color::Red),
            Status::Skip => nu_ansi_term::Style::new()
                .italic()
                .fg(nu_ansi_term::Color::LightGray),
            Status::Success => nu_ansi_term::Style::new()
                .bold()
                .fg(nu_ansi_term::Color::Green),
        }
    }

    const fn small_label(&self) -> &'static str {
        match self {
            Status::Failed => "failed",
            Status::Skip => "skip",
            Status::Success => "check",
        }
    }
}
