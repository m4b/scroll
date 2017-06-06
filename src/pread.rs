use core::result;
use core::fmt::Debug;

use ctx::{TryFromCtx, TryRefFromCtx};
use error;
use endian::Endian;

/// A very generic, contextual pread interface in Rust. Allows completely parallelized reads, as `Self` is immutable
///
/// Don't be scared! The `Pread` definition _is_ terrifying, but it is definitely tractable. Essentially, `E` is the error, `Ctx` the parsing context, `I` is the indexing type, `TryCtx` is the "offset + ctx" Context given to the `TryFromCtx` trait bounds, and `SliceCtx` is the "offset + size + ctx" context given to the `TryRefFromCtx` trait bound.
///
/// # Implementing Your Own Reader
/// If you want to implement your own reader for a type `Foo` from some kind of buffer (say `[u8]`), then you need to implement [TryFromCtx](trait.TryFromCtx.html)
///
/// ```rust
/// use scroll::{self, ctx};
///  #[derive(Debug, PartialEq, Eq)]
///  pub struct Foo(u16);
///
///  impl<'a> ctx::TryFromCtx<'a> for Foo {
///      type Error = scroll::Error;
///      fn try_from_ctx(this: &'a [u8], ctx: (usize, scroll::Endian)) -> Result<Self, Self::Error> {
///          use scroll::Pread;
///          let offset = ctx.0;
///          let le = ctx.1;
///          if offset > 2 { return Err((scroll::Error::Custom("whatever".to_string())).into()) }
///          let n = this.pread_with(offset, le)?;
///          Ok(Foo(n))
///      }
///  }
///
/// use scroll::Pread;
/// // you can now read `Foo`'s out of a &[u8] buffer for free, with `Pread` and `Gread` without doing _anything_ else!
/// let bytes: [u8; 4] = [0xde, 0xad, 0, 0];
/// let foo = bytes.pread::<Foo>(0).unwrap();
/// assert_eq!(Foo(0xadde), foo);
/// let foo2 = bytes.pread_with::<Foo>(0, scroll::BE).unwrap();
/// assert_eq!(Foo(0xdeadu16), foo2);
/// ```
///
/// # Advanced: Using Your Own Error in `TryFromCtx`
/// ```rust
///  use scroll::{self, ctx};
///  use std::error;
///  use std::fmt::{self, Display};
///  // make some kind of normal error which also can transform a scroll error ideally (quick_error, error_chain allow this automatically nowadays)
///  #[derive(Debug)]
///  pub struct ExternalError {}
///
///  impl Display for ExternalError {
///      fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
///          write!(fmt, "ExternalError")
///      }
///  }
///
///  impl error::Error for ExternalError {
///      fn description(&self) -> &str {
///          "ExternalError"
///      }
///      fn cause(&self) -> Option<&error::Error> { None}
///  }
///
///  impl From<scroll::Error> for ExternalError {
///      fn from(err: scroll::Error) -> Self {
///          match err {
///              _ => ExternalError{},
///          }
///      }
///  }
///  #[derive(Debug, PartialEq, Eq)]
///  pub struct Foo(u16);
///
///  impl<'a> ctx::TryFromCtx<'a> for Foo {
///      type Error = ExternalError;
///      fn try_from_ctx(this: &'a [u8], ctx: (usize, scroll::Endian)) -> Result<Self, Self::Error> {
///          use scroll::Pread;
///          let offset = ctx.0;
///          let le = ctx.1;
///          if offset > 2 { return Err((ExternalError {}).into()) }
///          let n = this.pread_with(offset, le)?;
///          Ok(Foo(n))
///      }
///  }
///
/// use scroll::Pread;
/// let bytes: [u8; 4] = [0xde, 0xad, 0, 0];
/// let foo: Result<Foo, ExternalError> = bytes.pread(0);
/// ```
pub trait Pread<Ctx = Endian, E = error::Error, I = usize, TryCtx = (I, Ctx), SliceCtx = (I, I, Ctx) >
 where E: Debug,
       Ctx: Copy + Default + Debug,
       I: Copy + Debug,
       TryCtx: Copy + Default + Debug,
       SliceCtx: Copy + Default + Debug,
{
    #[inline]
    /// Reads a value at `offset` with `ctx` - or those times when you _know_ your deserialization can't fail.
    fn pread_unsafe<'a, N: TryFromCtx<'a, TryCtx, Error = E>>(&'a self, offset: I, ctx: Ctx) -> N {
        self.pread_with(offset, ctx).unwrap()
    }
    #[inline]
    /// Reads a value from `self` at `offset` with a default `Ctx`. For the primitive numeric values, this will read at the machine's endianness.
    /// # Example
    /// ```rust
    /// use scroll::Pread;
    /// let bytes = [0x7fu8; 0x01];
    /// let byte = bytes.pread::<u8>(0).unwrap();
    fn pread<'a, N: TryFromCtx<'a, TryCtx, Error = E>>(&'a self, offset: I) -> result::Result<N, E> {
        self.pread_with(offset, Ctx::default())
    }
    #[inline]
    /// Reads a value from `self` at `offset` with the given `ctx`
    /// # Example
    /// ```rust
    /// use scroll::Pread;
    /// let bytes: [u8; 2] = [0xde, 0xad];
    /// let dead: u16 = bytes.pread_with(0, scroll::BE).unwrap();
    /// assert_eq!(dead, 0xdeadu16);
    fn pread_with<'a, N: TryFromCtx<'a, TryCtx, Error = E>>(&'a self, offset: I, ctx: Ctx) -> result::Result<N, E>;
    /// Slices an `N` from `self` at `offset` up to `count` times
    #[inline]
    /// # Example
    /// ```rust
    /// use scroll::Pread;
    /// let bytes: [u8; 2] = [0x48, 0x49];
    /// let hi: &str = bytes.pread_slice(0, 2).unwrap();
    /// assert_eq!(hi, "HI");
    /// let bytes2 = bytes.pread_slice::<[u8]>(0, 2).unwrap();
    /// assert_eq!(bytes, bytes2);
    fn pread_slice<'a, N: ?Sized + TryRefFromCtx<SliceCtx, Error = E>>(&'a self, offset: I, count: I) -> result::Result<&'a N, E>;
}

