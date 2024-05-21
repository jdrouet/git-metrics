use std::{borrow::Cow, collections::HashSet};

use super::{Error, Repository, NOTES_REF};
use crate::metric::Metric;

const NOTES_REF_OPTS: Option<&str> = Some(super::NOTES_REF);

struct Authenticator<'s> {
    url: &'s str,
    config: git2::Config,
    cred_helper: git2::CredentialHelper,
    github_token: Option<String>,
    cred_helper_failed: bool,
    default_failed: bool,
    github_plaintext_failed: bool,
    ssh_key_from_agent_failed: HashSet<Cow<'s, str>>,
    username_failed: HashSet<Cow<'s, str>>,
}

impl<'s> Authenticator<'s> {
    #[inline]
    fn new(config: git2::Config, url: &'s str) -> Self {
        let mut cred_helper = git2::CredentialHelper::new(url);
        cred_helper.config(&config);

        Self {
            url,
            config,
            cred_helper,
            github_token: std::env::var("GITHUB_TOKEN").ok(),
            cred_helper_failed: false,
            default_failed: false,
            github_plaintext_failed: false,
            ssh_key_from_agent_failed: HashSet::with_capacity(4),
            username_failed: HashSet::with_capacity(4),
        }
    }

    #[tracing::instrument(skip(self, allowed))]
    fn authenticate(
        &mut self,
        url: &str,
        username: Option<&str>,
        allowed: git2::CredentialType,
    ) -> Result<git2::Cred, git2::Error> {
        if url != self.url {
            tracing::warn!(
                "remote url different from given url, got {url:?}, expected {:?}",
                self.url
            );
        }

        if let Some(ref token) = self.github_token {
            if !self.github_plaintext_failed {
                tracing::trace!("found github token, authenticating with token");
                let res = git2::Cred::userpass_plaintext(token, "");
                if res.is_err() {
                    tracing::trace!("unable to authenticate with userpass_plaintext(github_token)");
                    self.github_plaintext_failed = true;
                }
                return res;
            }
        }

        if allowed.contains(git2::CredentialType::SSH_KEY) {
            if let Some(username) = username {
                if !self.ssh_key_from_agent_failed.contains(username) {
                    let res = git2::Cred::ssh_key_from_agent(username);
                    if res.is_err() {
                        tracing::trace!(
                            "unable to authenticate with ssh_key_from_agent({username:?})"
                        );
                        self.ssh_key_from_agent_failed
                            .insert(Cow::Owned(username.to_string()));
                    }
                    return res;
                }
            }
            if let Some(ref username) = self.cred_helper.username {
                if !self.ssh_key_from_agent_failed.contains(username.as_str()) {
                    let res = git2::Cred::ssh_key_from_agent(username);
                    if res.is_err() {
                        tracing::trace!(
                            "unable to authenticate with ssh_key_from_agent({username:?})"
                        );
                        self.ssh_key_from_agent_failed
                            .insert(Cow::Owned(username.to_string()));
                    }
                    return res;
                }
            }
            if !self.ssh_key_from_agent_failed.contains("git") {
                let res = git2::Cred::ssh_key_from_agent("git");
                if res.is_err() {
                    tracing::trace!("unable to authenticate with ssh_key_from_agent(git)");
                    self.ssh_key_from_agent_failed.insert(Cow::Borrowed("git"));
                }
                return res;
            }

            if let Some(ref username) = self.github_token {
                if !self.ssh_key_from_agent_failed.contains(username.as_str()) {
                    let res = git2::Cred::ssh_key_from_agent(username.as_str());
                    if res.is_err() {
                        tracing::trace!(
                            "unable to authenticate with ssh_key_from_agent(user_agent)"
                        );
                        self.ssh_key_from_agent_failed
                            .insert(Cow::Owned(username.to_string()));
                    }
                    return res;
                }
            }
        }

        if allowed.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
            if !self.cred_helper_failed {
                let res = git2::Cred::credential_helper(&self.config, url, username);
                if res.is_err() {
                    tracing::trace!(
                        "unable to authenticate with credential_helper(_, {url:?}, {username:?})"
                    );
                    self.cred_helper_failed = true;
                }
                return res;
            }
        }

        if allowed.contains(git2::CredentialType::USERNAME) {
            if let Some(username) = username {
                if !self.username_failed.contains(username) {
                    let res = git2::Cred::username(username);
                    if res.is_err() {
                        tracing::trace!("unable to authenticate with username({username:?})");
                        self.username_failed
                            .insert(Cow::Owned(username.to_string()));
                    }
                    return res;
                }
            }
            if let Some(ref username) = self.cred_helper.username {
                if !self.username_failed.contains(username.as_str()) {
                    let res = git2::Cred::username(username);
                    if res.is_err() {
                        tracing::trace!("unable to authenticate with username({username:?})");
                        self.username_failed
                            .insert(Cow::Owned(username.to_string()));
                    }
                    return res;
                }
            }

            if !self.username_failed.contains("git") {
                let res = git2::Cred::username("git");
                if res.is_err() {
                    tracing::trace!("unable to authenticate with username(git)");
                    self.username_failed.insert(Cow::Borrowed("git"));
                }
                return res;
            }
        }

