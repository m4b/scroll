use core::result;
use core::fmt::Debug;
use core::ops::{Add, AddAssign};
use core::ops::{Index, IndexMut, RangeFrom};

use ctx::{TryIntoCtx, FromCtx, IntoCtx, SizeWith, MeasureWith};
use error;
use pwrite::Pwrite;

/// The Greater Write (`Gwrite`) writes a value into its mutable insides, at a mutable offset
pub trait Gwrite<Ctx, E, I = usize>: Pwrite<Ctx, E, I>
 where Ctx: Copy,
       E: From<error::Error<I>> + Debug,
       I: Add + Copy + PartialOrd + AddAssign + Default + Debug,
{
    /// Write `n` into `self` at `offset`, with a default `Ctx`. Updates the offset.
    #[inline]
    fn gwrite<N: SizeWith<Ctx, Units = I> + TryIntoCtx<Ctx, <Self as Index<RangeFrom<I>>>::Output,  Error = E, Size = I>>(&mut self, n: N, offset: &mut I) -> result::Result<I, E> where Ctx: Default {
        let ctx = Ctx::default();
        self.gwrite_with(n, offset, ctx)
    }
    /// Write `n` into `self` at `offset`, with the `ctx`. Updates the offset.
    #[inline]
    fn gwrite_with<N: SizeWith<Ctx, Units = I> + TryIntoCtx<Ctx, <Self as Index<RangeFrom<I>>>::Output, Error = E, Size = I>>(&mut self, n: N, offset: &mut I, ctx: Ctx) -> result::Result<I, E> {
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
     I: Add + Copy + PartialOrd + AddAssign + Default + Debug,
     E: From<error::Error<I>> + Debug,
     W: ?Sized + Index<I> + IndexMut<RangeFrom<I>> + MeasureWith<Ctx, Units = I>>
    Gwrite<Ctx, E, I> for W {}

/// Core-read - core, no_std friendly trait for reading basic traits from byte buffers. Cannot fail unless the buffer is too small, in which case an assert fires and the program panics.
///
/// If your type implements [FromCtx](trait.FromCtx.html) then you can `cread::<YourType>(offset)`.
///
/// # Example
///
/// ```rust
/// use scroll::{ctx, Cread};
///
/// #[repr(packed)]
/// struct Bar {
///     foo: i32,
///     bar: u32,
/// }
///
/// impl ctx::FromCtx<scroll::Endian> for Bar {
///     fn from_ctx(bytes: &[u8], ctx: scroll::Endian) -> Self {
///         use scroll::Cread;
///         Bar { foo: bytes.cread_with(0, ctx), bar: bytes.cread_with(4, ctx) }
///     }
/// }
///
/// let bytes = [0xff, 0xff, 0xff, 0xff, 0xef,0xbe,0xad,0xde,];
/// let bar = bytes.cread::<Bar>(0);
/// assert_eq!(bar.foo, -1);
/// assert_eq!(bar.bar, 0xdeadbeef);
/// ```
pub trait Cread<Ctx, I = usize> : Index<I> + Index<RangeFrom<I>>
 where
    Ctx: Copy,
{
    /// Reads a value from `Self` at `offset` with `ctx`. Cannot fail.
    /// If the buffer is too small for the value requested, this will panic.
    ///
    /// # Example
    ///
    /// ```rust
    /// use scroll::{Cread, BE, LE};
    /// use std::i64::MAX;
    ///
    /// let bytes = [0x7f,0xff,0xff,0xff,0xff,0xff,0xff,0xff, 0xef,0xbe,0xad,0xde,];
    /// let foo = bytes.cread_with::<i64>(0, BE);
    /// let bar = bytes.cread_with::<u32>(8, LE);
    /// assert_eq!(foo, MAX);
    /// assert_eq!(bar, 0xdeadbeef);
    /// ```
    #[inline]
    fn cread_with<'a, N: FromCtx<Ctx, <Self as Index<RangeFrom<I>>>::Output>>(&'a self, offset: I, ctx: Ctx) -> N {
        N::from_ctx(&self[offset..], ctx)
    }
    /// Reads a value implementing `FromCtx` from `Self` at `offset`
    ///
    /// # Example
    ///
    /// ```rust
    /// use scroll::Cread;
    ///
    /// let bytes = [0x01,0x00,0x00,0x00,0x00,0x00,0x00,0x00, 0xef,0xbe,0x00,0x00,];
    /// let foo = bytes.cread::<usize>(0);
    /// let bar = bytes.cread::<u32>(8);
    /// assert_eq!(foo, 1);
    /// assert_eq!(bar, 0xbeef);
    /// ```
    #[inline]
    fn cread<'a, N: FromCtx<Ctx, <Self as Index<RangeFrom<I>>>::Output>>(&'a self, offset: I) -> N where Ctx: Default {
        let ctx = Ctx::default();
        N::from_ctx(&self[offset..], ctx)
    }
}

