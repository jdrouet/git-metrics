use crate::{repository::Repository, ExitCode};
use std::io::Write;

pub(crate) mod add;
pub(crate) mod log;
pub(crate) mod pull;
pub(crate) mod push;
pub(crate) mod remove;
pub(crate) mod show;

mod prelude;

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("unable to write to stdout or stderr")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Repository(#[from] crate::repository::Error),
}

pub(crate) trait Executor {
    fn execute<Repo: Repository, Out: Write, Err: Write>(
        self,
        repo: Repo,
        stdout: &mut Out,
        stderr: &mut Err,
    ) -> Result<(), Error>;
}

#[derive(Debug, clap::Subcommand)]
pub(crate) enum Command {
    Add(add::CommandAdd),
    Log(log::CommandLog),
    Pull(pull::CommandPull),
    Push(push::CommandPush),
    Remove(remove::CommandRemove),
    Show(show::CommandShow),
}

impl Default for Command {
    fn default() -> Self {
        Self::Show(show::CommandShow::default())
    }
}

impl Command {
    pub(crate) fn execute<Repo: Repository, Out: Write, Err: Write>(
        self,
        repo: Repo,
        stdout: &mut Out,
        stderr: &mut Err,
    ) -> ExitCode {
        let result = match self {
            Self::Add(inner) => inner.execute(repo, stdout, stderr),
            Self::Log(inner) => inner.execute(repo, stdout, stderr),
            Self::Pull(inner) => inner.execute(repo, stdout, stderr),
            Self::Push(inner) => inner.execute(repo, stdout, stderr),
            Self::Remove(inner) => inner.execute(repo, stdout, stderr),
            Self::Show(inner) => inner.execute(repo, stdout, stderr),
        };

        if let Err(error) = result {
            writeln!(stderr, "{error}").expect("couldn't log error");
            ExitCode::Failure
        } else {
            ExitCode::Success
        }
    }
}

#[derive(Debug, Default, clap::Parser)]
pub(crate) struct GitCredentials {
    /// Username for git authentication
    #[clap(long, env = "GIT_USERNAME")]
    pub(crate) username: Option<String>,
    /// Password for git authentication
    #[clap(long, env = "GIT_PASSWORD")]
    pub(crate) password: Option<String>,
}
