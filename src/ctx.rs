//! Generic context-aware conversion traits, for automatic _downstream_ extension of `Pread`, et. al
//!
//! # Discussion
//! **TODO**: UPDATE THIS
//!
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
//! **TODO**: THIS IS NOT TRUE ANYMORE
//!
//! Suppose we have a datatype and we want to specify how to parse or serialize this datatype out of some arbitrary
//! byte buffer. In order to do this, we need to provide a `TryFromCtx` impl for our datatype. In particular, if we
//! do this for the `[u8]` target, using the convention `(usize, YourCtx)`, you will automatically get access to
//! calling `pread_with::<YourDatatype>` on arrays of bytes.
//!
//! ```rust
//! use scroll::{self, ctx, Endian, Pread, BE};
//!
//! struct Data<'a> {
//!   name: &'a str,
//!   id: u32,
//! }
//!
//! impl<'a> ctx::TryFromCtx<'a, Endian> for Data<'a> {
//!   type Error = scroll::Error;
//!   fn try_from_ctx (src: &'a [u8], ctx: Endian)
//!     -> Result<Self, Self::Error> {
//!     let name = src.pread::<&str>(0)?;
//!     let id = src.pread_with(name.len() + 1, ctx)?;
//!     Ok(Data { name: name, id: id })
//!   }
//! }
//!
//! let bytes = b"UserName\x00\x01\x02\x03\x04";
//! let data = bytes.pread_with::<Data>(0, BE).unwrap();
//! assert_eq!(data.id, 0x01020304);
//! assert_eq!(data.name.to_string(), "UserName".to_string());
//!
//! ```

use core::ptr::copy_nonoverlapping;
use core::mem::transmute;
use core::mem::size_of;
use core::str;
use core::result;

use error;
use endian;

pub trait MeasureWith<Ctx> {
    type Units;
    #[inline]
    fn measure_with(&self, ctx: &Ctx) -> Self::Units;
}

impl<Ctx> MeasureWith<Ctx> for [u8] {
    type Units = usize;
    #[inline]
    fn measure_with(&self, _ctx: &Ctx) -> Self::Units {
        self.len()
    }
}

impl<Ctx, T: AsRef<[u8]>> MeasureWith<Ctx> for T {
    type Units = usize;
    #[inline]
    fn measure_with(&self, _ctx: &Ctx) -> Self::Units {
        self.as_ref().len()
    }
}

/// The default parsing context; use this when the context isn't important for your datatype
pub type DefaultCtx = endian::Endian;

/// Convenience constant for the default parsing context
pub const CTX: DefaultCtx = endian::NATIVE;

/// The parsing context for converting a byte sequence to a `&str`
///
/// `StrCtx` specifies what byte delimiter to use, and defaults to C-style null terminators. Be careful.
#[derive(Debug, Copy, Clone)]
pub enum StrCtx {
    Delimiter(u8),
    DelimiterUntil(u8, usize),
    Length(usize),
}

/// A C-style, null terminator based delimiter for a `StrCtx`
pub const NULL: StrCtx = StrCtx::Delimiter(0);
/// A space-based delimiter for a `StrCtx`
pub const SPACE: StrCtx = StrCtx::Delimiter(0x20);
/// A newline-based delimiter for a `StrCtx`
pub const RET: StrCtx = StrCtx::Delimiter(0x0a);
/// A tab-based delimiter for a `StrCtx`
pub const TAB: StrCtx = StrCtx::Delimiter(0x09);

impl Default for StrCtx {
    #[inline]
    fn default() -> Self {
        NULL
    }
}

/// Reads `Self` from `This` using the context `Ctx`; must _not_ fail
pub trait FromCtx<Ctx: Copy = DefaultCtx, This: ?Sized = [u8]> {
    #[inline]
    fn from_ctx(this: &This, ctx: Ctx) -> Self;
}

