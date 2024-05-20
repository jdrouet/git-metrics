use super::{Error, Repository, NOTES_REF};
use crate::metric::Metric;

const NOTES_REF_OPTS: Option<&str> = Some(super::NOTES_REF);

// based on https://github.com/rust-lang/cargo/blob/bb6e446067d436acd14dda163f269cb295051e98/src/cargo/sources/git/utils.rs#L569
fn with_authentication<T, F>(url: &str, cfg: &git2::Config, mut f: F) -> Result<T, git2::Error>
where
    F: FnMut(&mut git2::Credentials<'_>) -> Result<T, git2::Error>,
{
    let mut cred_helper = git2::CredentialHelper::new(url);
    cred_helper.config(cfg);

    let mut ssh_username_requested = false;
    let mut cred_helper_bad = None;
    let mut github_bad = None;
    let mut ssh_agent_attempts = Vec::new();
    let mut any_attempts = false;
    let mut tried_sshkey = false;
    let mut url_attempt = None;

    let orig_url = url;
    let mut res = f(&mut |url, username, allowed| {
        any_attempts = true;
        if url != orig_url {
            url_attempt = Some(url.to_string());
        }
        // libgit2's "USERNAME" authentication actually means that it's just
        // asking us for a username to keep going. This is currently only really
        // used for SSH authentication and isn't really an authentication type.
        // The logic currently looks like:
        //
        //      let user = ...;
        //      if (user.is_null())
        //          user = callback(USERNAME, null, ...);
        //
        //      callback(SSH_KEY, user, ...)
        //
        // So if we're being called here then we know that (a) we're using ssh
        // authentication and (b) no username was specified in the URL that
        // we're trying to clone. We need to guess an appropriate username here,
        // but that may involve a few attempts. Unfortunately we can't switch
        // usernames during one authentication session with libgit2, so to
        // handle this we bail out of this authentication session after setting
        // the flag `ssh_username_requested`, and then we handle this below.
        if allowed.contains(git2::CredentialType::USERNAME) {
            debug_assert!(username.is_none());
            ssh_username_requested = true;
            return Err(git2::Error::from_str("gonna try usernames later"));
        }

        // An "SSH_KEY" authentication indicates that we need some sort of SSH
        // authentication. This can currently either come from the ssh-agent
        // process or from a raw in-memory SSH key. Cargo only supports using
        // ssh-agent currently.
        //
        // If we get called with this then the only way that should be possible
        // is if a username is specified in the URL itself (e.g., `username` is
        // Some), hence the unwrap() here. We try custom usernames down below.
        if allowed.contains(git2::CredentialType::SSH_KEY) && !tried_sshkey {
            // If ssh-agent authentication fails, libgit2 will keep
            // calling this callback asking for other authentication
            // methods to try. Make sure we only try ssh-agent once,
            // to avoid looping forever.
            tried_sshkey = true;
            let username = username.unwrap();
            debug_assert!(!ssh_username_requested);
            ssh_agent_attempts.push(username.to_string());
            return git2::Cred::ssh_key_from_agent(username);
        }

        if allowed.contains(git2::CredentialType::USER_PASS_PLAINTEXT) && github_bad.is_none() {
            if let Ok(github_token) = std::env::var("GITHUB_TOKEN") {
                let r = git2::Cred::userpass_plaintext(&github_token, "");
                github_bad = Some(r.is_err());
                return r;
            }
        }

        // Sometimes libgit2 will ask for a username/password in plaintext. This
        // is where Cargo would have an interactive prompt if we supported it,
        // but we currently don't! Right now the only way we support fetching a
        // plaintext password is through the `credential.helper` support, so
        // fetch that here.
        //
        // If ssh-agent authentication fails, libgit2 will keep calling this
        // callback asking for other authentication methods to try. Check
        // cred_helper_bad to make sure we only try the git credential helper
        // once, to avoid looping forever.
        if allowed.contains(git2::CredentialType::USER_PASS_PLAINTEXT) && cred_helper_bad.is_none()
        {
            let r = git2::Cred::credential_helper(cfg, url, username);
            cred_helper_bad = Some(r.is_err());
            return r;
        }

        // I'm... not sure what the DEFAULT kind of authentication is, but seems
        // easy to support?
        if allowed.contains(git2::CredentialType::DEFAULT) {
            return git2::Cred::default();
        }

        // Whelp, we tried our best
        Err(git2::Error::from_str("no authentication methods succeeded"))
    });

    // Ok, so if it looks like we're going to be doing ssh authentication, we
    // want to try a few different usernames as one wasn't specified in the URL
    // for us to use. In order, we'll try:
    //
    // * A credential helper's username for this URL, if available.
    // * This account's username.
    // * "git"
    //
    // We have to restart the authentication session each time (due to
    // constraints in libssh2 I guess? maybe this is inherent to ssh?), so we
    // call our callback, `f`, in a loop here.
    if ssh_username_requested {
        debug_assert!(res.is_err());
        let mut attempts = vec![String::from("git")];
        if let Ok(s) = std::env::var("GITHUB_TOKEN") {
            attempts.push(s);
        }
        if let Some(ref s) = cred_helper.username {
            attempts.push(s.clone());
        }

        while let Some(s) = attempts.pop() {
            // We should get `USERNAME` first, where we just return our attempt,
            // and then after that we should get `SSH_KEY`. If the first attempt
            // fails we'll get called again, but we don't have another option so
            // we bail out.
            let mut attempts = 0;
            res = f(&mut |_url, username, allowed| {
                if allowed.contains(git2::CredentialType::USERNAME) {
                    return git2::Cred::username(&s);
                }
                if allowed.contains(git2::CredentialType::SSH_KEY) {
                    debug_assert_eq!(Some(&s[..]), username);
                    attempts += 1;
                    if attempts == 1 {
                        ssh_agent_attempts.push(s.to_string());
                        return git2::Cred::ssh_key_from_agent(&s);
                    }
                }
                Err(git2::Error::from_str("no authentication methods succeeded"))
            });

            // If we made two attempts then that means:
            //
            // 1. A username was requested, we returned `s`.
            // 2. An ssh key was requested, we returned to look up `s` in the
            //    ssh agent.
            // 3. For whatever reason that lookup failed, so we were asked again
            //    for another mode of authentication.
            //
            // Essentially, if `attempts == 2` then in theory the only error was
            // that this username failed to authenticate (e.g., no other network
            // errors happened). Otherwise something else is funny so we bail
            // out.
            if attempts != 2 {
                break;
            }
        }
    }

    res
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
        let url = remote.url().unwrap();
        with_authentication(url, &config, |f| {
            let mut remote = remote.clone();
            let mut remote_cb = git2::RemoteCallbacks::new();
            remote_cb.credentials(f);
            let mut fetch_opts = git2::FetchOptions::new();
            fetch_opts.remote_callbacks(remote_cb);
            remote.fetch(&[NOTES_REF], Some(&mut fetch_opts), None)
        })
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
        let url = remote.url().unwrap();

        with_authentication(url, &config, |f| {
            let mut remote = remote.clone();
            let mut remote_cb = git2::RemoteCallbacks::new();
            remote_cb.credentials(f);
            let mut push_opts = git2::PushOptions::new();
            push_opts.remote_callbacks(remote_cb);
            remote.push(&[NOTES_REF], Some(&mut push_opts))
        })
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
