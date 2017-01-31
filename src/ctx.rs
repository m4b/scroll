//! Generic context-aware conversion traits, for automatic _downstream_ extension of `Pread`, et. al
//!
//! # Discussion
//! Let us postulate that there is a deep relationship between trying to make something from something else, and
//! the generic concept of "parsing" or "reading".
//!
//! Further let us suppose that central to this notion is also the importance of codified failure, in addition to a
//! _context_ in which this reading/parsing/from-ing occurs.
//!
//! A context in this case is a set of values, preconditions, etc., which make the parsing meaningful for a particular type of input.
//!
//! For example, to make this more concrete, when parsing an array of bytes, for a concrete numeric type, say `u32`,
//! we might be interested in parsing this value at a given offset in a "big endian" byte order.
//! Consequently, we might say our context is a 2-tuple, `(offset, endianness)`.
//!
//! Another example might be parsing a `&str` from a stream of bytes, which would require both an offset and a size.
//! Still another might be parsing a list of ELF dynamic table entries from a byte array - which requires both something called
//! a "load bias" and an array of program headers _maybe_ pointing to their location.
//!
//! Scroll builds on this idea by providing a generic context as a parameter to conversion traits
//! (the parsing `Ctx`, akin to a "contextual continuation"), which is typically sufficient to model a large amount of data constructs using this single conversion trait, but with different `Ctx` implementations.
//! In particular, parsing a u64, a leb128, a byte, a custom datatype, can all be modelled using a single trait - `TryFromCtx<Ctx, This, Error = E>`. What this enables is a _single method_ for parsing disparate datatypes out of a given type, with a given context - **without** re-implementing the reader functions, and all done at compile time, without runtime dispatch!
//!
//! Consequently, instead of "hand specializing" function traits by appending `pread_<type>`,
//! almost all of the complexity of `Pread` and its sister trait `Gread` can be collapsed
//! into two methods (`pread_with` and `pread_slice`).
//!
//! # Example
//!
//! Suppose we have a datatype and we want to specify how to parse or serialize this datatype out of some arbitrary
//! byte buffer. In order to do this, we need to provide a `TryFromCtx` impl for our datatype. In particular, if we
//! do this for the `[u8]` target, using the convention `(usize, YourCtx)`, you will automatically get access to
//! calling `pread_with::<YourDatatype>` on arrays of bytes.
//!
//! ```rust
//! use scroll::{self, ctx, Pread, BE};
//! struct Data<'a> {
//!   name: &'a str,
//!   id: u32,
//! }
//!
//! // we could use a `(usize, endian::Scroll)` if we wanted
//! #[derive(Debug, Clone, Copy, Default)]
//! struct DataCtx { pub size: usize, pub endian: scroll::Endian }
//!
//! impl<'a> ctx::TryFromCtx<'a, (usize, DataCtx)> for Data<'a> {
//!   type Error = scroll::Error;
//!   fn try_from_ctx (src: &'a [u8], (offset, DataCtx {size, endian}): (usize, DataCtx))
//!     -> Result<Self, Self::Error> {
//!     let name = src.pread_slice::<str>(offset, size)?;
//!     let id = src.pread_with(offset+size, endian)?;
//!     Ok(Data { name: name, id: id })
//!   }
//! }
//!
//! let bytes = scroll::Buffer::new(b"UserName\x01\x02\x03\x04");
//! let data = bytes.pread_with::<Data>(0, DataCtx { size: 8, endian: BE }).unwrap();
//! assert_eq!(data.id, 0x01020304);
//! assert_eq!(data.name.to_string(), "UserName".to_string());
//!
//! ```

use core::ptr::copy_nonoverlapping;
use core::mem::transmute;
use core::str;

use error;
use endian;

/// The default parsing context; use this when the context isn't important for your datatype
pub type DefaultCtx = endian::Endian;

/// Convenience constant for the default parsing context
pub const CTX: DefaultCtx = endian::NATIVE;

/// The parsing context for converting a byte sequence to a `&str`
///
/// `StrCtx` specifies what byte delimiter to use, and defaults to C-style null terminators. Be careful.
#[derive(Debug, Copy, Clone)]
pub struct StrCtx {
    pub delimiter: u8
}

