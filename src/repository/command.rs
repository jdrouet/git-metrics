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

    fn get_metrics(&self, target: &str) -> Result<Vec<crate::metric::Metric>, Error> {
        tracing::trace!("getting metrics");
        let output = std::process::Command::new("git")
            .args(["notes", "--ref=metrics", "show", target])
            .output()
            .map_err(|err| {
                tracing::error!("unable to run git: {err:?}");
                Error::target_not_found(err)
            })?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let note: super::Note = toml::from_str(&stdout).map_err(|err| {
                tracing::error!("unable to deserialize: {err:?}");
                Error::unable_to_decode(err)
            })?;
            Ok(note.metrics)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if stderr.starts_with("error: no note found for object") {
                return Ok(Vec::new());
            }
            Err(Error::target_not_found(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                stderr,
            )))
        }
    }

    fn set_metrics(&self, target: &str, metrics: Vec<crate::metric::Metric>) -> Result<(), Error> {
        let note = super::Note { metrics };
        let message = toml::to_string(&note).map_err(|err| {
            tracing::error!("unable to serialize metrics: {err:?}");
            Error::unable_to_encode(err)
        })?;
        let output = std::process::Command::new("git")
            .args([
                "notes",
                "--ref=metrics",
                "add",
                "-f",
                "-m",
                message.as_str(),
                target,
            ])
            .output()
            .map_err(|err| {
                tracing::error!("unable to run git: {err:?}");
                Error::target_not_found(err)
            })?;
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(Error::target_not_found(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                stderr,
            )))
        }
    }
}
