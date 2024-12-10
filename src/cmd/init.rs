use super::prelude::PrettyWriter;
use crate::entity::config::Config;
use crate::ExitCode;

/// Initialize the git-metrics configuration
#[derive(clap::Parser, Debug, Default)]
pub struct CommandInit;

impl crate::cmd::Executor for CommandInit {
    fn execute<B: crate::backend::Backend, Out: PrettyWriter>(
        self,
        backend: B,
        _stdout: Out,
    ) -> Result<ExitCode, crate::service::Error> {
        let root = backend.root_path()?;
        Config::write_sample(&root)?;
        Ok(ExitCode::Success)
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::CommandInit;
    use crate::cmd::prelude::BasicWriter;
    use crate::cmd::Executor;

    #[test]
    fn should_do_nothing_for_now() {
        let backend = crate::backend::mock::MockBackend::default();
        let stdout = BasicWriter::from(Vec::<u8>::new());
        let cmd = CommandInit::parse_from(["_"]).execute(backend, stdout);
        assert!(cmd.is_ok());
    }
}
