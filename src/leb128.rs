use core::u8;
use core::convert::{From, AsRef};

use ctx::{self, TryFromCtx};
use error;

/// A variable length integer parsing `Ctx`, compatible with the standard integer endian-aware parsing context
pub type Leb128 = ctx::DefaultCtx;

/// This context instructs the underlying `Pread` implementor to parse as a variable length integer.
///
/// It currently is just the default ctx.
pub const LEB128: Leb128 = ctx::CTX;

#[derive(Debug, PartialEq, Copy, Clone)]
/// An unsigned leb128 integer
pub struct Uleb128 {
    value: u64,
    count: usize,
}

impl Uleb128 {
    #[inline]
    /// Return how many bytes this Uleb128 takes up in memory
    pub fn size(&self) -> usize {
        self.count
    }
    #[inline]
    /// Read a variable length u64 from `bytes` at `offset`
    pub fn read<B: AsRef<[u8]>>(bytes: &B, offset: &mut usize) -> error::Result<u64> {
        use Pread;
        let tmp = bytes.pread::<Uleb128>(*offset)?;
        *offset = *offset + tmp.size();
        Ok(tmp.into())
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

#[derive(Debug, PartialEq, Copy, Clone)]
/// An signed leb128 integer
pub struct Sleb128 {
    value: i64,
    count: usize,
}

impl Sleb128 {
    #[inline]
    /// Return how many bytes this Sleb128 takes up in memory
    pub fn size(&self) -> usize {
        self.count
    }
    #[inline]
    /// Read a variable length i64 from `bytes` at `offset`
    pub fn read<B: AsRef<[u8]>>(bytes: &B, offset: &mut usize) -> error::Result<i64> {
        use Pread;
        let tmp = bytes.pread::<Sleb128>(*offset)?;
        *offset = *offset + tmp.size();
        Ok(tmp.into())
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

// Below implementation heavily adapted from: https://github.com/fitzgen/leb128
const CONTINUATION_BIT: u8 = 1 << 7;
const SIGN_BIT: u8 = 1 << 6;

#[inline]
fn mask_continuation(byte: u8) -> u8 {
    byte & !CONTINUATION_BIT
}

// #[inline]
// fn mask_continuation_u64(val: u64) -> u8 {
//     let byte = val & (u8::MAX as u64);
//     mask_continuation(byte as u8)
// }

impl<'a> TryFromCtx<'a, (usize, Leb128)> for Uleb128 {
    type Error = error::Error;
    #[inline]
    fn try_from_ctx(src: &'a [u8], (offset, _ctx): (usize, Leb128)) -> error::Result<Self> {
        use pread::Pread;
        let mut result = 0;
        let mut shift = 0;
        let mut count = 0;
        loop {
            let byte: u8 = src.pread(offset + count)?;

            if shift == 63 && byte != 0x00 && byte != 0x01 {
                return Err(error::Error::BadInput((offset..offset+count), src.len(), "failed to parse"))
            }

            let low_bits = mask_continuation(byte) as u64;
            result |= low_bits << shift;

            count += 1;
            shift += 7;

            if byte & CONTINUATION_BIT == 0 {
                return Ok(Uleb128 { value: result, count: count });
            }
        }
    }
}

impl<'a> TryFromCtx<'a, (usize, Leb128)> for Sleb128 {
    type Error = error::Error;
    #[inline]
    fn try_from_ctx(src: &'a [u8], (offset, _): (usize, Leb128)) -> error::Result<Self> {
        use greater::Gread;
        let o = offset;
        let offset = &mut offset.clone();
        let mut result = 0;
        let mut shift = 0;
        let size = 64;
        let mut byte: u8;
        loop {
            byte = src.gread(offset)?;

            if shift == 63 && byte != 0x00 && byte != 0x7f {
                return Err(error::Error::BadInput((o..*offset), src.len(), "failed to parse"))
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
    fn uleb_size() {
        use super::super::Pread;
        let buf = [2u8 | CONTINUATION_BIT, 1];
        let bytes = &buf[..];
        let num = bytes.pread::<Uleb128>(0).unwrap();
        println!("num: {:?}", &num);
        assert_eq!(130u64, num.into());
        assert_eq!(num.size(), 2);

        let buf = [0x00,0x01];
        let bytes = &buf[..];
        let num = bytes.pread::<Uleb128>(0).unwrap();
        println!("num: {:?}", &num);
        assert_eq!(0u64, num.into());
        assert_eq!(num.size(), 1);

        let buf = [0x21];
        let bytes = &buf[..];
        let num = bytes.pread::<Uleb128>(0).unwrap();
        println!("num: {:?}", &num);
        assert_eq!(0x21u64, num.into());
        assert_eq!(num.size(), 1);
    }

    #[test]
    fn uleb128() {
        use super::super::Pread;
        let buf = [2u8 | CONTINUATION_BIT, 1];
        let bytes = &buf[..];
        let num = bytes.pread::<Uleb128>(0).expect("Should read Uleb128");
        assert_eq!(130u64, num.into());
        assert_eq!(386, bytes.pread_with::<u16>(0, LE).expect("Should read number"));
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
        assert!(bytes.pread_with::<Uleb128>(0, LEB128).is_err());
    }

    #[test]
    fn sleb128() {
        use super::super::Pread;
        let bytes = [0x7fu8 | CONTINUATION_BIT, 0x7e];
        let num: i64 = bytes.pread::<Sleb128>(0).expect("Should read Sleb128").into();
        assert_eq!(-129, num);
    }
}
