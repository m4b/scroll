use core::u8;
use core::convert::{From, AsRef};

use ctx::TryFromCtx;
use measure;
use error;

#[derive(Debug, Copy, Clone, Default)]
/// A variable length integer parsing `Ctx`
// TODO: might want to just type alias this to endian::Endian, set the const to LE, drop the u64 ctx impl, and allow it to be used alongside endian ctxs
pub struct Leb128 {}

/// This context instructs the underlying Scroll (Buffer, Readable) to parse as a variable length integer
pub const LEB128: Leb128 = Leb128 {};

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

impl TryFromCtx<(usize, Leb128)> for u64 {
    type Error = error::Error;
    #[inline]
    fn try_from_ctx(src: &[u8], ctx: (usize, Leb128)) -> error::Result<Self> {
        use pread::Pread;

        let mut offset = ctx.0;
        let mut result = 0;
        let mut shift = 0;

        loop {
            let byte: u8 = src.pread_into(offset)?;

            if shift == 63 && byte != 0x00 && byte != 0x01 {
                return Err(error::Error::BadInput(format!("Failed to parse uleb128 from: {:?}", &src[ctx.0..offset])));
            }

            let low_bits = mask_continuation(byte) as u64;
            result |= low_bits << shift;

            if byte & CONTINUATION_BIT == 0 {
                return Ok(result);
            }
            offset += 1;
            shift += 7;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LEB128, Uleb128};
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
        assert_eq!(130, bytes.pread::<u64>(0, LEB128).expect("Should read uleb128 number"));
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
        //let res = bytes.pread::<u64>(0, LEB128).unwrap();
        assert!(bytes.pread::<u64>(0, LEB128).is_err());
    }
}