/// Tries to read `Self` from `This` using the context `Ctx`
pub trait TryFromCtx<'a, Ctx: Copy = DefaultCtx, This: ?Sized = [u8]> where Self: 'a + Sized {
    type Error;
    #[inline]
    fn try_from_ctx(from: &'a This, ctx: Ctx) -> Result<Self, Self::Error>;
}

/// Writes `Self` into `This` using the context `Ctx`
pub trait IntoCtx<Ctx: Copy = DefaultCtx, This: ?Sized = [u8]>: Sized {
    fn into_ctx(self, &mut This, ctx: Ctx);
}

/// Tries to write `Self` into `This` using the context `Ctx`
pub trait TryIntoCtx<Ctx: Copy = DefaultCtx, This: ?Sized = [u8]>: Sized {
    type Error;
    fn try_into_ctx(self, &mut This, ctx: Ctx) -> Result<(), Self::Error>;
}

/// Gets the size of `Self` with a `Ctx`, and in `Self::Units`. Implementors can then call `Gread` related functions
///
/// The rationale behind this trait is to:
///
/// 1. Prevent `gread` from being used, and the offset being modified based on simply the sizeof the value, which can be a misnomer, e.g., for Leb128, etc.
/// 2. Allow a context based size, which is useful for 32/64 bit variants for various containers, etc.
pub trait SizeWith<Ctx = DefaultCtx> {
    type Units;
    #[inline]
    fn size_with(ctx: &Ctx) -> Self::Units;
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
            assert!($dst.len() >= $size);
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
                assert!(dst.len() >= $size);
                write_into!($typ, $size, self, dst, le);
            }
        }
        impl TryIntoCtx<$ctx> for $typ where $typ: IntoCtx<$ctx> {
            type Error = error::Error;
            #[inline]
            fn try_into_ctx(self, dst: &mut [u8], le: super::Endian) -> error::Result<()> {
                if $size > dst.len () {
                    Err(error::Error::TooBig{size: $size, len: dst.len()})
                } else {
                    Ok(<$typ as IntoCtx<$ctx>>::into_ctx(self, dst, le))
                }
            }
        }
    }
}

