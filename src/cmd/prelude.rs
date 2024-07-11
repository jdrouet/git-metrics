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

pub type Style = &'static str;

pub trait PrettyWriter: std::io::Write + Sized {
    fn set_style(&mut self, style: Style) -> std::io::Result<usize>;

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
    fn from(value: W) -> Self {
        Self(value)
    }
}

impl From<ColoredWriter<String>> for String {
    fn from(value: ColoredWriter<String>) -> Self {
        value.0
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
    fn set_style(&mut self, style: Style) -> std::io::Result<usize> {
        self.0.write(style.as_bytes())
    }
}

pub struct BasicWriter<W>(W);

impl<W: std::io::Write> From<W> for BasicWriter<W> {
    fn from(value: W) -> Self {
        Self(value)
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
    fn set_style(&mut self, _style: Style) -> std::io::Result<usize> {
        Ok(0)
    }
}

pub trait PrettyDisplay {
    fn print<W: PrettyWriter>(&self, writer: &mut W) -> std::io::Result<()>;

    fn to_colored_string(&self) -> std::io::Result<String> {
        let mut writer = ColoredWriter::from(Vec::<u8>::new());
        self.print(&mut writer).unwrap();
        String::from_utf8(writer.0)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))
    }

    fn to_basic_string(&self) -> std::io::Result<String> {
        let mut writer = BasicWriter::from(Vec::<u8>::new());
        self.print(&mut writer).unwrap();
        String::from_utf8(writer.0)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))
    }
}

impl<E: std::fmt::Display> PrettyDisplay for E {
    fn print<W: PrettyWriter>(&self, writer: &mut W) -> std::io::Result<()> {
        write!(writer, "{self}")
    }
}
