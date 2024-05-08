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
    if let Err(err) = args
        .command
        .execute(repo, std::io::stdout(), std::io::stderr())
    {
        eprintln!("io error: {err:?}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::Args;

    #[test]
    fn args_should_parse_show() {
        let args = Args::parse_from(&["this", "show"]);
        assert!(matches!(args.command, crate::cmd::Command::Show(_)));
    }
}
