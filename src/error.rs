pub(crate) trait DetailedError: std::error::Error {
    fn details(&self) -> Option<String>;
}

pub(crate) trait ErrorWriter {
    fn write<W: std::io::Write, E: DetailedError>(
        &self,
        writer: W,
        element: E,
    ) -> std::io::Result<()>;
}

#[derive(Default)]
pub(crate) struct SimpleErrorWriter;

impl ErrorWriter for SimpleErrorWriter {
    fn write<W: std::io::Write, E: DetailedError>(
        &self,
        mut writer: W,
        element: E,
    ) -> std::io::Result<()> {
        writeln!(writer, "{element}")?;
        if let Some(details) = element.details() {
            write!(writer, "\n\n")?;
            for line in details.split('\n').filter(|line| !line.is_empty()) {
                writeln!(writer, "\t{line}")?;
            }
        }
        Ok(())
    }
}
