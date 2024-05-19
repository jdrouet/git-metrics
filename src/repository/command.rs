use super::Error;

#[derive(Debug, Default)]
pub(crate) struct CommandRepository;

impl super::Repository for CommandRepository {
    fn pull(&self, remote: &str) -> Result<(), Error> {
        tracing::trace!("pulling metrics");
        let refs = format!("{}:{}", super::NOTES_REF, super::NOTES_REF);
        std::process::Command::new("git")
            .args(["fetch", remote, refs.as_str()])
            .spawn()
            .map_err(|err| {
                tracing::error!("unable to start pulling: {err:?}");
                Error::unable_to_pull(err)
            })
            .and_then(|mut cmd| {
                cmd.wait().map(|_| ()).map_err(|err| {
                    tracing::error!("pulling failed: {err:?}");
                    Error::unable_to_pull(err)
                })
            })
    }

    fn push(&self, remote: &str) -> Result<(), Error> {
        tracing::trace!("pushing metrics");
        std::process::Command::new("git")
            .args(["push", remote, super::NOTES_REF, "--force"])
            .spawn()
            .map_err(|err| {
                tracing::error!("unable to start pushing: {err:?}");
                Error::unable_to_push(err)
            })
            .and_then(|mut cmd| {
                cmd.wait().map(|_| ()).map_err(|err| {
                    tracing::error!("pushing failed: {err:?}");
                    Error::unable_to_push(err)
                })
            })
    }

    fn get_metrics(&self, _target: &str) -> Result<Vec<crate::metric::Metric>, Error> {
        unimplemented!()
    }

    fn set_metrics(
        &self,
        _target: &str,
        _metrics: Vec<crate::metric::Metric>,
    ) -> Result<(), Error> {
        unimplemented!()
    }
}