macro_rules! from_ctx_impl {
    ($typ:tt, $size:expr, $ctx:ty) => {
        impl<'a> FromCtx<$ctx> for $typ {
            #[inline]
            fn from_ctx(src: &[u8], le: $ctx) -> Self {
                assert!(src.len() >= $size);
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

        impl<'a> TryFromCtx<'a, $ctx> for $typ where $typ: FromCtx<$ctx> {
            type Error = error::Error<usize>;
            #[inline]
            fn try_from_ctx(src: &'a [u8], le: $ctx) -> result::Result<Self, Self::Error> {
                if $size > src.len () {
                    Err(error::Error::TooBig{size: $size, len: src.len()})
                } else {
                    Ok(FromCtx::from_ctx(&src, le))
                }
            }
        }
        // as ref
        impl<'a, T> FromCtx<$ctx, T> for $typ where T: AsRef<[u8]> {
            #[inline]
            fn from_ctx(src: &T, le: $ctx) -> Self {
                let src = src.as_ref();
                assert!(src.len() >= $size);
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

        impl<'a, T> TryFromCtx<'a, $ctx, T> for $typ where $typ: FromCtx<$ctx, T>, T: AsRef<[u8]> {
            type Error = error::Error;
            #[inline]
            fn try_from_ctx(src: &'a T, le: $ctx) -> result::Result<Self, Self::Error> {
                let src = src.as_ref();
                Self::try_from_ctx(src, le)
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
        impl<'a> FromCtx<$ctx> for $typ {
            #[inline]
            fn from_ctx(src: &[u8], le: $ctx) -> Self {
                assert!(src.len() >= ::core::mem::size_of::<Self>());
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

        impl<'a> TryFromCtx<'a, $ctx> for $typ where $typ: FromCtx<$ctx> {
            type Error = error::Error;
            #[inline]
            fn try_from_ctx(src: &'a [u8], le: $ctx) -> result::Result<Self, Self::Error> {
                if $size > src.len () {
                    Err(error::Error::TooBig{size: $size, len: src.len()})
                } else {
                    Ok(FromCtx::from_ctx(src, le))
                }
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
                assert!(dst.len() >= $size);
                write_into!(signed_to_unsigned!($typ), $size, transmute::<$typ, signed_to_unsigned!($typ)>(self), dst, le);
            }
        }
        impl TryIntoCtx<$ctx> for $typ where $typ: IntoCtx<$ctx> {
            type Error = error::Error;
            #[inline]
            fn try_into_ctx(self, dst: &mut [u8], le: super::Endian) -> error::Result<()> {
                if $size > dst.len () {
                    Err(error::Error::TooBig{size: $size, len: dst.len()})
                } else {
                    Ok(<$typ as IntoCtx<$ctx>>::into_ctx(self, dst, le))
                }
            }
        }
    }
}

into_ctx_float_impl!(f32, 4, super::Endian);
into_ctx_float_impl!(f64, 8, super::Endian);

impl<'a> TryFromCtx<'a, StrCtx> for &'a str {
    type Error = error::Error;
    #[inline]
    /// Read a `&str` from `src` using `delimiter`
    fn try_from_ctx(src: &'a [u8], ctx: StrCtx) -> Result<Self, Self::Error> {
        let len = match ctx {
            StrCtx::Length(len) => len,
            StrCtx::Delimiter(delimiter) => src.iter().take_while(|c| **c != delimiter).count(),
            StrCtx::DelimiterUntil(delimiter, len) => {
                if len > src.len() {
                    return Err(error::Error::TooBig{size: len, len: src.len()});
                };
                src
                    .iter()
                    .take_while(|c| **c != delimiter)
                    .take(len)
                    .count()
            }
        };

        if len > src.len() {
            return Err(error::Error::TooBig{size: len, len: src.len()});
        };

        str::from_utf8(&src[..len])
            .map_err(|_| error::Error::Custom("utf8 error".to_owned()))
    }
}

impl<'a, T> TryFromCtx<'a, StrCtx, T> for &'a str where T: AsRef<[u8]> {
    type Error = error::Error;
    #[inline]
    fn try_from_ctx(src: &'a T, ctx: StrCtx) -> result::Result<Self, Self::Error> {
        let src = src.as_ref();
        TryFromCtx::try_from_ctx(src, ctx)
    }
}

impl<'a> TryIntoCtx<DefaultCtx> for &'a [u8] {
    type Error = error::Error;
    #[inline]
    fn try_into_ctx(self, dst: &mut [u8], _ctx: DefaultCtx) -> error::Result<()> {
        let src_len = self.len() as isize;
        let dst_len = dst.len() as isize;
        // if src_len < 0 || dst_len < 0 || offset < 0 {
        //     return Err(error::Error::BadOffset(format!("requested operation has negative casts: src len: {} dst len: {} offset: {}", src_len, dst_len, offset)).into())
        // }
        if src_len > dst_len {
            Err(error::Error::TooBig{ size: self.len(), len: dst.len()})
        } else {
            unsafe { copy_nonoverlapping(self.as_ptr(), dst.as_mut_ptr(), src_len as usize) };
            Ok(())
        }
    }
}

impl<'a> TryIntoCtx<DefaultCtx> for &'a str {
    type Error = error::Error;
    #[inline]
    fn try_into_ctx(self, dst: &mut [u8], ctx: DefaultCtx) -> error::Result<()> {
        let bytes = self.as_bytes();
        TryIntoCtx::try_into_ctx(bytes, dst, ctx)
    }
}

// TODO: we can make this compile time without size_of call, but compiler probably does that anyway
macro_rules! sizeof_impl {
    ($ty:ty) => {
        impl SizeWith for $ty {
            type Units = usize;
            #[inline]
            fn size_with(_ctx: &DefaultCtx) -> usize {
                size_of::<$ty>()
            }
        }
    }
}

sizeof_impl!(u8);
sizeof_impl!(i8);
sizeof_impl!(u16);
sizeof_impl!(i16);
sizeof_impl!(u32);
sizeof_impl!(i32);
sizeof_impl!(u64);
sizeof_impl!(i64);
sizeof_impl!(f32);
sizeof_impl!(f64);
sizeof_impl!(usize);
sizeof_impl!(isize);

impl FromCtx for usize {
    #[inline]
    fn from_ctx(src: &[u8], le: super::Endian) -> Self {
        let size = ::core::mem::size_of::<Self>();
        assert!(src.len() >= size);
        let mut data: usize = 0;
        unsafe {
            copy_nonoverlapping(
                src.as_ptr(),
                &mut data as *mut usize as *mut u8,
                size);
            transmute((if le.is_little() { data.to_le() } else { data.to_be() }))
        }
    }
}

impl<'a> TryFromCtx<'a, super::Endian> for usize where usize: FromCtx<super::Endian> {
    type Error = error::Error;
    #[inline]
    fn try_from_ctx(src: &'a [u8], le: super::Endian) -> error::Result<Self> {
        let size = ::core::mem::size_of::<usize>();
        if size > src.len () {
            Err(error::Error::TooBig{size: size, len: src.len()})
        } else {
            Ok(FromCtx::from_ctx(src, le))
        }
    }
}

impl IntoCtx for usize {
    #[inline]
    fn into_ctx(self, dst: &mut [u8], le: super::Endian) {
        let size = ::core::mem::size_of::<Self>();
        assert!(dst.len() >= size);
        let mut data = if le.is_little() { self.to_le() } else { self.to_be() };
        let data = &mut data as *mut usize as *mut u8;
        unsafe {
            copy_nonoverlapping(data, dst.as_mut_ptr(), size);
        }
    }
}

impl TryIntoCtx<super::Endian> for usize where usize: IntoCtx<super::Endian> {
    type Error = error::Error;
    #[inline]
    fn try_into_ctx(self, dst: &mut [u8], le: super::Endian) -> error::Result<()> {
        let size = ::core::mem::size_of::<usize>();
        if size > dst.len() {
            Err(error::Error::TooBig{size: size, len: dst.len()})
        } else {
            Ok(<usize as IntoCtx<super::Endian>>::into_ctx(self, dst, le))
        }
    }
}

// example of marshalling to bytes, let's wait until const is an option
// impl FromCtx for [u8; 10] {
//     fn from_ctx(bytes: &[u8], _ctx: super::Endian) -> Self {
//         let mut dst: Self = [0; 10];
//         assert!(bytes.len() >= dst.len());
//         unsafe {
//             copy_nonoverlapping(bytes.as_ptr(), dst.as_mut_ptr(), dst.len());
//         }
//         dst
//     }
// }
