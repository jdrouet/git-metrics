pub(crate) trait DetailedError: std::error::Error {
    fn details(&self) -> Option<String>;

    fn write<W: std::io::Write>(&self, mut w: W) -> std::io::Result<()> {
        writeln!(w, "{self}")?;
        if let Some(details) = self.details() {
            write!(w, "\n\n")?;
            for line in details.split('\n').filter(|line| !line.is_empty()) {
                writeln!(w, "\t{line}")?;
            }
        }
        Ok(())
    }
}
