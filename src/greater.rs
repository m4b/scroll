use core::result;
use core::fmt::Debug;
use core::ops::{Add, AddAssign};
use core::ops::{Index, IndexMut, RangeFrom};

use ctx::{TryFromCtx, TryIntoCtx, FromCtx, IntoCtx, SizeWith, MeasureWith};
use error;
use pread::Pread;
use pwrite::Pwrite;

/// Attempt to add an offset for a given `N`'s size, used to compute error values in `Gread`, _or_ return the `N`'s size in units the same as the offset
///
/// NB: this trait's name is likely to be changed, tweaked slightly, if you are implementing an entire `Pread` stack, beware this could change
pub trait TryOffsetWith<Ctx, E = error::Error, I = usize> {
    /// Given the `offset`, see if a size + offset can safely be performed on `Self`, and return the resulting computed size
    fn try_offset<N: SizeWith<Ctx, Units = I>>(&self, offset: I, ctx: &Ctx) -> result::Result<I, E>;
}

/// The Greater Read (`Gread`) reads a value at a mutable offset, and increments the offset by the size of the interpreted value.
///
/// `Gread` implements an immutable `Self`, `mutable` reference offset incrementor which uses `Pread` as its base.
/// If you are writing a custom `Gread` interface,
/// you should only need to implement `Pread` for a particular
/// `Ctx`, `Error`, `Index` target, _and_ implement `TryOffsetWith` to explain to the trait how it should increment the mutable offset,
/// and then a simple blanket `impl Gread<I, E, Ctx> for YourType`, etc.
pub trait Gread<Ctx, E, I = usize>: Pread<Ctx, E, I> + Index<RangeFrom<I>>
    where Ctx: Copy,
          I: AddAssign + Copy + Add + Default + Debug + PartialOrd,
          E: From<error::Error<I>> + Debug,
{
    #[inline]
    /// Reads a value from `self` at `offset` with a default `Ctx`. For the primitive numeric values, this will read at the machine's endianness. Updates the offset
    /// # Example
    /// ```rust
    /// use scroll::Gread;
    /// let offset = &mut 0;
    /// let bytes = [0x7fu8; 0x01];
    /// let byte = bytes.gread::<u8>(offset).unwrap();
    /// assert_eq!(*offset, 1);
    fn gread<'a, N: TryFromCtx<'a, Ctx, <Self as Index<RangeFrom<I>>>::Output, Error = E, Size = I>>(&'a self, offset: &mut I) -> result::Result<N, E> where Ctx: Default, <Self as Index<RangeFrom<I>>>::Output: 'a {
        let ctx = Ctx::default();
        self.gread_with(offset, ctx)
    }
    /// Reads a value from `self` at `offset` with the given `ctx`, and updates the offset.
    /// # Example
    /// ```rust
    /// use scroll::Gread;
    /// let offset = &mut 0;
    /// let bytes: [u8; 2] = [0xde, 0xad];
    /// let dead: u16 = bytes.gread_with(offset, scroll::BE).unwrap();
    /// assert_eq!(dead, 0xdeadu16);
    /// assert_eq!(*offset, 2);
    #[inline]
    fn gread_with<'a, N: TryFromCtx<'a, Ctx, <Self as Index<RangeFrom<I>>>::Output, Error = E, Size = I>>
        (&'a self, offset: &mut I, ctx: Ctx) ->
        result::Result<N, E>
        where <Self as Index<RangeFrom<I>>>::Output: 'a
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
    /// use scroll::Gread;
    /// let mut bytes: Vec<u8> = vec![0, 0];
    /// let offset = &mut 0;
    /// let bytes_from: [u8; 2] = [0x48, 0x49];
    /// bytes_from.gread_inout(offset, &mut bytes).unwrap();
    /// assert_eq!(&bytes, &bytes_from);
    /// assert_eq!(*offset, 2);
    #[inline]
    fn gread_inout<'a, N>(&'a self, offset: &mut I, inout: &mut [N]) -> result::Result<(), E>
        where
        N: TryFromCtx<'a, Ctx, <Self as Index<RangeFrom<I>>>::Output, Error = E, Size = I>,
    Ctx: Default,
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
    /// use scroll::{ctx, LE, Gread};
    /// let mut bytes: Vec<u8> = vec![0, 0];
    /// let offset = &mut 0;
    /// let bytes_from: [u8; 2] = [0x48, 0x49];
    /// bytes_from.gread_inout_with(offset, &mut bytes, LE).unwrap();
    /// assert_eq!(&bytes, &bytes_from);
    /// assert_eq!(*offset, 2);
    #[inline]
    fn gread_inout_with<'a, N>(&'a self, offset: &mut I, inout: &mut [N], ctx: Ctx) -> result::Result<(), E>
        where
        N: TryFromCtx<'a, Ctx, <Self as Index<RangeFrom<I>>>::Output, Error = E, Size = I>,
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
     I: Add + Copy + PartialOrd + AddAssign + Default + Debug,
     E: From<error::Error<I>> + Debug,
     R: ?Sized + Index<I> + Index<RangeFrom<I>> + MeasureWith<Ctx, Units = I>>
    Gread<Ctx, E, I> for R {}

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