        if allowed.contains(git2::CredentialType::DEFAULT) && !self.default_failed {
            let res = git2::Cred::default();
            if res.is_err() {
                tracing::trace!("unable to authenticate with default method");
                self.default_failed = true;
            }
            return res;
        }

        Err(git2::Error::from_str(
            "unable to find authentication method",
        ))
    }
}

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
        tracing::trace!("fetching revision id for target {target:?}");
        self.repo
            .revparse_single(target)
            .map(|rev| rev.id())
            .map_err(|err| {
                tracing::error!("unable to find revision id for target {target:?}: {err:?}");
                Error::new("target not found", err)
            })
    }

    fn signature(&self) -> Result<git2::Signature, Error> {
        tracing::trace!("fetching signature");
        self.repo.signature().map_err(|err| {
            tracing::error!("unable to get signature: {err:?}");
            Error::new("unable to get signature", err)
        })
    }
}

impl Repository for GitRepository {
    fn pull(&self, remote: &str) -> Result<(), Error> {
        let config = self.repo.config().map_err(|err| {
            tracing::error!("unable to read config: {err:?}");
            Error::new("unable to read config", err)
        })?;
        let remote = self.repo.find_remote(remote).map_err(|err| {
            tracing::error!("unable to find remote: {err:?}");
            Error::new("unable to find remote", err)
        })?;
        let url = remote.url().ok_or_else(|| {
            tracing::error!("unable to get url from remote");
            Error::new(
                "unable to get url from remote",
                git2::Error::from_str("invalid remote configuration"),
            )
        })?;
        let mut remote = remote.clone();
        let mut auth = Authenticator::new(config, url);
        let mut remote_cb = git2::RemoteCallbacks::new();
        remote_cb.credentials(|url, username, allowed| auth.authenticate(url, username, allowed));
        let mut fetch_opts = git2::FetchOptions::new();
        fetch_opts.remote_callbacks(remote_cb);
        remote
            .fetch(&[NOTES_REF], Some(&mut fetch_opts), None)
            .map_err(|err| {
                tracing::error!("unable to pull metrics: {err:?}");
                Error::new("unable to pull metrics", err)
            })
    }

    fn push(&self, remote: &str) -> Result<(), Error> {
        let config = self.repo.config().map_err(|err| {
            tracing::error!("unable to read config: {err:?}");
            Error::new("unable to read config", err)
        })?;
        let remote = self.repo.find_remote(remote).map_err(|err| {
            tracing::error!("unable to find remote {remote:?}: {err:?}");
            Error::new("unable to find remote", err)
        })?;
        let url = remote.url().ok_or_else(|| {
            tracing::error!("unable to get url from remote");
            Error::new(
                "unable to get url from remote",
                git2::Error::from_str("invalid remote configuration"),
            )
        })?;
        let mut remote = remote.clone();
        let mut auth = Authenticator::new(config, url);
        let mut remote_cb = git2::RemoteCallbacks::new();
        remote_cb.credentials(|url, username, allowed| auth.authenticate(url, username, allowed));

        let mut push_opts = git2::PushOptions::new();
        push_opts.remote_callbacks(remote_cb);
        remote
            .push(&[NOTES_REF], Some(&mut push_opts))
            .map_err(|err| {
                tracing::error!("unable to push metrics: {err:?}");
                Error::new("unable to push metrics", err)
            })
    }

    fn get_metrics(&self, target: &str) -> Result<Vec<Metric>, Error> {
        tracing::trace!("getting metrics for target {target:?}");
        let rev_id = self.revision_id(target)?;

        let Ok(note) = self.repo.find_note(NOTES_REF_OPTS, rev_id) else {
            tracing::debug!("no note found for revision");
            return Ok(Default::default());
        };

        note.message()
            .map(|msg| {
                tracing::trace!("deserializing note content");
                toml::from_str::<super::Note>(msg).map_err(|err| {
                    tracing::error!("unable to deserialize note: {err:?}");
                    Error::new("unable to deserialize note", err)
                })
            })
            .unwrap_or_else(|| {
                tracing::debug!("no message found for note {:?}", note.id());
                Ok(super::Note::default())
            })
            .map(|res| res.metrics)
    }

    fn set_metrics(&self, target: &str, metrics: Vec<Metric>) -> Result<(), Error> {
        tracing::trace!("settings {} metrics for target {target:?}", metrics.len());
        let head_id = self.revision_id(target)?;
        let sig = self.signature()?;

        tracing::trace!("serializing metrics");
        let note = toml::to_string_pretty(&super::Note { metrics }).map_err(|err| {
            tracing::error!("unable to serialize metrics: {err:?}");
            Error::new("unable to serialize metrics", err)
        })?;
        self.repo
            .note(&sig, &sig, NOTES_REF_OPTS, head_id, &note, true)
            .map_err(|err| {
                tracing::error!("unable to persist metrics: {err:?}");
                Error::new("unable to persist metrics", err)
            })?;

        Ok(())
    }
}
