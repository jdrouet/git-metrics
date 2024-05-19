mod cmd;
mod metric;
mod repository;

use clap::Parser;
use cmd::Executor;

#[cfg(not(any(feature = "impl-command", feature = "impl-git2")))]
compile_error!("you need to pick at least one implementation");

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum Protocol {
    #[cfg(feature = "impl-command")]
    Command,
    #[cfg(feature = "impl-git2")]
    Git2,
}

/// Git extension in order to attach metrics to commits
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Select the backend to use to interact with git.
    ///
    /// If running on the CI, you should use command to avoid authentication failures.
    #[cfg_attr(
        not(feature = "impl-git2"),
        clap(short, long, default_value = "command", value_enum, env = "PROTOCOL")
    )]
    #[cfg_attr(
        feature = "impl-git2",
        clap(short, long, default_value = "git2", value_enum, env = "PROTOCOL")
    )]
    protocol: Protocol,
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
    ) -> Result<(), crate::cmd::Error> {
        match self.protocol {
            #[cfg(feature = "impl-command")]
            Protocol::Command => {
                self.command
                    .execute(crate::repository::CommandRepository, stdout, stderr)
            }
            #[cfg(feature = "impl-git2")]
            Protocol::Git2 => self.command.execute(
                crate::repository::GitRepository::from_env().unwrap(),
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

    if let Err(err) = args.execute(&mut stdout, &mut stderr) {
        eprintln!("{err:?}");
        std::process::exit(1);
    }
}
