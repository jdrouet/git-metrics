pub(crate) mod add;
pub(crate) mod remove;
pub(crate) mod show;

pub(crate) trait Executor {
    fn execute<Repo: crate::repository::Repository, Out: std::io::Write, Err: std::io::Write>(
        self,
        repo: Repo,
        stdout: Out,
        stderr: Err,
    ) -> std::io::Result<()>;
}

#[derive(Debug, clap::Subcommand)]
pub(crate) enum Command {
    Add(add::CommandAdd),
    Remove(remove::CommandRemove),
    Show(show::CommandShow),
}

impl Default for Command {
    fn default() -> Self {
        Self::Show(show::CommandShow::default())
    }
}

impl Executor for Command {
    fn execute<Repo: crate::repository::Repository, Out: std::io::Write, Err: std::io::Write>(
        self,
        repo: Repo,
        stdout: Out,
        stderr: Err,
    ) -> std::io::Result<()> {
        match self {
            Self::Add(inner) => inner.execute(repo, stdout, stderr),
            Self::Remove(inner) => inner.execute(repo, stdout, stderr),
            Self::Show(inner) => inner.execute(repo, stdout, stderr),
        }
    }
}
