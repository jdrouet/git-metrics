#[cfg(test)]
pub(crate) mod tests;

mod cmd;
mod entity;
mod repository;

use std::path::PathBuf;

use clap::Parser;

pub(crate) enum ExitCode {
    Success,
    Failure,
}

impl ExitCode {
    #[cfg(test)]
    fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }

    fn exit(self) {
        std::process::exit(match self {
            Self::Success => 0,
            Self::Failure => 1,
        })
    }
}

#[cfg(not(any(feature = "impl-command", feature = "impl-git2")))]
compile_error!("you need to pick at least one implementation");

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum Backend {
    #[cfg(feature = "impl-command")]
    Command,
    #[cfg(feature = "impl-git2")]
    Git2,
}

/// Git extension in order to attach metrics to commits
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Root directory of the git repository
    #[clap(long)]
    root_dir: Option<PathBuf>,
    #[clap(flatten)]
    auth: cmd::GitCredentials,
    /// Select the backend to use to interact with git.
    ///
    /// If running on the CI, you should use command to avoid authentication failures.
    #[cfg_attr(
        not(feature = "impl-git2"),
        clap(
            short,
            long,
            default_value = "command",
            value_enum,
            env = "GIT_BACKEND"
        )
    )]
    #[cfg_attr(
        feature = "impl-git2",
        clap(short, long, default_value = "git2", value_enum, env = "GIT_BACKEND")
    )]
    backend: Backend,
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

    fn execute<Out: std::io::Write, Err: std::io::Write>(
        self,
        stdout: &mut Out,
        stderr: &mut Err,
    ) -> ExitCode {
        match self.backend {
            #[cfg(feature = "impl-command")]
            Backend::Command => self.command.execute(
                crate::repository::CommandRepository::new(self.root_dir),
                stdout,
                stderr,
            ),
            #[cfg(feature = "impl-git2")]
            Backend::Git2 => self.command.execute(
                crate::repository::GitRepository::new(self.root_dir)
                    .unwrap()
                    .with_credentials(self.auth),
                stdout,
                stderr,
            ),
        }
    }
}

fn main() {
    let args = Args::parse();

    if let Some(level) = args.log_level() {
        tracing_subscriber::fmt().with_max_level(level).init();
    }

    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();

    args.execute(&mut stdout, &mut stderr).exit();
}
