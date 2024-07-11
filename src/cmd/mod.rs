use std::io::Write;

use prelude::{ColoredWriter, PrettyWriter};

use crate::backend::Backend;
use crate::error::DetailedError;
use crate::ExitCode;

mod add;
mod check;
mod diff;
mod init;
mod log;
mod pull;
mod push;
mod remove;
mod show;

mod format;
mod prelude;

trait Executor {
    fn execute<B: Backend, Out: PrettyWriter>(
        self,
        backend: B,
        stdout: &mut Out,
    ) -> Result<ExitCode, crate::service::Error>;
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
        let mut stdout = ColoredWriter::from(stdout);
        let result = match self {
            Self::Add(inner) => inner.execute(repo, &mut stdout),
            Self::Check(inner) => inner.execute(repo, &mut stdout),
            Self::Diff(inner) => inner.execute(repo, &mut stdout),
            Self::Init(inner) => inner.execute(repo, &mut stdout),
            Self::Log(inner) => inner.execute(repo, &mut stdout),
            Self::Pull(inner) => inner.execute(repo, &mut stdout),
            Self::Push(inner) => inner.execute(repo, &mut stdout),
            Self::Remove(inner) => inner.execute(repo, &mut stdout),
            Self::Show(inner) => inner.execute(repo, &mut stdout),
        };

        match result {
            Ok(res) => res,
            Err(error) => {
                error.write(stderr).expect("couldn't log error");
                ExitCode::Failure
            }
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
