use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct Tag {
    pub name: String,
    pub value: String,
}

impl FromStr for Tag {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split_once(':')
            .map(|(name, value)| Tag {
                name: name.trim().to_string(),
                value: value.trim().to_string(),
            })
            .ok_or("unable to decode tag name and value")
    }
}

pub trait PrettyWriter: std::io::Write + Sized {
    fn set_style<S: std::fmt::Display>(&mut self, style: S) -> std::io::Result<()>;

    #[inline]
    fn write_str(&mut self, value: &str) -> std::io::Result<usize> {
        self.write(value.as_bytes())
    }

    #[inline]
    fn write_element<E: PrettyDisplay>(&mut self, element: E) -> std::io::Result<()> {
        element.print(self)
    }
}

pub struct ColoredWriter<W>(W);

impl<W: std::io::Write> From<W> for ColoredWriter<W> {
    #[inline]
    fn from(value: W) -> Self {
        Self(value)
    }
}

impl<W: std::io::Write> std::io::Write for ColoredWriter<W> {
    #[inline]
    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        self.0.write_vectored(bufs)
    }
    #[inline]
    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        self.0.write_fmt(fmt)
    }
    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.0.write_all(buf)
    }
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)
    }
    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

impl<W: std::io::Write> PrettyWriter for ColoredWriter<W> {
    #[inline]
    fn set_style<S: std::fmt::Display>(&mut self, style: S) -> std::io::Result<()> {
        write!(self.0, "{style}")?;
        Ok(())
    }
}

pub struct BasicWriter<W>(W);

impl<W: std::io::Write> From<W> for BasicWriter<W> {
    #[inline]
    fn from(value: W) -> Self {
        Self(value)
    }
}

#[cfg(test)]
impl BasicWriter<Vec<u8>> {
    pub fn into_string(self) -> String {
        String::from_utf8(self.0).unwrap()
    }
}

impl<W: std::io::Write> std::io::Write for BasicWriter<W> {
    #[inline]
    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        self.0.write_vectored(bufs)
    }
    #[inline]
    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        self.0.write_fmt(fmt)
    }
    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.0.write_all(buf)
    }
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)
    }
    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

impl<W: std::io::Write> PrettyWriter for BasicWriter<W> {
    fn set_style<S: std::fmt::Display>(&mut self, _style: S) -> std::io::Result<()> {
        Ok(())
    }
}

pub trait PrettyDisplay {
    fn print<W: PrettyWriter>(&self, writer: &mut W) -> std::io::Result<()>;

    #[cfg(test)]
    fn to_basic_string(&self) -> std::io::Result<String> {
        let mut writer = BasicWriter::from(Vec::<u8>::new());
        self.print(&mut writer).unwrap();
        String::from_utf8(writer.0)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))
    }
}

impl<E: std::fmt::Display> PrettyDisplay for E {
    #[inline]
    fn print<W: PrettyWriter>(&self, writer: &mut W) -> std::io::Result<()> {
        write!(writer, "{self}")
    }
}
