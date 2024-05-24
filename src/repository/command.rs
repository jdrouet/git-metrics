use std::path::PathBuf;

use crate::repository::NOTES_REF_MAP;

use super::Error;

#[inline]
fn unable_execute_git_command(err: std::io::Error) -> Error {
    tracing::error!("unable to execute git command: {err:?}");
    Error::new("unable to execute git command", err)
}

#[derive(Debug)]
pub(crate) struct CommandRepository {
    path: Option<PathBuf>,
}

impl CommandRepository {
    pub fn new(path: Option<PathBuf>) -> Self {
        Self { path }
    }
}

impl CommandRepository {
    fn cmd(&self) -> std::process::Command {
        let mut cmd = std::process::Command::new("git");
        if let Some(ref path) = self.path {
            cmd.current_dir(path);
        }
        cmd
    }
}

impl super::Repository for CommandRepository {
    fn pull(&self, remote: &str) -> Result<(), Error> {
        tracing::trace!("pulling metrics");
        let output = self
            .cmd()
            .args(["fetch", remote, NOTES_REF_MAP])
            .output()
            .map_err(unable_execute_git_command)?;
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);

            if stderr.starts_with("fatal: couldn't find remote ref") {
                Ok(())
            } else {
                tracing::error!("something went wrong when fetching metrics");
                Err(Error::new(
                    "something went wrong when fetching metrics",
                    std::io::Error::new(std::io::ErrorKind::Other, stderr),
                ))
            }
        }
    }

    fn push(&self, remote: &str) -> Result<(), Error> {
        tracing::trace!("pushing metrics");
        let output = self
            .cmd()
            .args(["push", remote, NOTES_REF_MAP])
            .output()
            .map_err(unable_execute_git_command)?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!("something went wrong when pushing metrics");
            Err(Error::new(
                "something went wrong when pushing metrics",
                std::io::Error::new(std::io::ErrorKind::Other, stderr),
            ))
        } else {
            Ok(())
        }
    }

    fn get_metrics(&self, target: &str) -> Result<Vec<crate::metric::Metric>, Error> {
        tracing::trace!("getting metrics");
        let output = self
            .cmd()
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
        let output = self
            .cmd()
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
