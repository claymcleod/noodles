use std::{
    io::{self, Write},
    mem,
};

use byteorder::{LittleEndian, WriteBytesExt};

use crate::writer::num::write_uint7;

pub fn encode(src: &[u8], n: usize) -> io::Result<(Vec<u32>, Vec<u8>)> {
    use super::{
        build_cumulative_frequencies, build_frequencies, normalize, normalize_frequencies, update,
    };

    const LOWER_BOUND: u32 = 0x8000;

    let frequencies = build_frequencies(src);

    let freq = normalize_frequencies(&frequencies);
    let cfreq = build_cumulative_frequencies(&freq);

    let mut buf = Vec::new();
    let mut states = vec![LOWER_BOUND; n];

    for (i, &sym) in src.iter().enumerate().rev() {
        let j = i % states.len();

        let mut x = states[j];
        let freq_i = freq[usize::from(sym)];
        let cfreq_i = cfreq[usize::from(sym)];

        x = normalize(&mut buf, x, freq_i, 12)?;
        states[j] = update(x, cfreq_i, freq_i, 12);
    }

    let mut dst = Vec::with_capacity(states.len() * mem::size_of::<u32>() + buf.len());

    for &state in &states {
        dst.write_u32::<LittleEndian>(state)?;
    }

    dst.extend(buf.iter().rev());

    Ok((freq, dst))
}

pub fn write_frequencies<W>(writer: &mut W, frequencies: &[u32]) -> io::Result<()>
where
    W: Write,
{
    use super::write_alphabet;

    write_alphabet(writer, frequencies)?;

    for &f in frequencies {
        if f > 0 {
            write_uint7(writer, f)?;
        }
    }

    Ok(())
}
