use core::result;
use core::ops::{Index, IndexMut, RangeFrom, Add, AddAssign};

use ctx::{IntoCtx, TryIntoCtx, MeasureWith, SizeWith};
use error;

/// Writes into `Self` at an offset of type `I` using a `Ctx`
///
/// To implement writing into an arbitrary byte buffer, implement `TryIntoCtx`
/// # Example
/// ```rust
/// use scroll::{self, ctx, LE, Endian, Pwrite};
/// #[derive(Debug, PartialEq, Eq)]
/// pub struct Foo(u16);
///
/// // this will use the default `DefaultCtx = scroll::Endian` and `I = usize`...
/// impl ctx::TryIntoCtx<Endian> for Foo {
///     // you can use your own error here too, but you will then need to specify it in fn generic parameters
///     type Error = scroll::Error;
///     type Size = usize;
///     // you can write using your own context too... see `leb128.rs`
///     fn try_into_ctx(self, this: &mut [u8], le: Endian) -> Result<Self::Size, Self::Error> {
///         if this.len() < 2 { return Err((scroll::Error::Custom("whatever".to_string())).into()) }
///         this.pwrite_with(self.0, 0, le)?;
///         Ok(2)
///     }
/// }
/// // now we can write a `Foo` into some buffer (in this case, a byte buffer, because that's what we implemented it for above)
///
/// let mut bytes: [u8; 4] = [0, 0, 0, 0];
/// bytes.pwrite_with(Foo(0x7f), 1, LE).unwrap();
///
pub trait Pwrite<Ctx, E, I = usize> : Index<I> + IndexMut<RangeFrom<I>> + MeasureWith<Ctx, Units = I>
 where
       Ctx: Copy,
       I: Add + Copy + PartialOrd,
       E: From<error::Error<I>>,
{
    fn pwrite<N: TryIntoCtx<Ctx, <Self as Index<RangeFrom<I>>>::Output, Error = E, Size = I>>(&mut self, n: N, offset: I) -> result::Result<I, E> where Ctx: Default {
        self.pwrite_with(n, offset, Ctx::default())
    }
    /// Write `N` at offset `I` with context `Ctx`
    /// # Example
    /// ```
    /// use scroll::{Pwrite, Pread, LE};
    /// let mut bytes: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
    /// bytes.pwrite_with::<u32>(0xbeefbeef, 0, LE).unwrap();
    /// assert_eq!(bytes.pread_with::<u32>(0, LE).unwrap(), 0xbeefbeef);
    fn pwrite_with<N: TryIntoCtx<Ctx, <Self as Index<RangeFrom<I>>>::Output, Error = E, Size = I>>(&mut self, n: N, offset: I, ctx: Ctx) -> result::Result<I, E> {
        let len = self.measure_with(&ctx);
        if offset >= len {
            return Err(error::Error::BadOffset(offset).into())
        }
        let dst = &mut self[offset..];
        n.try_into_ctx(dst, ctx)
    }
    /// Writes the type `N` into `Self`, with the parsing context `ctx`.
    /// **NB**: this will panic if the type you're writing has a size greater than 256. Plans are to have this allocate in larger cases.
    ///
    /// For the primitive numeric types, this will be at the host machine's endianness.
    ///
    /// # Example
    /// ```rust
    /// use scroll::Lwrite;
    /// use std::io::Cursor;
    ///
    /// let mut bytes = [0x0u8; 4];
    /// let mut bytes = Cursor::new(&mut bytes[..]);
    /// bytes.lwrite(0xdeadbeef as u32).unwrap();
    /// assert_eq!(bytes.into_inner(), [0xef, 0xbe, 0xad, 0xde,]);
    /// ```
    #[inline]
    #[cfg(feature = "std")]
    fn lwrite<N: SizeWith<Ctx, Units = usize> + IntoCtx<Ctx>>(&mut self, n: N) -> error::Result<()>
        where
        Ctx: Default,
        Self: ::std::io::Write
    {
        let ctx = Ctx::default();
        self.lwrite_with(n, ctx)
    }

    /// Writes the type `N` into `Self`, with the parsing context `ctx`.
    /// **NB**: this will panic if the type you're writing has a size greater than 256. Plans are to have this allocate in larger cases.
    ///
    /// For the primitive numeric types, this will be at the host machine's endianness.
    ///
    /// # Example
    /// ```rust
    /// use scroll::{Lwrite, LE, BE};
    /// use std::io::{Write, Cursor};
    ///
    /// let mut bytes = [0x0u8; 10];
    /// let mut cursor = Cursor::new(&mut bytes[..]);
    /// cursor.write_all(b"hello").unwrap();
    /// cursor.lwrite_with(0xdeadbeef as u32, BE).unwrap();
    /// assert_eq!(cursor.into_inner(), [0x68, 0x65, 0x6c, 0x6c, 0x6f, 0xde, 0xad, 0xbe, 0xef, 0x0]);
    /// ```
    #[inline]
    #[cfg(feature = "std")]
    fn lwrite_with<N: SizeWith<Ctx, Units = usize> + IntoCtx<Ctx>>(&mut self, n: N, ctx: Ctx) -> error::Result<()> where Self: ::std::io::Write {
        let mut buf = [0u8; 256];
        let size = N::size_with(&ctx);
        let mut buf = &mut buf[0..size];
        n.into_ctx(buf, ctx);
        self.write_all(buf)?;
        Ok(())
    }
    /// Write `n` into `self` at `offset`, with a default `Ctx`. Updates the offset.
    #[inline]
    fn gwrite<N: TryIntoCtx<Ctx, <Self as Index<RangeFrom<I>>>::Output,  Error = E, Size = I>>(&mut self, n: N, offset: &mut I) -> result::Result<I, E> where
        I: AddAssign,
        Ctx: Default {
        let ctx = Ctx::default();
        self.gwrite_with(n, offset, ctx)
    }
    /// Write `n` into `self` at `offset`, with the `ctx`. Updates the offset.
    #[inline]
    fn gwrite_with<N: TryIntoCtx<Ctx, <Self as Index<RangeFrom<I>>>::Output, Error = E, Size = I>>(&mut self, n: N, offset: &mut I, ctx: Ctx) -> result::Result<I, E>
        where I: AddAssign,
    {
        let o = *offset;
        match self.pwrite_with(n, o, ctx) {
            Ok(size) => {
                *offset += size;
                Ok(size)
            },
            err => err
        }
    }
}

impl<Ctx: Copy,
     I: Add + Copy + PartialOrd,
     E: From<error::Error<I>>,
     R: ?Sized + Index<I> + IndexMut<RangeFrom<I>> + MeasureWith<Ctx, Units = I>>
    Pwrite<Ctx, E, I> for R {}
