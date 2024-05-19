mod cmd;
mod metric;
mod repository;

use clap::Parser;
use cmd::Executor;

/// Git extension in order to attach metrics to commits
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Allows to use git as an external fallback when command fails.
    #[clap(long, default_value = "true")]
    fallback_git: bool,
    /// Enables verbosity
    #[clap(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    #[command(subcommand)]
    command: cmd::Command,
}

impl Args {
    fn log_level(&self) -> Option<tracing::Level> {
        match self.verbose {
            0 => None,
            1 => Some(tracing::Level::ERROR),
            2 => Some(tracing::Level::WARN),
            3 => Some(tracing::Level::INFO),
            4 => Some(tracing::Level::DEBUG),
            _ => Some(tracing::Level::TRACE),
        }
    }
}

fn main() {
    let args = Args::parse();

    if let Some(level) = args.log_level() {
        tracing_subscriber::fmt().with_max_level(level).init();
    }

    let repo = crate::repository::GitRepository::from_env()
        .unwrap()
        .with_fallback(args.fallback_git);

    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();

    if let Err(err) = args.command.execute(repo, &mut stdout, &mut stderr) {
        eprintln!("{err:?}");
        std::process::exit(1);
    }
}
