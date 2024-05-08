mod cmd;
mod metric;
mod repository;

use clap::Parser;
use cmd::Executor;

/// Git extension in order to attach metrics to commits
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: cmd::Command,
}

fn main() {
    let args = Args::parse();
    let repo = crate::repository::GitRepository::from_env().unwrap();

    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();

    if let Err(err) = args.command.execute(repo, &mut stdout, &mut stderr) {
        eprintln!("{err:?}");
        std::process::exit(1);
    }
}