impl<Ctx: Copy, I, R: ?Sized + Index<I> + Index<RangeFrom<I>>> Cread<Ctx, I> for R {}

/// Core-write - core, no_std friendly trait for writing basic types into byte buffers. Cannot fail unless the buffer is too small, in which case an assert fires and the program panics.
/// Similar to [Cread](trait.Cread.html), if your type implements [IntoCtx](trait.IntoCtx.html) then you can `cwrite(your_type, offset)`.
///
/// # Example
///
/// ```rust
/// use scroll::{ctx, Cwrite};
///
/// #[repr(packed)]
/// struct Bar {
///     foo: i32,
///     bar: u32,
/// }
///
/// impl ctx::IntoCtx<scroll::Endian> for Bar {
///     fn into_ctx(self, bytes: &mut [u8], ctx: scroll::Endian) {
///         use scroll::Cwrite;
///         bytes.cwrite_with(self.foo, 0, ctx);
///         bytes.cwrite_with(self.bar, 4, ctx);
///     }
/// }
///
/// let bar = Bar { foo: -1, bar: 0xdeadbeef };
/// let mut bytes = [0x0; 0x10];
/// bytes.cwrite::<Bar>(bar, 0);
/// ```
pub trait Cwrite<Ctx: Copy, I = usize>: Index<I> + IndexMut<RangeFrom<I>> {
    /// Writes `n` into `Self` at `offset`; uses default context.
    ///
    /// # Example
    ///
    /// ```
    /// use scroll::{Cwrite, Cread};
    /// let mut bytes = [0x0; 0x10];
    /// bytes.cwrite::<usize>(42, 0);
    /// bytes.cwrite::<u32>(0xdeadbeef, 8);
    /// assert_eq!(bytes.cread::<usize>(0), 42);
    /// assert_eq!(bytes.cread::<u32>(8), 0xdeadbeef);
    #[inline]
    fn cwrite<N: IntoCtx<Ctx, <Self as Index<RangeFrom<I>>>::Output>>(&mut self, n: N, offset: I) where Ctx: Default {
        let ctx = Ctx::default();
        n.into_ctx(self.index_mut(offset..), ctx)
    }
    /// Writes `n` into `Self` at `offset` with `ctx`
    ///
    /// # Example
    ///
    /// ```
    /// use scroll::{Cwrite, Cread, LE, BE};
    /// let mut bytes = [0x0; 0x10];
    /// bytes.cwrite_with::<usize>(42, 0, LE);
    /// bytes.cwrite_with::<u32>(0xdeadbeef, 8, BE);
    /// assert_eq!(bytes.cread::<usize>(0), 42);
    /// assert_eq!(bytes.cread::<u32>(8), 0xefbeadde);
    #[inline]
    fn cwrite_with<N: IntoCtx<Ctx, <Self as Index<RangeFrom<I>>>::Output>>(&mut self, n: N, offset: I, ctx: Ctx) {
        n.into_ctx(self.index_mut(offset..), ctx)
    }
}

impl<Ctx: Copy, I, W: ?Sized + Index<I> + IndexMut<RangeFrom<I>>> Cwrite<Ctx, I> for W {}
