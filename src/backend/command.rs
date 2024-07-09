use std::path::PathBuf;

use super::NoteRef;
use crate::backend::REMOTE_METRICS_REF;
use crate::entity::git::Commit;

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("unable to execute")]
    UnableToExecute {
        #[from]
        #[source]
        source: std::io::Error,
    },
    #[error("execution failed")]
    Failed(String),
    #[error("invalid git range")]
    InvalidRange(String),
    #[error("unable to deserialize metrics")]
    Deserialize {
        #[from]
        #[source]
        source: toml::de::Error,
    },
    #[error("unable to serialize metrics")]
    Serialize {
        #[from]
        #[source]
        source: toml::ser::Error,
    },
    #[error("unable to push metrics")]
    UnableToPush(String),
}

impl crate::error::DetailedError for Error {
    fn details(&self) -> Option<String> {
        match self {
            Self::Deserialize { source } => Some(source.to_string()),
            Self::Failed(inner) | Self::InvalidRange(inner) | Self::UnableToPush(inner) => {
                Some(inner.clone())
            }
            Self::Serialize { source } => Some(source.to_string()),
            Self::UnableToExecute { source } => Some(source.to_string()),
        }
    }
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
    type Err = Error;

    fn rev_list(&self, range: &str) -> Result<Vec<String>, Self::Err> {
        tracing::trace!("listing revisions in range {range:?}");
        let output = self.cmd().arg("rev-list").arg(range).output()?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            tracing::trace!("stdout {stdout:?}");
            Ok(stdout
                .split('\n')
                .filter(|v| !v.is_empty())
                .map(String::from)
                .collect())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            tracing::trace!("stderr {stderr:?}");
            Err(Error::Failed(stderr))
        }
    }

    fn rev_parse(&self, range: &str) -> Result<super::RevParse, Self::Err> {
        tracing::trace!("parse revision range {range:?}");
        let output = self.cmd().arg("rev-parse").arg(range).output()?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            tracing::trace!("stdout {stdout:?}");
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
                Err(Error::InvalidRange(stdout.into()))
            }
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            tracing::trace!("stderr {stderr:?}");
            Err(Error::Failed(stderr))
        }
    }

    fn list_notes(&self, note_ref: &NoteRef) -> Result<Vec<super::Note>, Self::Err> {
        tracing::trace!("listing notes for ref {note_ref:?}");
        let output = self
            .cmd()
            .arg("notes")
            .arg("--ref")
            .arg(note_ref.to_string())
            .output()?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            tracing::trace!("stdout {stdout:?}");
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
            tracing::trace!("stderr {stderr:?}");
            Err(Error::Failed(stderr))
        }
    }

    fn remove_note(&self, target: &str, note_ref: &NoteRef) -> Result<(), Self::Err> {
        tracing::trace!("removing note for target {target:?} and {note_ref:?}");
        let output = self
            .cmd()
            .arg("notes")
            .arg("--ref")
            .arg(note_ref.to_string())
            .arg("remove")
            .arg(target)
            .output()?;
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            tracing::trace!("stderr {stderr:?}");
            Err(Error::Failed(stderr))
        }
    }

    fn read_note<T: serde::de::DeserializeOwned>(
        &self,
        target: &str,
        note_ref: &NoteRef,
    ) -> Result<Option<T>, Self::Err> {
        tracing::trace!("getting note for target {target:?} and note {note_ref:?}");
        let output = self
            .cmd()
            .arg("notes")
            .arg("--ref")
            .arg(note_ref.to_string())
            .arg("show")
            .arg(target)
            .output()?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            tracing::trace!("stdout {stdout:?}");
            let note: T = toml::from_str(&stdout)?;
            Ok(Some(note))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            tracing::trace!("stderr {stderr:?}");
            if stderr.starts_with("error: no note found for object") {
                return Ok(None);
            }
            Err(Error::Failed(stderr))
        }
    }

    fn write_note<T: serde::Serialize>(
        &self,
        target: &str,
        note_ref: &NoteRef,
        value: &T,
    ) -> Result<(), Self::Err> {
        tracing::trace!("setting note for target {target:?} and note {note_ref:?}",);
        let message = toml::to_string(value)?;
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
            .output()?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            tracing::trace!("stderr {stderr:?}");
            Err(Error::Failed(stderr))
        }
    }

    fn pull(&self, remote: &str, local_ref: &NoteRef) -> Result<(), Self::Err> {
        tracing::trace!("pulling metrics");
        let output = self
            .cmd()
            .arg("fetch")
            .arg(remote)
            .arg(format!("+{REMOTE_METRICS_REF}:{local_ref}",))
            .output()?;
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::trace!("stderr {stderr:?}");

            if stderr.starts_with("fatal: couldn't find remote ref") {
                Ok(())
            } else {
                tracing::error!("something went wrong when fetching metrics");
                tracing::trace!("{stderr}");
                Err(Error::Failed(stderr.into()))
            }
        }
    }

    fn push(&self, remote: &str, local_ref: &NoteRef) -> Result<(), Self::Err> {
        tracing::trace!("pushing metrics");

        let output = self
            .cmd()
            .arg("push")
            .arg(remote)
            .arg(format!("{local_ref}:{REMOTE_METRICS_REF}",))
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!("unable to push metrics");
            tracing::trace!("stderr {stderr:?}");
            Err(Error::UnableToPush(stderr.into()))
        } else {
            Ok(())
        }
    }

    fn get_commits(&self, range: &str) -> Result<Vec<Commit>, Self::Err> {
        let output = self
            .cmd()
            .arg("log")
            .arg("--format=format:%H:%s")
            .arg(range)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!("something went wrong when getting commits");
            tracing::trace!("stderr {stderr:?}");
            Err(Error::Failed(stderr.into()))
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout);
            tracing::trace!("stdout {stdout:?}");
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

    fn root_path(&self) -> Result<PathBuf, Self::Err> {
        let output = self
            .cmd()
            .arg("rev-parse")
            .arg("--show-toplevel")
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!("something went wrong when getting commits");
            tracing::trace!("stderr {stderr:?}");
            Err(Error::Failed(stderr.into()))
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout);
            tracing::trace!("stdout {stdout:?}");
            Ok(PathBuf::from(stdout.trim()))
        }
    }
}
