use std::io;

/// pack_uint_in is like pack_uint, but always uses the number of bytes given
/// to pack the number given.
///
/// `nbytes` must be >= pack_size(n) and <= 4, where `pack_size(n)` is the
/// smallest number of bytes that can store the integer given.
#[inline]
pub fn pack_u32<W: io::Write>(mut wtr: W, mut n: u32, nbytes: u8) -> io::Result<()> {
    assert!(1 <= nbytes && nbytes <= 4);

    let mut buf = [0u8; 4];
    for i in 0..nbytes {
        buf[i as usize] = n as u8;
        n = n >> 8;
    }
    wtr.write_all(&buf[..nbytes as usize])?;
    Ok(())
}

/// unpack_uint is the dual of pack_uint. It unpacks the integer at the current
/// position in `slice` after reading `nbytes` bytes.
///
/// `nbytes` must be >= 1 and <= 4.
#[inline]
pub fn unpack_u32(slice: &[u8], nbytes: u8) -> u32 {
    assert!(1 <= nbytes && nbytes <= 4);

    let mut n = 0;
    for (i, &b) in slice[..nbytes as usize].iter().enumerate() {
        n = n | ((b as u32) << (8 * i));
    }
    n
}

/// pack_size returns the smallest number of bytes that can encode `n`.
#[inline]
pub fn pack_size(n: u32) -> u8 {
    if n < 1 << 8 {
        1
    } else if n < 1 << 16 {
        2
    } else if n < 1 << 24 {
        3
    } else {
        4
    }
}