/// A C-style, null terminator based delimiter for a `StrCtx`
pub const NULL: StrCtx = StrCtx { delimiter: 0 };
/// A space-based delimiter for a `StrCtx`
pub const SPACE: StrCtx = StrCtx { delimiter: 0x20 };
/// A newline-based delimiter for a `StrCtx`
pub const RET: StrCtx = StrCtx { delimiter: 0x0a };
/// A tab-based delimiter for a `StrCtx`
pub const TAB: StrCtx = StrCtx { delimiter: 0x09 };

impl Default for StrCtx {
    #[inline]
    fn default() -> Self {
        NULL
    }
}

impl From<u8> for StrCtx {
    fn from(delimiter: u8) -> Self {
        StrCtx { delimiter: delimiter }
    }
}

/// Reads `Self` from `This` using the context `Ctx`
pub trait FromCtx<Ctx: Copy = DefaultCtx, This: ?Sized = [u8]> where Self: Sized {
    #[inline]
    fn from_ctx(this: &This, ctx: Ctx) -> Self;
}

/// Tries to read `Self` from `This` using the context `Ctx`
pub trait TryFromCtx<'a, Ctx: Copy = (usize, DefaultCtx), This: ?Sized = [u8]> where Self: 'a + Sized {
    type Error;
    #[inline]
    fn try_from_ctx(from: &'a This, ctx: Ctx) -> Result<Self, Self::Error>;
}

/// Writes `Self` into `This` using the context `Ctx`
pub trait IntoCtx<Ctx: Copy = DefaultCtx, This: ?Sized = [u8]>: Sized {
    fn into_ctx(self, &mut This, ctx: Ctx);
}

/// Tries to write `Self` into `This` using the context `Ctx`
pub trait TryIntoCtx<Ctx: Copy = (usize, DefaultCtx), This: ?Sized = [u8]>: Sized {
    type Error;
    fn try_into_ctx(self, &mut This, ctx: Ctx) -> Result<(), Self::Error>;
}

pub trait RefFrom<This: ?Sized = [u8], I = usize> {
    type Error;
    #[inline]
    fn ref_from(from: &This, offset: I, count: I) -> Result<&Self, Self::Error>;
}

/// Tries to read a reference to `Self` from `This` using the context `Ctx`
pub trait TryRefFromCtx<Ctx: Copy = (usize, usize, DefaultCtx), This: ?Sized = [u8]> {
    type Error;
    #[inline]
    fn try_ref_from_ctx(from: &This, ctx: Ctx) -> Result<&Self, Self::Error>;
}

/// Tries to write a reference to `Self` into `This` using the context `Ctx`
pub trait TryRefIntoCtx<Ctx: Copy = (usize, usize, DefaultCtx), This: ?Sized = [u8]>: Sized {
    type Error;
    fn try_ref_into_ctx(self, &mut This, ctx: Ctx) -> Result<(), Self::Error>;
}

impl<T> TryRefFromCtx<(usize, usize, super::Endian), T> for [u8] where T: AsRef<[u8]> {
    type Error = error::Error;
    #[inline]
    fn try_ref_from_ctx(b: &T, ctx: (usize, usize, super::Endian)) -> error::Result<&[u8]> {
       let (offset, count) = (ctx.0, ctx.1);
        let b = b.as_ref();
        if offset + count > b.len () {
            Err(error::Error::BadOffset(format!("count: {}, offset: {},  len: {}", count, offset, b.len())).into())
        } else {
            Ok(&b[offset..(offset+count)])
        }
    }
}

// uses the default generics
impl TryRefFromCtx for [u8] {
    type Error = error::Error;
    #[inline]
    fn try_ref_from_ctx(b: &[u8], ctx: (usize, usize, super::Endian)) -> error::Result<&[u8]> {
        let (offset, count) = (ctx.0, ctx.1);
        if offset + count > b.len () {
            Err(error::Error::BadOffset(format!("count: {}, offset: {},  len: {}", count, offset, b.len())).into())
        } else {
            Ok(&b[offset..(offset+count)])
        }
    }
}

impl TryRefFromCtx for str {
    type Error = error::Error;
    #[inline]
    fn try_ref_from_ctx(b: &[u8], (offset, count, _): (usize, usize, super::Endian)) -> error::Result<&str> {
        if offset + count > b.len () {
            Err(error::Error::BadOffset(format!("count: {}, offset: {},  len: {}", count, offset, b.len())).into())
        } else {
            let bytes = &b[offset..(offset+count)];
            str::from_utf8(bytes).map_err(| err | {
                let up_to = err.valid_up_to();
                error::Error::BadInput(format!("invalid utf8: requested: {:?} valid len: {:?} remaining: {:?}", offset..(offset+count), offset..(offset+up_to), (offset+up_to)..(offset+count)))
            })
        }
    }
}

impl<T> TryRefFromCtx<(usize, usize, super::Endian), T> for str where T: AsRef<[u8]> {
    type Error = error::Error;
    #[inline]
    fn try_ref_from_ctx(b: &T, (offset, count, _): (usize, usize, super::Endian)) -> error::Result<&str> {
        let b = b.as_ref();
        if offset + count > b.len () {
            Err(error::Error::BadOffset(format!("count: {}, offset: {},  len: {}", count, offset, b.len())).into())
        } else {
            let bytes = &b[offset..(offset+count)];
            str::from_utf8(bytes).map_err(| err | {
                let up_to = err.valid_up_to();
                error::Error::BadInput(format!("invalid utf8: requested: {:?} valid len: {:?} remaining: {:?}", offset..(offset+count), offset..(offset+up_to), (offset+up_to)..(offset+count)))
            })
        }
    }
}

macro_rules! signed_to_unsigned {
    (i8) =>  {u8 };
    (u8) =>  {u8 };
    (i16) => {u16};
    (u16) => {u16};
    (i32) => {u32};
    (u32) => {u32};
    (i64) => {u64};
    (u64) => {u64};
    (f32) => {u32};
    (f64) => {u64};
}

macro_rules! write_into {
    ($typ:ty, $size:expr, $n:expr, $dst:expr, $endian:expr) => ({
        unsafe {
            let bytes = transmute::<$typ, [u8; $size]>(if $endian.is_little() { $n.to_le() } else { $n.to_be() });
            copy_nonoverlapping((&bytes).as_ptr(), $dst.as_mut_ptr(), $size);
        }
    });
}

macro_rules! into_ctx_impl {
    ($typ:tt, $size:expr, $ctx:ty) => {
        impl IntoCtx for $typ {
            #[inline]
            fn into_ctx(self, dst: &mut [u8], le: super::Endian) {
                write_into!($typ, $size, self, dst, le);
            }
        }
        impl TryIntoCtx<(usize, $ctx)> for $typ where $typ: IntoCtx<$ctx> {
            type Error = error::Error;
            #[inline]
            fn try_into_ctx(self, dst: &mut [u8], ctx: (usize, super::Endian)) -> error::Result<()> {
                let offset = ctx.0;
                let le = ctx.1;
                if offset + $size > dst.len () {
                    return Err(error::Error::BadOffset(format!("size_of: {}, offset: {},  len: {}", $size, offset, dst.len())).into())
                }
                <$typ as IntoCtx<$ctx>>::into_ctx(self, &mut dst[offset..(offset+$size)], le);
                Ok(())
            }
        }
    }
}

macro_rules! from_ctx_impl {
    ($typ:tt, $size:expr, $ctx:ty) => {
        impl FromCtx<$ctx> for $typ {
            #[inline]
            fn from_ctx(src: &[u8], le: $ctx) -> Self {
                let mut data: signed_to_unsigned!($typ) = 0;
                unsafe {
                    copy_nonoverlapping(
                        src.as_ptr(),
                        &mut data as *mut signed_to_unsigned!($typ) as *mut u8,
                        $size);
                }
                (if le.is_little() { data.to_le() } else { data.to_be() }) as $typ
            }
        }

        impl<'a> TryFromCtx<'a, (usize, $ctx)> for $typ where $typ: FromCtx<$ctx> {
            type Error = error::Error;
            #[inline]
            fn try_from_ctx(src: &'a [u8], ctx: (usize, $ctx)) -> error::Result<Self> {
                let offset = ctx.0;
                let le = ctx.1;
                if offset + $size > src.len () {
                    return Err(error::Error::BadOffset(format!("value size: {}, offset: {}, src len: {}", $size, offset, src.len())).into())
                }
                Ok(FromCtx::from_ctx(&src[offset..(offset + $size)], le))
            }
        }
        // as ref
        impl<T> FromCtx<$ctx, T> for $typ where T: AsRef<[u8]> {
            #[inline]
            fn from_ctx(src: &T, le: $ctx) -> Self {
                let src = src.as_ref();
                let mut data: signed_to_unsigned!($typ) = 0;
                unsafe {
                    copy_nonoverlapping(
                        src.as_ptr(),
                        &mut data as *mut signed_to_unsigned!($typ) as *mut u8,
                        $size);
                }
                (if le.is_little() { data.to_le() } else { data.to_be() }) as $typ
            }
        }

        impl<'a, T> TryFromCtx<'a, (usize, $ctx), T> for $typ where $typ: FromCtx<$ctx, T>, T: AsRef<[u8]> {
            type Error = error::Error;
            #[inline]
            fn try_from_ctx(src: &'a T, ctx: (usize, $ctx)) -> error::Result<Self> {
                let src = src.as_ref();
                let offset = ctx.0;
                let le = ctx.1;
                if offset + $size > src.len () {
                    return Err(error::Error::BadOffset(format!("value size: {}, offset: {}, src len: {}", $size, offset, src.len())).into())
                }
                Ok(FromCtx::from_ctx(&src[offset..(offset + $size)], le))
            }
        }

    };
}

macro_rules! ctx_impl {
    ($typ:tt, $size:expr) => {
        from_ctx_impl!($typ, $size, super::Endian);
     };
}

ctx_impl!(u8, 1);
ctx_impl!(i8, 1);
ctx_impl!(u16, 2);
ctx_impl!(i16, 2);
ctx_impl!(u32, 4);
ctx_impl!(i32, 4);
ctx_impl!(u64, 8);
ctx_impl!(i64, 8);

macro_rules! from_ctx_float_impl {
    ($typ:tt, $size:expr, $ctx:ty) => {
        impl FromCtx<$ctx> for $typ {
            #[inline]
            fn from_ctx(src: &[u8], le: $ctx) -> Self {
                let mut data: signed_to_unsigned!($typ) = 0;
                unsafe {
                    copy_nonoverlapping(
                        src.as_ptr(),
                        &mut data as *mut signed_to_unsigned!($typ) as *mut u8,
                        $size);
                    transmute((if le.is_little() { data.to_le() } else { data.to_be() }))
                }
            }
        }

        impl<'a> TryFromCtx<'a, (usize, $ctx)> for $typ where $typ: FromCtx<$ctx> {
            type Error = error::Error;
            #[inline]
            fn try_from_ctx(src: &'a [u8], (offset, le): (usize, $ctx)) -> error::Result<Self> {
                if offset + $size > src.len () {
                    return Err(error::Error::BadOffset(format!("value size: {}, offset: {}, src len: {}", $size, offset, src.len())).into())
                }
                Ok(FromCtx::from_ctx(&src[offset..(offset + $size)], le))
            }
        }
    }
}

from_ctx_float_impl!(f32, 4, super::Endian);
from_ctx_float_impl!(f64, 8, super::Endian);

into_ctx_impl!(u8,  1, super::Endian);
into_ctx_impl!(i8,  1, super::Endian);
into_ctx_impl!(u16, 2, super::Endian);
into_ctx_impl!(i16, 2, super::Endian);
into_ctx_impl!(u32, 4, super::Endian);
into_ctx_impl!(i32, 4, super::Endian);
into_ctx_impl!(u64, 8, super::Endian);
into_ctx_impl!(i64, 8, super::Endian);

macro_rules! into_ctx_float_impl {
    ($typ:tt, $size:expr, $ctx:ty) => {
        impl IntoCtx for $typ {
            #[inline]
            fn into_ctx(self, dst: &mut [u8], le: super::Endian) {
                write_into!(signed_to_unsigned!($typ), $size, transmute::<$typ, signed_to_unsigned!($typ)>(self), dst, le);
            }
        }
        impl TryIntoCtx<(usize, $ctx)> for $typ where $typ: IntoCtx<$ctx> {
            type Error = error::Error;
            #[inline]
            fn try_into_ctx(self, dst: &mut [u8], (offset, le): (usize, super::Endian)) -> error::Result<()> {
                if offset + $size > dst.len () {
                    return Err(error::Error::BadOffset(format!("size_of: {}, offset: {},  len: {}", $size, offset, dst.len())).into())
                }
                <$typ as IntoCtx<$ctx>>::into_ctx(self, &mut dst[offset..(offset+$size)], le);
                Ok(())
            }
        }
    }
}

into_ctx_float_impl!(f32, 4, super::Endian);
into_ctx_float_impl!(f64, 8, super::Endian);


#[inline(always)]
fn get_str_delimiter_offset(bytes: &[u8], idx: usize, delimiter: u8) -> usize {
    let len = bytes.len();
    let mut i = idx;
    let mut byte = bytes[i];
    // TODO: this is still a hack and getting worse and worse - this hack has come from dryad -> goblin -> scroll :D
    if byte == delimiter {
        return i;
    }
    while byte != delimiter && i < len {
        byte = bytes[i];
        i += 1;
    }
    // we drop the terminator/delimiter unless we're at the end and the byte isn't the terminator
    if i < len || bytes[i - 1] == delimiter {
        i -= 1;
    }
    i
}

impl<'a> TryFromCtx<'a, (usize, StrCtx)> for &'a str {
    type Error = error::Error;
    #[inline]
    /// Read a `&str` from `src` using `delimiter`
    fn try_from_ctx(src: &'a [u8], (offset, StrCtx {delimiter}): (usize, StrCtx)) -> error::Result<Self> {
        let len = src.len();
        if offset >= len {
            return Err(error::Error::BadOffset(format!("offset: {} >= src len: {}", offset, len)).into())
        }
        let delimiter_offset = get_str_delimiter_offset(src, offset, delimiter);
        let count = delimiter_offset - offset;
        if count == 0 { return Ok("") }
        // we do not need to check if offset + count > len because get_str_delimiter_offset returns a valid index
        // if offset + count > len {
        //     return Err(error::Error::BadOffset(format!("str len: {}, offset: {}, src len: {}", count, offset, len)).into());
        // }
        let bytes = &src[offset..(offset+count)];
        str::from_utf8(bytes).map_err(| err | {
            let up_to = err.valid_up_to();
            error::Error::BadInput(format!("invalid utf8: requested: {:?} valid len: {:?} remaining: {:?}", offset..(offset+count), offset..(offset+up_to), (offset+up_to)..(offset+count)))
        })
    }
}

