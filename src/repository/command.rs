use std::path::PathBuf;

use super::{HEAD, REMOTE_METRICS_MAP, REMOTE_METRICS_MAP_FORCE};
use crate::metric::Metric;

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

    fn fetch_remote_metrics(&self, remote: &str) -> Result<(), Error> {
        tracing::trace!("fetch remote metrics from {remote:?}");
        let output = self
            .cmd()
            .args(["fetch", remote, REMOTE_METRICS_MAP_FORCE])
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

    fn get_metrics_for_note(&self, target: &str, note_name: &str) -> Result<Vec<Metric>, Error> {
        tracing::trace!("getting metrics for target {target:?} and note {note_name:?}");
        let output = self
            .cmd()
            .arg("notes")
            .arg("--ref")
            .arg(note_name)
            .arg("show")
            .arg(target)
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

    fn set_metrics_for_note(
        &self,
        target: &str,
        note_name: &str,
        metrics: Vec<Metric>,
    ) -> Result<(), Error> {
        tracing::trace!(
            "settings {} metrics for target {target:?} and note {note_name:?}",
            metrics.len()
        );
        let note = super::Note { metrics };
        let message = toml::to_string(&note).map_err(|err| {
            tracing::error!("unable to serialize metrics: {err:?}");
            Error::new("unable to serialize metrics", err)
        })?;
        let output = self
            .cmd()
            .arg("notes")
            .arg("--ref")
            .arg(note_name)
            .arg("add")
            .arg("-f")
            .arg("-m")
            .arg(message.as_str())
            .arg(target)
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

impl super::Repository for CommandRepository {
    fn pull(&self, remote: &str) -> Result<(), Error> {
        tracing::trace!("pulling metrics");
        self.fetch_remote_metrics(remote)?;
        let remote_metrics = self.get_metrics_for_note(HEAD, "metrics")?;
        let local_metrics = self.get_metrics_for_note(HEAD, "local-metrics")?;
        let metrics = crate::metric::merge(remote_metrics, local_metrics);
        self.set_metrics_for_note(HEAD, "local-metrics", metrics)?;
        Ok(())
    }

    fn push(&self, remote: &str) -> Result<(), Error> {
        tracing::trace!("pushing metrics");
        let local_metrics = self.get_metrics_for_note(HEAD, "local-metrics")?;
        self.set_metrics_for_note(HEAD, "metrics", local_metrics)?;

        let output = self
            .cmd()
            .arg("push")
            .arg(remote)
            .arg(REMOTE_METRICS_MAP)
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
        self.get_metrics_for_note(target, "local-metrics")
    }

    fn set_metrics(&self, target: &str, metrics: Vec<crate::metric::Metric>) -> Result<(), Error> {
        self.set_metrics_for_note(target, "local-metrics", metrics)
    }
}
