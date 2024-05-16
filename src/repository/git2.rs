use std::process::Command;

use super::{Error, Repository, NOTES_REF};
use crate::metric::Metric;

pub(crate) struct GitRepository {
    repo: git2::Repository,
}

impl GitRepository {
    pub(crate) fn from_env() -> Result<Self, String> {
        let repo = git2::Repository::open_from_env()
            .map_err(|err| format!("unable to open repository: {err:?}"))?;
        Ok(GitRepository { repo })
    }

    fn revision_id(&self, target: &str) -> Result<git2::Oid, Error> {
        self.repo
            .revparse_single(target)
            .map(|rev| rev.id())
            .map_err(Error::target_not_found)
    }

    fn signature(&self) -> Result<git2::Signature, Error> {
        self.repo.signature().map_err(Error::signature_not_found)
    }
}

fn git_credentials(
    _url: &str,
    username_from_url: Option<&str>,
    _allowed_types: git2::CredentialType,
) -> Result<git2::Cred, git2::Error> {
    let res = git2::Cred::default();
    if let Some(username) = username_from_url {
        let res = if let Some(id_rsa) = std::env::var("HOME")
            .ok()
            .map(|value| std::path::PathBuf::from(value).join(".ssh").join("id_rsa"))
            .filter(|value| value.exists())
        {
            git2::Cred::ssh_key(username, None, &id_rsa, None).or(res)
        } else {
            res
        };
        git2::Cred::ssh_key_from_agent(username).or(res)
    } else {
        res
    }
}

impl Repository for GitRepository {
    fn push(&self, remote: &str) -> Result<(), Error> {
        Command::new("git")
            .args(["push", remote, NOTES_REF])
            .spawn()
            .and_then(|mut res| res.wait())
            .map(|_| ())
            .map_err(Error::unable_to_push)
    }

    fn get_metrics(&self, target: &str) -> Result<Vec<Metric>, Error> {
        let rev_id = self.revision_id(target)?;

        let Ok(note) = self.repo.find_note(super::NOTES_REF_OPTS, rev_id) else {
            return Ok(Default::default());
        };

        note.message()
            .map(|msg| toml::from_str::<super::Note>(msg).map_err(Error::unable_to_decode))
            .unwrap_or(Ok(super::Note::default()))
            .map(|res| res.metrics)
    }

    fn set_metrics(&self, target: &str, metrics: Vec<Metric>) -> Result<(), Error> {
        let head_id = self.revision_id(target)?;
        let sig = self.signature()?;

        let note =
            toml::to_string_pretty(&super::Note { metrics }).map_err(Error::unable_to_encode)?;
        self.repo
            .note(&sig, &sig, super::NOTES_REF_OPTS, head_id, &note, true)
            .map_err(Error::unable_to_persist)?;

        Ok(())
    }
}
