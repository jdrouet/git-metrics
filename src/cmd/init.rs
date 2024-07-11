use crate::ExitCode;

/// Initialize the git-metrics configuration
#[derive(clap::Parser, Debug, Default)]
pub struct CommandInit;

impl crate::cmd::Executor for CommandInit {
    fn execute<B: crate::backend::Backend, Out: std::io::Write>(
        self,
        _backend: B,
        _stdout: &mut Out,
    ) -> Result<ExitCode, crate::service::Error> {
        Ok(ExitCode::Success)
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::CommandInit;
    use crate::cmd::Executor;

    #[test]
    fn should_do_nothing_for_now() {
        let backend = crate::backend::mock::MockBackend::default();
        let mut stdout: Vec<u8> = Vec::new();
        let cmd = CommandInit::parse_from(["_"]).execute(backend, &mut stdout);
        assert!(cmd.is_ok());
    }
}