impl<Ctx, E> Pread<Ctx, E> for [u8]
    where
    E: Debug,
    Ctx: Debug + Copy + Default {
    #[inline]
    fn pread_unsafe<'a, N: TryFromCtx<'a, (usize, Ctx), Error = E>>(&'a self, offset: usize, le: Ctx) -> N {
        TryFromCtx::try_from_ctx(self, (offset, le)).unwrap()
    }
    #[inline]
    fn pread_with<'a, N: TryFromCtx<'a, (usize, Ctx), Error = E>>(&'a self, offset: usize, le: Ctx) -> result::Result<N, E> {
        TryFromCtx::try_from_ctx(self, (offset, le))
    }
    #[inline]
    fn pread_slice<N: ?Sized + TryRefFromCtx<(usize, usize, Ctx), Error = E>>(&self, offset: usize, count: usize) -> result::Result<&N, E> {
        TryRefFromCtx::try_ref_from_ctx(self, (offset, count, Ctx::default()))
    }
}

impl<Ctx, E, T> Pread<Ctx, E> for T
    where
    E: Debug,
    Ctx: Debug + Copy + Default,
    T: AsRef<[u8]> {
    #[inline]
    fn pread_unsafe<'a, N: TryFromCtx<'a, (usize, Ctx), Error = E>>(&'a self, offset: usize, le: Ctx) -> N {
        <[u8] as Pread<Ctx, E>>::pread_unsafe::<N>(self.as_ref(), offset, le)
    }
    #[inline]
    fn pread_with<'a, N: TryFromCtx<'a, (usize, Ctx), Error = E>>(&'a self, offset: usize, le: Ctx) -> result::Result<N, E> {
        TryFromCtx::try_from_ctx(self.as_ref(), (offset, le))
    }
    #[inline]
    fn pread_slice<N: ?Sized + TryRefFromCtx<(usize, usize, Ctx), Error = E>>(&self, offset: usize, count: usize) -> result::Result<&N, E> {
        <[u8] as Pread<Ctx, E>>::pread_slice::<N>(self.as_ref(), offset, count)
    }
}
