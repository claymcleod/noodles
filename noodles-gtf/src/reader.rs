use std::io::{self, BufRead};

/// A GTF reader.
pub struct Reader<R> {
    inner: R,
}

impl<R> Reader<R>
where
    R: BufRead,
{
    /// Creates a GTF reader.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_gtf as gtf;
    /// let data = [];
    /// let reader = gtf::Reader::new(&data[..]);
    /// ```
    pub fn new(inner: R) -> Self {
        Self { inner }
    }

    /// Reads a raw GTF line.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io;
    /// use noodles_gtf as gtf;
    ///
    /// let data = b"sq0\tNOODLES\tgene\t8\t13\t.\t+\t.\tgene_id \"ndls0\"; transcript_id \"ndls0\"";
    /// let mut reader = gtf::Reader::new(&data[..]);
    ///
    /// let mut buf = String::new();
    /// reader.read_line(&mut buf)?;
    ///
    /// assert_eq!(
    ///     buf,
    ///     "sq0\tNOODLES\tgene\t8\t13\t.\t+\t.\tgene_id \"ndls0\"; transcript_id \"ndls0\""
    /// );
    /// # Ok::<_, io::Error>(())
    /// ```
    pub fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        read_line(&mut self.inner, buf)
    }
}

fn read_line<R>(reader: &mut R, buf: &mut String) -> io::Result<usize>
where
    R: BufRead,
{
    const LINE_FEED: char = '\n';
    const CARRIAGE_RETURN: char = '\r';

    match reader.read_line(buf) {
        Ok(0) => Ok(0),
        Ok(n) => {
            if buf.ends_with(LINE_FEED) {
                buf.pop();

                if buf.ends_with(CARRIAGE_RETURN) {
                    buf.pop();
                }
            }

            Ok(n)
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_line() -> io::Result<()> {
        let mut buf = String::new();

        let data = b"noodles\n";
        let mut reader = &data[..];
        read_line(&mut reader, &mut buf)?;
        assert_eq!(buf, "noodles");

        let data = b"noodles\r\n";
        let mut reader = &data[..];
        buf.clear();
        read_line(&mut reader, &mut buf)?;
        assert_eq!(buf, "noodles");

        let data = b"noodles";
        let mut reader = &data[..];
        buf.clear();
        read_line(&mut reader, &mut buf)?;
        assert_eq!(buf, "noodles");

        Ok(())
    }
}