use std::{io, num::NonZeroUsize};

pub fn decode(src: &[u8], p: &[u8], n_sym: NonZeroUsize, len: usize) -> io::Result<Vec<u8>> {
    let mut dst = vec![0; len];
    let mut j = 0;

    if n_sym.get() <= 1 {
        dst.fill(p[0]);
    } else if n_sym.get() <= 2 {
        let mut v = 0;

        for (i, b) in dst.iter_mut().enumerate() {
            if i % 8 == 0 {
                v = src[j];
                j += 1;
            }

            *b = p[usize::from(v & 0x01)];
            v >>= 1;
        }
    } else if n_sym.get() <= 4 {
        let mut v = 0;

        for (i, b) in dst.iter_mut().enumerate() {
            if i % 4 == 0 {
                v = src[j];
                j += 1;
            }

            *b = p[usize::from(v & 0x03)];
            v >>= 2;
        }
    } else if n_sym.get() <= 16 {
        let mut v = 0;

        for (i, b) in dst.iter_mut().enumerate() {
            if i % 2 == 0 {
                v = src[j];
                j += 1;
            }

            *b = p[usize::from(v & 0x0f)];
            v >>= 4;
        }
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("expected n_sym to be <= 16, got {}", n_sym),
        ));
    }

    Ok(dst)
}
