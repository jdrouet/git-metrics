use std::io::Write;

use prelude::{BasicWriter, ColoredWriter, PrettyWriter};

use crate::backend::Backend;
use crate::error::DetailedError;
use crate::ExitCode;

mod add;
mod check;
mod diff;
#[cfg(feature = "exporter")]
mod export;
#[cfg(feature = "importer")]
mod import;
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
        stdout: Out,
    ) -> Result<ExitCode, crate::service::Error>;
}

#[derive(Debug, clap::Subcommand)]
pub(crate) enum Command {
    Add(add::CommandAdd),
    Check(check::CommandCheck),
    Diff(diff::CommandDiff),
    Export(export::CommandExport),
    Init(init::CommandInit),
    #[cfg(feature = "importer")]
    Import(import::CommandImport),
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
    fn execute_with<Repo: Backend, Out: PrettyWriter>(
        self,
        repo: Repo,
        stdout: Out,
    ) -> Result<ExitCode, crate::service::Error> {
        match self {
            Self::Add(inner) => inner.execute(repo, stdout),
            Self::Check(inner) => inner.execute(repo, stdout),
            Self::Diff(inner) => inner.execute(repo, stdout),
            Self::Export(inner) => inner.execute(repo, stdout),
            Self::Init(inner) => inner.execute(repo, stdout),
            #[cfg(feature = "importer")]
            Self::Import(inner) => inner.execute(repo, stdout),
            Self::Log(inner) => inner.execute(repo, stdout),
            Self::Pull(inner) => inner.execute(repo, stdout),
            Self::Push(inner) => inner.execute(repo, stdout),
            Self::Remove(inner) => inner.execute(repo, stdout),
            Self::Show(inner) => inner.execute(repo, stdout),
        }
    }

    pub(crate) fn execute<Repo: Backend, Out: Write, Err: Write>(
        self,
        repo: Repo,
        color_enabled: bool,
        stdout: Out,
        stderr: Err,
    ) -> ExitCode {
        let result = if color_enabled {
            self.execute_with(repo, ColoredWriter::from(stdout))
        } else {
            self.execute_with(repo, BasicWriter::from(stdout))
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
