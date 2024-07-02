use std::io::Write;

use crate::backend::Backend;
use crate::error::DetailedError;
use crate::ExitCode;

pub(crate) mod add;
pub(crate) mod check;
pub(crate) mod diff;
pub(crate) mod init;
pub(crate) mod log;
pub(crate) mod pull;
pub(crate) mod push;
pub(crate) mod remove;
pub(crate) mod show;

mod prelude;

pub(crate) trait Executor {
    fn execute<B: Backend, Out: Write>(
        self,
        backend: B,
        stdout: &mut Out,
    ) -> Result<(), crate::service::Error>;
}

#[derive(Debug, clap::Subcommand)]
pub(crate) enum Command {
    Add(add::CommandAdd),
    Check(check::CommandCheck),
    Diff(diff::CommandDiff),
    Init(init::CommandInit),
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
    pub(crate) fn execute<Repo: Backend, Out: Write, Err: Write>(
        self,
        repo: Repo,
        stdout: &mut Out,
        stderr: &mut Err,
    ) -> ExitCode {
        let result = match self {
            Self::Add(inner) => inner.execute(repo, stdout),
            Self::Check(inner) => inner.execute(repo, stdout),
            Self::Diff(inner) => inner.execute(repo, stdout),
            Self::Init(inner) => inner.execute(repo, stdout),
            Self::Log(inner) => inner.execute(repo, stdout),
            Self::Pull(inner) => inner.execute(repo, stdout),
            Self::Push(inner) => inner.execute(repo, stdout),
            Self::Remove(inner) => inner.execute(repo, stdout),
            Self::Show(inner) => inner.execute(repo, stdout),
        };

        if let Err(error) = result {
            error.write(stderr).expect("couldn't log error");
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
