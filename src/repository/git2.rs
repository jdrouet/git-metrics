use super::{Error, Repository, NOTES_REF};
use crate::metric::Metric;

// see https://github.com/rust-lang/cargo/blob/bb28e71202260180ecff658cd0fa0c7ba86d0296/src/cargo/sources/git/utils.rs#L344-L391
fn with_credentials(
    config: &git2::Config,
    url: &str,
    username: Option<&str>,
    allowed_types: git2::CredentialType,
) -> Result<git2::Cred, git2::Error> {
    let mut cred_helper = git2::CredentialHelper::new(url);
    cred_helper.config(config);

    let mut res = Err(git2::Error::from_str("no authentication available"));

    if allowed_types.contains(git2::CredentialType::SSH_KEY) {
        res = res.or_else(|_| {
            let user = username
                .or(cred_helper.username.as_deref())
                .unwrap_or("git");
            git2::Cred::ssh_key_from_agent(user)
        });
    }

    if allowed_types.contains(git2::CredentialType::USERNAME) {
        if let Some(username) = username {
            res = res.or_else(|_| git2::Cred::username(username));
        }
        if let Some(ref username) = cred_helper.username {
            res = res.or_else(|_| git2::Cred::username(username));
        }
    }

    if allowed_types.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
        if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            res = res.or_else(|_| git2::Cred::userpass_plaintext(&token, ""));
        }
        res = res.or_else(|_| git2::Cred::credential_helper(config, url, username));
    }

    if allowed_types.contains(git2::CredentialType::DEFAULT) {
        res = res.or_else(|_| git2::Cred::default());
    }

    res
}

pub(crate) struct GitRepository {
    repo: git2::Repository,
    fallback_command: bool,
}

impl GitRepository {
    pub(crate) fn from_env() -> Result<Self, String> {
        let repo = git2::Repository::open_from_env()
            .map_err(|err| format!("unable to open repository: {err:?}"))?;
        Ok(GitRepository {
            repo,
            fallback_command: true,
        })
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

    fn command_push(&self, remote: &str) -> Result<(), Error> {
        std::process::Command::new("git")
            .args(["push", remote, super::NOTES_REF])
            .spawn()
            .map_err(Error::unable_to_push)
            .and_then(|mut cmd| cmd.wait().map(|_| ()).map_err(Error::unable_to_push))
    }

    fn manual_push(&self, remote: &str) -> Result<(), Error> {
        let config = self
            .repo
            .config()
            .map_err(|err| Error::new(super::ErrorKind::UnableToReadConfig, err))?;
        let mut remote = self
            .repo
            .find_remote(remote)
            .map_err(Error::remote_not_found)?;
        let mut remote_cb = git2::RemoteCallbacks::new();
        remote_cb.credentials(|url, username, allowed_types| {
            with_credentials(&config, url, username, allowed_types)
        });
        let mut push_opts = git2::PushOptions::new();
        push_opts.remote_callbacks(remote_cb);
        remote
            .push(&[NOTES_REF], Some(&mut push_opts))
            .map_err(Error::unable_to_push)
    }
}

impl Repository for GitRepository {
    fn push(&self, remote: &str) -> Result<(), Error> {
        let res = self.manual_push(remote);
        if self.fallback_command {
            res.or_else(|_| self.command_push(remote))
        } else {
            res
        }
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
