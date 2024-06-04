use std::path::PathBuf;

use super::{Error, NoteRef};
use crate::backend::REMOTE_METRICS_REF;
use crate::entity::Commit;

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
}

impl super::Backend for CommandBackend {
    fn rev_list(&self, range: &str) -> Result<Vec<String>, Error> {
        tracing::trace!("listing revisions in range {range:?}");
        let output = self
            .cmd()
            .arg("rev-list")
            .arg(range)
            .output()
            .map_err(unable_execute_git_command)?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(stdout
                .split('\n')
                .filter(|v| !v.is_empty())
                .map(String::from)
                .collect())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(Error::new(
                "git error",
                std::io::Error::new(std::io::ErrorKind::InvalidData, stderr),
            ))
        }
    }

    fn rev_parse(&self, range: &str) -> Result<super::RevParse, Error> {
        tracing::trace!("parse revision range {range:?}");
        let output = self
            .cmd()
            .arg("rev-parse")
            .arg(range)
            .output()
            .map_err(unable_execute_git_command)?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut iter = stdout.split('\n').filter(|v| !v.is_empty());
            if let Some(first) = iter.next() {
                if let Some(second) = iter.next().and_then(|v| v.strip_prefix('^')) {
                    Ok(super::RevParse::Range(
                        second.to_string(),
                        first.to_string(),
                    ))
                } else {
                    Ok(super::RevParse::Single(first.to_string()))
                }
            } else {
                Err(Error::new(
                    "invalid range",
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, stdout),
                ))
            }
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(Error::new(
                "git error",
                std::io::Error::new(std::io::ErrorKind::InvalidData, stderr),
            ))
        }
    }

    fn list_notes(&self, note_ref: &NoteRef) -> Result<Vec<super::Note>, Error> {
        tracing::trace!("listing notes for ref {note_ref:?}");
        let output = self
            .cmd()
            .arg("notes")
            .arg("--ref")
            .arg(note_ref.to_string())
            .output()
            .map_err(unable_execute_git_command)?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(stdout
                .split('\n')
                .filter_map(|line| {
                    line.split_once(' ')
                        .map(|(note_id, commit_id)| super::Note {
                            note_id: note_id.to_string(),
                            commit_id: commit_id.to_string(),
                        })
                })
                .collect())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(Error::new(
                "git error",
                std::io::Error::new(std::io::ErrorKind::InvalidData, stderr),
            ))
        }
    }

    fn remove_note(&self, target: &str, note_ref: &NoteRef) -> Result<(), Error> {
        tracing::trace!("removing note for target {target:?} and {note_ref:?}");
        let output = self
            .cmd()
            .arg("notes")
            .arg("--ref")
            .arg(note_ref.to_string())
            .arg("remove")
            .arg(target)
            .output()
            .map_err(unable_execute_git_command)?;
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(Error::new(
                "git error",
                std::io::Error::new(std::io::ErrorKind::Other, stderr),
            ))
        }
    }

    fn read_note<T: serde::de::DeserializeOwned>(
        &self,
        target: &str,
        note_ref: &NoteRef,
    ) -> Result<Option<T>, Error> {
        tracing::trace!("getting note for target {target:?} and note {note_ref:?}");
        let output = self
            .cmd()
            .arg("notes")
            .arg("--ref")
            .arg(note_ref.to_string())
            .arg("show")
            .arg(target)
            .output()
            .map_err(unable_execute_git_command)?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let note: T = toml::from_str(&stdout).map_err(|err| {
                tracing::error!("unable to deserialize: {err:?}");
                Error::new("unable to deserialize note", err)
            })?;
            Ok(Some(note))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if stderr.starts_with("error: no note found for object") {
                return Ok(None);
            }
            Err(Error::new(
                "git error",
                std::io::Error::new(std::io::ErrorKind::InvalidData, stderr),
            ))
        }
    }

    fn write_note<T: serde::Serialize>(
        &self,
        target: &str,
        note_ref: &NoteRef,
        value: &T,
    ) -> Result<(), Error> {
        tracing::trace!("setting note for target {target:?} and note {note_ref:?}",);
        let message = toml::to_string(value).map_err(|err| {
            tracing::error!("unable to serialize metrics: {err:?}");
            Error::new("unable to serialize metrics", err)
        })?;
        let output = self
            .cmd()
            .arg("notes")
            .arg("--ref")
            .arg(note_ref.to_string())
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

    fn pull(&self, remote: &str, local_ref: &NoteRef) -> Result<(), Error> {
        tracing::trace!("pulling metrics");
        let output = self
            .cmd()
            .arg("fetch")
            .arg(remote)
            .arg(format!("+{REMOTE_METRICS_REF}:{local_ref}",))
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
                tracing::trace!("{stderr}");
                Err(Error::new(
                    "something went wrong when fetching metrics",
                    std::io::Error::new(std::io::ErrorKind::Other, stderr),
                ))
            }
        }
    }

    fn push(&self, remote: &str, local_ref: &NoteRef) -> Result<(), Error> {
        tracing::trace!("pushing metrics");

        let output = self
            .cmd()
            .arg("push")
            .arg(remote)
            .arg(format!("{local_ref}:{REMOTE_METRICS_REF}",))
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
