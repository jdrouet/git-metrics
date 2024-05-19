use crate::repository::Repository;
use std::io::Write;

pub(crate) mod add;
pub(crate) mod push;
pub(crate) mod remove;
pub(crate) mod show;

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
    Push(push::CommandPush),
    Remove(remove::CommandRemove),
    Show(show::CommandShow),
}

impl Default for Command {
    fn default() -> Self {
        Self::Show(show::CommandShow::default())
    }
}

impl Executor for Command {
    fn execute<Repo: Repository, Out: Write, Err: Write>(
        self,
        repo: Repo,
        stdout: &mut Out,
        stderr: &mut Err,
    ) -> Result<(), Error> {
        match self {
            Self::Add(inner) => inner.execute(repo, stdout, stderr),
            Self::Push(inner) => inner.execute(repo, stdout, stderr),
            Self::Remove(inner) => inner.execute(repo, stdout, stderr),
            Self::Show(inner) => inner.execute(repo, stdout, stderr),
        }
    }
}
