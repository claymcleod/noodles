//! GFF reader and iterators.

mod lines;
mod records;

pub use self::{lines::Lines, records::Records};

use std::io::{self, BufRead};

const LINE_FEED: char = '\n';
const CARRIAGE_RETURN: char = '\r';

/// A GFF reader.
pub struct Reader<R> {
    inner: R,
}

impl<R> Reader<R>
where
    R: BufRead,
{
    /// Creates a GFF reader.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_gff as gff;
    /// let data = b"##gff-version 3\n";
    /// let mut reader = gff::Reader::new(&data[..]);
    /// ```
    pub fn new(inner: R) -> Self {
        Self { inner }
    }

    /// Returns a reference to the underlying reader.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_gff as gff;
    ///
    /// let data = b"##gff-version 3\n";
    /// let reader = gff::Reader::new(&data[..]);
    ///
    /// let _ = reader.get_ref();
    /// ```
    pub fn get_ref(&self) -> &R {
        &self.inner
    }

    /// Unwraps and returns the underlying reader.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io;
    /// use noodles_gff as gff;
    ///
    /// let data = b"##gff-version 3
    /// #format: gff3
    /// ";
    /// let mut reader = gff::Reader::new(&data[..]);
    /// reader.read_line(&mut String::new())?;
    ///
    /// assert_eq!(reader.into_inner(), b"#format: gff3\n");
    /// # Ok::<_, io::Error>(())
    /// ```
    pub fn into_inner(self) -> R {
        self.inner
    }

    /// Reads a raw GFF line.
    ///
    /// This reads from the underlying stream until a newline is reached and appends it to the
    /// given buffer, sans the final newline character. The buffer can subsequently be parsed as a
    /// [`crate::Line`].
    ///
    /// It is more ergonomic to read records using an iterator (see [`Self::lines`]), but using
    /// this method allows control of the line buffer and whether the raw line should be parsed.
    ///
    /// If successful, the number of bytes read is returned. If the number of bytes read is 0, the
    /// stream reached EOF.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io;
    /// use noodles_gff as gff;
    ///
    /// let data = b"##gff-version 3
    /// sq0\tNOODLES\tgene\t8\t13\t.\t+\t.\tgene_id=ndls0;gene_name=gene0
    /// ";
    /// let mut reader = gff::Reader::new(&data[..]);
    ///
    /// let mut buf = String::new();
    /// reader.read_line(&mut buf)?;
    /// assert_eq!(buf, "##gff-version 3");
    /// # Ok::<_, io::Error>(())
    /// ```
    pub fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        read_line(&mut self.inner, buf)
    }

    /// Returns an iterator over lines starting from the current stream position.
    ///
    /// When using this, the caller is responsible to stop reading at either EOF or when the
    /// `FASTA` directive is read, whichever comes first.
    ///
    /// Unlike [`Self::read_line`], each line is parsed as a [`crate::Line`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io;
    /// use noodles_gff as gff;
    ///
    /// let data = b"##gff-version 3
    /// sq0\tNOODLES\tgene\t8\t13\t.\t+\t.\tgene_id=ndls0;gene_name=gene0
    /// ";
    /// let mut reader = gff::Reader::new(&data[..]);
    /// let mut lines = reader.lines();
    ///
    /// let line = lines.next().transpose()?;
    /// assert!(matches!(line, Some(gff::Line::Directive(_))));
    ///
    /// let line = lines.next().transpose()?;
    /// assert!(matches!(line, Some(gff::Line::Record(_))));
    ///
    /// assert!(lines.next().is_none());
    /// # Ok::<_, io::Error>(())
    /// ```
    pub fn lines(&mut self) -> Lines<'_, R> {
        Lines::new(self)
    }

    /// Returns an iterator over records starting from the current stream position.
    ///
    /// This filters lines for only records. It stops at either EOF or when the `FASTA` directive
    /// is read, whichever comes first.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io;
    /// use noodles_gff as gff;
    ///
    /// let data = b"##gff-version 3
    /// sq0\tNOODLES\tgene\t8\t13\t.\t+\t.\tgene_id=ndls0;gene_name=gene0
    /// ";
    /// let mut reader = gff::Reader::new(&data[..]);
    /// let mut records = reader.records();
    ///
    /// assert!(records.next().transpose()?.is_some());
    /// assert!(records.next().is_none());
    /// # Ok::<_, io::Error>(())
    /// ```
    pub fn records(&mut self) -> Records<'_, R> {
        Records::new(self.lines())
    }
}

fn read_line<R>(reader: &mut R, buf: &mut String) -> io::Result<usize>
where
    R: BufRead,
{
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
    fn test_records() -> io::Result<()> {
        let data = b"\
##gff-version 3
sq0\tNOODLES\tgene\t8\t13\t.\t+\t.\tgene_id=ndls0;gene_name=gene0
";

        let mut reader = Reader::new(&data[..]);
        let mut n = 0;

        for result in reader.records() {
            let _ = result?;
            n += 1;
        }

        assert_eq!(n, 1);

        Ok(())
    }

    #[test]
    fn test_records_with_fasta_directive() -> io::Result<()> {
        let data = b"\
##gff-version 3
sq0\tNOODLES\tgene\t8\t13\t.\t+\t.\tgene_id=ndls0;gene_name=gene0
##FASTA
>sq0
ACGT
";

        let mut reader = Reader::new(&data[..]);
        let mut n = 0;

        for result in reader.records() {
            let _ = result?;
            n += 1;
        }

        assert_eq!(n, 1);

        Ok(())
    }

    #[test]
    fn test_read_line() -> io::Result<()> {
        fn t(buf: &mut String, mut reader: &[u8], expected: &str) -> io::Result<()> {
            buf.clear();
            read_line(&mut reader, buf)?;
            assert_eq!(buf, expected);
            Ok(())
        }

        let mut buf = String::new();

        t(&mut buf, b"noodles\n", "noodles")?;
        t(&mut buf, b"noodles\r\n", "noodles")?;
        t(&mut buf, b"noodles", "noodles")?;

        Ok(())
    }
}
