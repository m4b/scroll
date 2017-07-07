use std::ops::{Deref, DerefMut};
use byte::*;

/// An unsigned leb128 integer
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Uleb128(u64);

impl Deref for Uleb128 {
    type Target = u64;

    fn deref(&self) -> &u64 {
        &self.0
    }
}

impl DerefMut for Uleb128 {
    fn deref_mut(&mut self) -> &mut u64 {
        &mut self.0
    }
}

impl Into<u64> for Uleb128 {
    #[inline]
    fn into(self) -> u64 {
        self.0
    }
}

/// An signed leb128 integer
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Sleb128(i64);

impl Deref for Sleb128 {
    type Target = i64;

    fn deref(&self) -> &i64 {
        &self.0
    }
}

impl DerefMut for Sleb128 {
    fn deref_mut(&mut self) -> &mut i64 {
        &mut self.0
    }
}

impl Into<i64> for Sleb128 {
    #[inline]
    fn into(self) -> i64 {
        self.0
    }
}

// Below implementation heavily adapted from:
// https://github.com/fitzgen/leb128

const CONTINUATION_BIT: u8 = 1 << 7;
const SIGN_BIT: u8 = 1 << 6;

#[inline]
fn mask_continuation(byte: u8) -> u8 {
    byte & !CONTINUATION_BIT
}

impl<'a> TryRead<'a> for Uleb128 {
    fn try_read(bytes: &'a [u8], _: ()) -> Result<(Self, usize)> {
        let mut result = 0;
        let mut shift = 0;
        let mut count = 0;
        loop {
            let byte = bytes.read_with::<u8>(&mut count, BE)?;

            if shift == 63 && byte != 0x00 && byte != 0x01 {
                return Err(Error::BadInput { err: "failed to parse" });
            }

            let low_bits = mask_continuation(byte) as u64;
            result |= low_bits << shift;

            shift += 7;

            if byte & CONTINUATION_BIT == 0 {
                return Ok((Uleb128(result), count));
            }
        }
    }
}

impl<'a> TryRead<'a> for Sleb128 {
    fn try_read(bytes: &'a [u8], _: ()) -> Result<(Self, usize)> {
        let o = 0;
        let offset = &mut 0;
        let mut result = 0;
        let mut shift = 0;
        let size = 64;
        let mut byte: u8;
        loop {
            byte = bytes.read_with::<u8>(offset, BE)?;

            if shift == 63 && byte != 0x00 && byte != 0x7f {
                return Err(Error::BadInput { err: "failed to parse" });
            }

            let low_bits = mask_continuation(byte) as i64;
            result |= low_bits << shift;
            shift += 7;

            if byte & CONTINUATION_BIT == 0 {
                break;
            }
        }

        if shift < size && (SIGN_BIT & byte) == SIGN_BIT {
            // Sign extend the result.
            result |= !0 << shift;
        }

        Ok(((Sleb128(result)), *offset - o))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use BytesExt;
    
    const CONTINUATION_BIT: u8 = 1 << 7;
    //const SIGN_BIT: u8 = 1 << 6;

    #[test]
    fn uleb_size() {
        let bytes = &[2u8 | CONTINUATION_BIT, 1];
        let (num, size) = Uleb128::try_read(bytes, ()).unwrap();
        println!("num: {:?}", num);
        assert_eq!(130u64, num.into());
        assert_eq!(size, 2);

        let bytes = &[0x00, 0x01];
        let (num, size) = Uleb128::try_read(bytes, ()).unwrap();
        println!("num: {:?}", num);
        assert_eq!(0u64, num.into());
        assert_eq!(size, 1);

        let bytes = &[0x21];
        let (num, size) = Uleb128::try_read(bytes, ()).unwrap();
        println!("num: {:?}", num);
        assert_eq!(0x21u64, num.into());
        assert_eq!(size, 1);
    }

    #[test]
    fn uleb128() {
        let bytes = &[2u8 | CONTINUATION_BIT, 1];
        let num = bytes
            .pread::<Uleb128>(0)
            .expect("Should read Uleb128");
        assert_eq!(130u64, num.into());
        assert_eq!(386,
                   bytes
                       .pread_with::<u16>(0, LE)
                       .expect("Should read number"));
    }

    #[test]
    fn uleb128_overflow() {
        let bytes = &[2u8 | CONTINUATION_BIT,
                      2 | CONTINUATION_BIT,
                      2 | CONTINUATION_BIT,
                      2 | CONTINUATION_BIT,
                      2 | CONTINUATION_BIT,
                      2 | CONTINUATION_BIT,
                      2 | CONTINUATION_BIT,
                      2 | CONTINUATION_BIT,
                      2 | CONTINUATION_BIT,
                      2 | CONTINUATION_BIT,
                      1];
        assert!(bytes.pread::<Uleb128>(0).is_err());
    }

    #[test]
    fn sleb128() {
        let bytes = &[0x7fu8 | CONTINUATION_BIT, 0x7e];
        let num: i64 = bytes
            .pread::<Sleb128>(0)
            .expect("Should read Sleb128")
            .into();
        assert_eq!(-129, num);
    }
}
