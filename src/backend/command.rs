use std::path::PathBuf;

use crate::backend::REMOTE_METRICS_REF;
use crate::entity::{Commit, Metric};

use super::Error;
use super::{HEAD, LOCAL_METRICS_REF, REMOTE_METRICS_MAP, REMOTE_METRICS_MAP_FORCE};

#[inline]
fn unable_execute_git_command(err: std::io::Error) -> Error {
    tracing::error!("unable to execute git command: {err:?}");
    Error::new("unable to execute git command", err)
}

#[derive(Debug)]
pub(crate) struct CommandBackend {
    path: Option<PathBuf>,
}

impl CommandBackend {
    pub fn new(path: Option<PathBuf>) -> Self {
        Self { path }
    }
}

impl CommandBackend {
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
}

impl super::Backend for CommandBackend {
    fn pull(&self, remote: &str) -> Result<(), Error> {
        tracing::trace!("pulling metrics");
        self.fetch_remote_metrics(remote)?;
        let remote_metrics = self.get_metrics_for_ref(HEAD, REMOTE_METRICS_REF)?;
        let local_metrics = self.get_metrics_for_ref(HEAD, LOCAL_METRICS_REF)?;
        let metrics = crate::entity::merge_metrics(remote_metrics, local_metrics);
        self.set_metrics_for_ref(HEAD, LOCAL_METRICS_REF, metrics)?;
        Ok(())
    }

    fn push(&self, remote: &str) -> Result<(), Error> {
        tracing::trace!("pushing metrics");
        let local_metrics = self.get_metrics_for_ref(HEAD, LOCAL_METRICS_REF)?;
        self.set_metrics_for_ref(HEAD, REMOTE_METRICS_REF, local_metrics)?;

        let output = self
            .cmd()
            .arg("push")
            .arg(remote)
            .arg(REMOTE_METRICS_MAP)
            .output()
            .map_err(unable_execute_git_command)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!("unable to push metrics");
            tracing::trace!("{stderr}");
            Err(Error::new(
                "unable to push metrics",
                std::io::Error::new(std::io::ErrorKind::Other, stderr),
            ))
        } else {
            Ok(())
        }
    }

    fn get_metrics_for_ref(&self, target: &str, ref_note: &str) -> Result<Vec<Metric>, Error> {
        tracing::trace!("getting metrics for target {target:?} and note {ref_note:?}");
        let output = self
            .cmd()
            .arg("notes")
            .arg("--ref")
            .arg(ref_note)
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

    fn set_metrics_for_ref(
        &self,
        target: &str,
        ref_note: &str,
        metrics: Vec<Metric>,
    ) -> Result<(), Error> {
        tracing::trace!(
            "settings {} metrics for target {target:?} and note {ref_note:?}",
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
            .arg(ref_note)
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

    fn get_commits(&self, range: &str) -> Result<Vec<Commit>, Error> {
        let output = self
            .cmd()
            .arg("log")
            .arg("--format=format:%H:%s")
            .arg(range)
            .output()
            .map_err(unable_execute_git_command)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!("something went wrong when getting commits");
            Err(Error::new(
                "something went wrong when getting commits",
                std::io::Error::new(std::io::ErrorKind::Other, stderr),
            ))
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(stdout
                .split('\n')
                .map(|item| item.trim())
                .filter(|item| !item.is_empty())
                .filter_map(|line| {
                    line.split_once(':').map(|(sha, summary)| Commit {
                        sha: sha.to_string(),
                        summary: summary.to_string(),
                    })
                })
                .collect())
        }
    }
}