impl<'a, T> TryFromCtx<'a, (usize, StrCtx), T> for &'a str where T: AsRef<[u8]> {
    type Error = error::Error;
    #[inline]
    fn try_from_ctx(src: &'a T, ctx: (usize, StrCtx)) -> error::Result<Self> {
        let src = src.as_ref();
        TryFromCtx::try_from_ctx(src, ctx)
    }
}

impl<'a> TryIntoCtx<(usize, DefaultCtx)> for &'a [u8] {
    type Error = error::Error;
    #[inline]
    fn try_into_ctx(self, dst: &mut [u8], (offset, _): (usize, DefaultCtx)) -> error::Result<()> {
        let size = self.len();
        if offset + size > dst.len () {
            return Err(error::Error::BadOffset(format!("requested: {:?}, dst len: {}", (offset..offset+size), dst.len())).into())
        }
        unsafe { copy_nonoverlapping(self.as_ptr(), dst.as_mut_ptr(), size) };
        Ok(())
    }
}

impl<'a> TryIntoCtx<(usize, StrCtx)> for &'a str {
    type Error = error::Error;
    #[inline]
    fn try_into_ctx(self, dst: &mut [u8], (offset, _): (usize, StrCtx)) -> error::Result<()> {
        let bytes = self.as_bytes();
        TryIntoCtx::try_into_ctx(bytes, dst, (offset, CTX))
    }
}
