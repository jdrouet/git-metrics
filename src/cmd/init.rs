/// Initialize the git-metrics configuration
#[derive(clap::Parser, Debug, Default)]
pub(crate) struct CommandInit;

impl crate::cmd::Executor for CommandInit {
    fn execute<B: crate::backend::Backend, Out: std::io::Write>(
        self,
        _backend: B,
        _stdout: &mut Out,
    ) -> Result<(), crate::service::Error> {
        Ok(())
    }
}
