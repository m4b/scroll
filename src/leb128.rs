use core::u8;
use core::convert::{From, AsRef};

use ctx::TryFromCtx;
use measure;
use error;
use endian;

#[derive(Debug, Copy, Clone, Default)]
/// A variable length integer parsing `Ctx`
// TODO: I think the way to go is just type alias this to endian::Endian, set the const to NATIVE, drop the u64 ctx impl, and allow it to be used alongside endian ctxs, e.g.:
//pub type Leb128 = endian::Endian;
// ...tested and it works
pub struct Leb128 {}

/// This context instructs the underlying Scroll (Buffer, Readable) to parse as a variable length integer
pub const LEB128: Leb128 = Leb128 {};
//pub const LEB128: Leb128 = endian::NATIVE;

#[derive(Debug, PartialEq, Copy, Clone)]
/// An unsigned leb128 integer
pub struct Uleb128 {
    value: u64,
    count: usize,
}

impl Uleb128 {
    #[inline]
    pub fn size(&self) -> usize {
        self.count
    }
}

impl AsRef<u64> for Uleb128 {
    fn as_ref(&self) -> &u64 {
        &self.value
    }
}

impl From<Uleb128> for u64 {
    #[inline]
    fn from(uleb128: Uleb128) -> u64 {
        uleb128.value
    }
}

impl measure::Measure for Uleb128 {
    type Units = usize;
    #[inline]
    fn measure (&self) -> usize {
        self.count
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
/// An signed leb128 integer
pub struct Sleb128 {
    value: i64,
    count: usize,
}

impl Sleb128 {
    #[inline]
    pub fn size(&self) -> usize {
        self.count
    }
}

impl AsRef<i64> for Sleb128 {
    fn as_ref(&self) -> &i64 {
        &self.value
    }
}

impl From<Sleb128> for i64 {
    #[inline]
    fn from(sleb128: Sleb128) -> i64 {
        sleb128.value
    }
}

impl measure::Measure for Sleb128 {
    type Units = usize;
    #[inline]
    fn measure (&self) -> usize {
        self.count
    }
}

// Below implementation heavily adapted from: https://github.com/fitzgen/leb128
const CONTINUATION_BIT: u8 = 1 << 7;
const SIGN_BIT: u8 = 1 << 6;

#[inline]
fn mask_continuation(byte: u8) -> u8 {
    byte & !CONTINUATION_BIT
}

#[inline]
fn mask_continuation_u64(val: u64) -> u8 {
    let byte = val & (u8::MAX as u64);
    mask_continuation(byte as u8)
}

impl TryFromCtx<(usize, Leb128)> for Uleb128 {
    type Error = error::Error;
    #[inline]
    fn try_from_ctx(src: &[u8], ctx: (usize, Leb128)) -> error::Result<Self> {
        use pread::Pread;
        let offset = ctx.0;
        let mut result = 0;
        let mut shift = 0;
        let mut count = 0;
        loop {
            let byte: u8 = src.pread_into(offset + count)?;

            if shift == 63 && byte != 0x00 && byte != 0x01 {
                return Err(error::Error::BadInput(format!("Failed to parse uleb128 from: {:?}", &src[offset..(offset+count)])));
            }

            let low_bits = mask_continuation(byte) as u64;
            result |= low_bits << shift;

            if byte & CONTINUATION_BIT == 0 {
                return Ok(Uleb128 { value: result, count: count });
            }
            count += 1;
            shift += 7;
        }
    }
}

impl TryFromCtx<(usize, Leb128)> for Sleb128 {
    type Error = error::Error;
    #[inline]
    fn try_from_ctx(src: &[u8], (offset, _): (usize, Leb128)) -> error::Result<Self> {
        use greater::Gread;
        let o = offset;
        let offset = &mut offset.clone();
        let mut result = 0;
        let mut shift = 0;
        let size = 64;
        let mut byte: u8;
        loop {
            byte = src.gread_into(offset)?;

            if shift == 63 && byte != 0x00 && byte != 0x7f {
                return Err(error::Error::BadInput(format!("Failed to parse sleb128 from: {:?}", &src[o..*offset])));
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
        Ok(Sleb128{ value: result, count: *offset - o })
    }
}

#[cfg(test)]
mod tests {
    use super::{LEB128, Uleb128, Sleb128};
    use super::super::LE;

    #[doc(hidden)]
    const CONTINUATION_BIT: u8 = 1 << 7;
    #[doc(hidden)]
    const SIGN_BIT: u8 = 1 << 6;

    #[test]
    fn uleb128() {
        use super::super::Pread;
        let buf = [2u8 | CONTINUATION_BIT, 1];
        let bytes = &buf[..];
        let num = bytes.pread_into::<Uleb128>(0).expect("Should read Uleb128");
        assert_eq!(130u64, num.into());
        assert_eq!(386, bytes.pread::<u16>(0, LE).expect("Should read number"));
    }

    #[test]
    fn uleb128_overflow() {
        use super::super::Pread;
        let buf = [2u8 | CONTINUATION_BIT,
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
        let bytes = &buf[..];
        assert!(bytes.pread::<Uleb128>(0, LEB128).is_err());
    }

    #[test]
    fn sleb128() {
        use super::super::Pread;
        let bytes = [0x7fu8 | CONTINUATION_BIT, 0x7e];
        let num: i64 = bytes.pread_into::<Sleb128>(0).expect("Should read Sleb128").into();
        assert_eq!(-129, num);
    }
}
