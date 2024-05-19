use super::Error;

pub(crate) fn pull(remote: &str) -> Result<(), super::Error> {
    tracing::trace!("pulling metrics");
    let refs = format!("{}:{}", super::NOTES_REF, super::NOTES_REF);
    std::process::Command::new("git")
        .args(["fetch", remote, refs.as_str()])
        .spawn()
        .map_err(|err| {
            tracing::error!("unable to start pulling: {err:?}");
            Error::unable_to_pull(err)
        })
        .and_then(|mut cmd| {
            cmd.wait().map(|_| ()).map_err(|err| {
                tracing::error!("pulling failed: {err:?}");
                Error::unable_to_pull(err)
            })
        })
}

pub(crate) fn push(remote: &str) -> Result<(), Error> {
    tracing::trace!("pushing metrics");
    std::process::Command::new("git")
        .args(["push", remote, super::NOTES_REF, "--force"])
        .spawn()
        .map_err(|err| {
            tracing::error!("unable to start pushing: {err:?}");
            Error::unable_to_push(err)
        })
        .and_then(|mut cmd| {
            cmd.wait().map(|_| ()).map_err(|err| {
                tracing::error!("pushing failed: {err:?}");
                Error::unable_to_push(err)
            })
        })
}
