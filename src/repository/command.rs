use super::Error;

#[inline]
fn unable_execute_git_command(err: std::io::Error) -> Error {
    tracing::error!("unable to execute git command: {err:?}");
    Error::new("unable to execute git command", err)
}

#[derive(Debug, Default)]
pub(crate) struct CommandRepository;

impl super::Repository for CommandRepository {
    fn pull(&self, remote: &str) -> Result<(), Error> {
        tracing::trace!("pulling metrics");
        let refs = format!("{}:{}", super::NOTES_REF, super::NOTES_REF);
        std::process::Command::new("git")
            .args(["fetch", remote, refs.as_str()])
            .spawn()
            .map_err(unable_execute_git_command)
            .and_then(|mut cmd| {
                cmd.wait().map(|_| ()).map_err(|err| {
                    tracing::error!("pulling failed: {err:?}");
                    Error::new("unable to pull metrics", err)
                })
            })
    }

    fn push(&self, remote: &str) -> Result<(), Error> {
        tracing::trace!("pushing metrics");
        std::process::Command::new("git")
            .args(["push", remote, super::NOTES_REF, "--force"])
            .spawn()
            .map_err(unable_execute_git_command)
            .and_then(|mut cmd| {
                cmd.wait().map(|_| ()).map_err(|err| {
                    tracing::error!("pushing failed: {err:?}");
                    Error::new("unable to push metrics", err)
                })
            })
    }

    fn get_metrics(&self, target: &str) -> Result<Vec<crate::metric::Metric>, Error> {
        tracing::trace!("getting metrics");
        let output = std::process::Command::new("git")
            .args(["notes", "--ref=metrics", "show", target])
            .output()
            .map_err(unable_execute_git_command)?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let note: super::Note = toml::from_str(&stdout).map_err(|err| {
                tracing::error!("unable to deserialize: {err:?}");
                Error::new("unable to deserialize note", err)
            })?;
            Ok(note.metrics)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if stderr.starts_with("error: no note found for object") {
                return Ok(Vec::new());
            }
            Err(Error::new(
                "git error",
                std::io::Error::new(std::io::ErrorKind::InvalidData, stderr),
            ))
        }
    }

    fn set_metrics(&self, target: &str, metrics: Vec<crate::metric::Metric>) -> Result<(), Error> {
        let note = super::Note { metrics };
        let message = toml::to_string(&note).map_err(|err| {
            tracing::error!("unable to serialize metrics: {err:?}");
            Error::new("unable to serialize metrics", err)
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
            .map_err(unable_execute_git_command)?;
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(Error::new(
                "git error",
                std::io::Error::new(std::io::ErrorKind::InvalidData, stderr),
            ))
        }
    }
}
