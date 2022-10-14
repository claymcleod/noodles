use std::{
    ffi::{OsStr, OsString},
    fs::File,
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
};

use super::IndexedReader;
use crate::fai;

/// An indexed FASTA reader builder.
#[derive(Default)]
pub struct Builder {
    index: Option<fai::Index>,
}

impl Builder {
    /// Sets an index.
    pub fn set_index(mut self, index: fai::Index) -> Self {
        self.index = Some(index);
        self
    }

    /// Builds an indexed FASTA reader from a path.
    pub fn build_from_path<P>(self, src: P) -> io::Result<IndexedReader<BufReader<File>>>
    where
        P: AsRef<Path>,
    {
        let index = match self.index {
            Some(index) => index,
            None => {
                let index_src = build_index_src(&src);
                fai::read(index_src)?
            }
        };

        let reader = File::open(&src).map(BufReader::new)?;
        Ok(IndexedReader::new(reader, index))
    }

    /// Builds an indexed FASTA reader from a reader.
    pub fn build_from_reader<R>(self, reader: R) -> io::Result<IndexedReader<R>>
    where
        R: BufRead,
    {
        let index = self
            .index
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing index"))?;

        Ok(IndexedReader::new(reader, index))
    }
}

fn build_index_src<P>(src: P) -> PathBuf
where
    P: AsRef<Path>,
{
    const EXT: &str = "fai";
    push_ext(src.as_ref().into(), EXT)
}

fn push_ext<S>(path: PathBuf, ext: S) -> PathBuf
where
    S: AsRef<OsStr>,
{
    let mut s = OsString::from(path);
    s.push(".");
    s.push(ext);
    PathBuf::from(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_index_src() {
        assert_eq!(build_index_src("ref.fa"), PathBuf::from("ref.fa.fai"));
    }
}