use std::path::Path;

pub(crate) fn to_file(path: &Path, payload: &super::Payload) -> Result<(), super::Error> {
    let mut file = super::with_file(path)?;
    to_writer(&mut file, payload)?;
    Ok(())
}

pub(crate) fn to_writer<W: std::io::Write>(
    output: W,
    payload: &super::Payload,
) -> Result<(), super::Error> {
    serde_json::to_writer(output, payload)?;
    Ok(())
}
