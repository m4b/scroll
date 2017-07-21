use core::result;
use core::ops::{Index, RangeFrom, Add, AddAssign};

use ctx::{FromCtx, TryFromCtx, SizeWith, MeasureWith};
use error;

/// A very generic, contextual pread interface in Rust. Allows completely parallelized reads, as `Self` is immutable
///
/// Don't be scared! The `Pread` definition _is_ terrifying, but it is definitely tractable. Essentially, `E` is the error, `Ctx` the parsing context, `I` is the indexing type, `TryCtx` is the "offset + ctx" Context given to the `TryFromCtx` trait bounds, and `SliceCtx` is the "offset + size + ctx" context given to the `TryRefFromCtx` trait bound.
///
/// # Implementing Your Own Reader
/// If you want to implement your own reader for a type `Foo` from some kind of buffer (say `[u8]`), then you need to implement [TryFromCtx](trait.TryFromCtx.html)
///
/// ```rust
/// use scroll::{self, ctx, Pread};
/// #[derive(Debug, PartialEq, Eq)]
/// pub struct Foo(u16);
///
/// impl<'a> ctx::TryFromCtx<'a, scroll::Endian> for Foo {
///      type Error = scroll::Error;
///      type Size = usize;
///      fn try_from_ctx(this: &'a [u8], le: scroll::Endian) -> Result<(Self, Self::Size), Self::Error> {
///          if this.len() < 2 { return Err((scroll::Error::Custom("whatever".to_string())).into()) }
///          let n = this.pread_with(0, le)?;
///          Ok((Foo(n), 2))
///      }
/// }
///
/// let bytes: [u8; 4] = [0xde, 0xad, 0, 0];
/// let foo = bytes.pread::<Foo>(0).unwrap();
/// assert_eq!(Foo(0xadde), foo);
/// let foo2 = bytes.pread_with::<Foo>(0, scroll::BE).unwrap();
/// assert_eq!(Foo(0xdeadu16), foo2);
/// ```
///
/// # Advanced: Using Your Own Error in `TryFromCtx`
/// ```rust
///  use scroll::{self, ctx, Pread};
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
///  impl<'a> ctx::TryFromCtx<'a, scroll::Endian> for Foo {
///      type Error = ExternalError;
///      type Size = usize;
///      fn try_from_ctx(this: &'a [u8], le: scroll::Endian) -> Result<(Self, Self::Size), Self::Error> {
///          if this.len() <= 2 { return Err((ExternalError {}).into()) }
///          let offset = &mut 0;
///          let n = this.gread_with(offset, le)?;
///          Ok((Foo(n), *offset))
///      }
///  }
///
/// let bytes: [u8; 4] = [0xde, 0xad, 0, 0];
/// let foo: Result<Foo, ExternalError> = bytes.pread(0);
/// ```
//pub trait Pread<Ctx, E, I = usize> : Index<I> + Index<RangeFrom<I>> + MeasureWith<Ctx, Units = I>
pub trait Pread<Ctx, E, I = usize>
 where
       Ctx: Copy,
       I: Add + Copy + PartialOrd,
       E: From<error::Error<I>>,
{
    #[inline]
    /// Reads a value from `self` at `offset` with a default `Ctx`. For the primitive numeric values, this will read at the machine's endianness.
    /// # Example
    /// ```rust
    /// use scroll::Pread;
    /// let bytes = [0x7fu8; 0x01];
    /// let byte = bytes.pread::<u8>(0).unwrap();
    fn pread<'a, N: TryFromCtx<'a, Ctx, <Self as Index<RangeFrom<I>>>::Output, Error = E, Size = I>>(&'a self, offset: I) -> result::Result<N, E>
        where
        <Self as Index<RangeFrom<I>>>::Output: 'a, Ctx: Default,
        Self: Index<I> + Index<RangeFrom<I>> + MeasureWith<Ctx, Units = I>,
    {
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
    fn pread_with<'a, N: TryFromCtx<'a, Ctx, <Self as Index<RangeFrom<I>>>::Output, Error = E, Size = I>>(&'a self, offset: I, ctx: Ctx) -> result::Result<N, E>
        where
        <Self as Index<RangeFrom<I>>>::Output: 'a,
        Self: Index<I> + Index<RangeFrom<I>> + MeasureWith<Ctx, Units = I>
    {
        let len = self.measure_with(&ctx);
        if offset >= len {
            return Err(error::Error::BadOffset(offset).into())
        }
        N::try_from_ctx(&self[offset..], ctx).and_then(|(n, _)| Ok(n))
    }

    /// Reads the type `N` from `Self`, with a default parsing context.
    /// For the primitive numeric types, this will be at the host machine's endianness.
    ///
    /// # Example
    /// ```rust
    /// use scroll::Lread;
    /// use std::io::Cursor;
    /// let bytes = [0xef, 0xbe];
    /// let mut bytes = Cursor::new(&bytes[..]);
    /// let beef = bytes.lread::<u16>().unwrap();
    /// assert_eq!(0xbeef, beef);
    /// ```
    #[inline]
    #[cfg(feature = "std")]
    fn lread<N: FromCtx<Ctx> + SizeWith<Ctx, Units = usize>>(&mut self) -> error::Result<N>
        where
        Ctx: Default,
        Self: ::std::io::Read,
    {
        let ctx = Ctx::default();
        self.lread_with(ctx)
    }

    /// Reads the type `N` from `Self`, with the parsing context `ctx`.
    /// **NB**: this will panic if the type you're reading has a size greater than 256. Plans are to have this allocate in larger cases.
    ///
    /// For the primitive numeric types, this will be at the host machine's endianness.
    ///
    /// # Example
    /// ```rust
    /// use scroll::{Lread, LE, BE};
    /// use std::io::Cursor;
    /// let bytes = [0xef, 0xbe, 0xb0, 0xb0, 0xfe, 0xed, 0xde, 0xad];
    /// let mut bytes = Cursor::new(&bytes[..]);
    /// let beef = bytes.lread_with::<u16>(LE).unwrap();
    /// assert_eq!(0xbeef, beef);
    /// let b0 = bytes.lread::<u8>().unwrap();
    /// assert_eq!(0xb0, b0);
    /// let b0 = bytes.lread::<u8>().unwrap();
    /// assert_eq!(0xb0, b0);
    /// let feeddead = bytes.lread_with::<u32>(BE).unwrap();
    /// assert_eq!(0xfeeddead, feeddead);
    /// ```
    #[inline]
    #[cfg(feature = "std")]
    fn lread_with<N: FromCtx<Ctx> + SizeWith<Ctx, Units = usize>>(&mut self, ctx: Ctx) -> error::Result<N> where Self: ::std::io::Read {
        let mut scratch = [0u8; 256];
        let size = N::size_with(&ctx);
        let mut buf = &mut scratch[0..size];
        self.read_exact(&mut buf)?;
        Ok(N::from_ctx(buf, ctx))
    }

    #[inline]
    /// Reads a value from `self` at `offset` with a default `Ctx`. For the primitive numeric values, this will read at the machine's endianness. Updates the offset
    /// # Example
    /// ```rust
    /// use scroll::Pread;
    /// let offset = &mut 0;
    /// let bytes = [0x7fu8; 0x01];
    /// let byte = bytes.gread::<u8>(offset).unwrap();
    /// assert_eq!(*offset, 1);
    fn gread<'a, N: TryFromCtx<'a, Ctx, <Self as Index<RangeFrom<I>>>::Output, Error = E, Size = I>>(&'a self, offset: &mut I) -> result::Result<N, E>
        where
        I: AddAssign,
        Ctx: Default,
        Self: Index<I> + Index<RangeFrom<I>> + MeasureWith<Ctx, Units = I>,
    <Self as Index<RangeFrom<I>>>::Output: 'a
    {
        let ctx = Ctx::default();
        self.gread_with(offset, ctx)
    }
    /// Reads a value from `self` at `offset` with the given `ctx`, and updates the offset.
    /// # Example
    /// ```rust
    /// use scroll::Pread;
    /// let offset = &mut 0;
    /// let bytes: [u8; 2] = [0xde, 0xad];
    /// let dead: u16 = bytes.gread_with(offset, scroll::BE).unwrap();
    /// assert_eq!(dead, 0xdeadu16);
    /// assert_eq!(*offset, 2);
    #[inline]
    fn gread_with<'a, N: TryFromCtx<'a, Ctx, <Self as Index<RangeFrom<I>>>::Output, Error = E, Size = I>>
        (&'a self, offset: &mut I, ctx: Ctx) ->
        result::Result<N, E>
        where
        I: AddAssign,
        Self: Index<I> + Index<RangeFrom<I>> + MeasureWith<Ctx, Units = I>,
    <Self as Index<RangeFrom<I>>>::Output: 'a,
    {
        let o = *offset;
        // self.pread_with(o, ctx).and_then(|(n, size)| {
        //     *offset += size;
        //     Ok(n)
        // })
        let len = self.measure_with(&ctx);
        if o >= len {
            return Err(error::Error::BadOffset(o).into())
        }
        N::try_from_ctx(&self[o..], ctx).and_then(|(n, size)| {
            *offset += size;
            Ok(n)
        })
    }

    /// Trys to write `inout.len()` `N`s into `inout` from `Self` starting at `offset`, using the default context for `N`, and updates the offset.
    /// # Example
    /// ```rust
    /// use scroll::Pread;
    /// let mut bytes: Vec<u8> = vec![0, 0];
    /// let offset = &mut 0;
    /// let bytes_from: [u8; 2] = [0x48, 0x49];
    /// bytes_from.gread_inout(offset, &mut bytes).unwrap();
    /// assert_eq!(&bytes, &bytes_from);
    /// assert_eq!(*offset, 2);
    #[inline]
    fn gread_inout<'a, N>(&'a self, offset: &mut I, inout: &mut [N]) -> result::Result<(), E>
        where
        I: AddAssign,
        N: TryFromCtx<'a, Ctx, <Self as Index<RangeFrom<I>>>::Output, Error = E, Size = I>,
        Ctx: Default,
        Self: Index<I> + Index<RangeFrom<I>> + MeasureWith<Ctx, Units = I>,
    <Self as Index<RangeFrom<I>>>::Output: 'a
    {
        let len = inout.len();
        for i in 0..len {
            inout[i] = self.gread(offset)?;
        }
        Ok(())
    }

    /// Trys to write `inout.len()` `N`s into `inout` from `Self` starting at `offset`, using the context `ctx`
    /// # Example
    /// ```rust
    /// use scroll::{ctx, LE, Pread};
    /// let mut bytes: Vec<u8> = vec![0, 0];
    /// let offset = &mut 0;
    /// let bytes_from: [u8; 2] = [0x48, 0x49];
    /// bytes_from.gread_inout_with(offset, &mut bytes, LE).unwrap();
    /// assert_eq!(&bytes, &bytes_from);
    /// assert_eq!(*offset, 2);
    #[inline]
    fn gread_inout_with<'a, N>(&'a self, offset: &mut I, inout: &mut [N], ctx: Ctx) -> result::Result<(), E>
        where
        I: AddAssign,
        N: TryFromCtx<'a, Ctx, <Self as Index<RangeFrom<I>>>::Output, Error = E, Size = I>,
        Self: Index<I> + Index<RangeFrom<I>> + MeasureWith<Ctx, Units = I>,
    <Self as Index<RangeFrom<I>>>::Output: 'a
    {
        let len = inout.len();
        for i in 0..len {
            inout[i] = self.gread_with(offset, ctx)?;
        }
        Ok(())
    }
}

impl<Ctx: Copy,
     I: Add + Copy + PartialOrd,
     E: From<error::Error<I>>,
     R: ?Sized>
    Pread<Ctx, E, I> for R {}
