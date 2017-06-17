use core::convert::{AsRef, AsMut};
use core::result;
use core::fmt::Debug;
use core::ops::{Add, AddAssign};
use core::ops::{Index, IndexMut, RangeFrom};

use ctx::{self, TryFromCtx, TryRefFromCtx, TryIntoCtx, FromCtx, IntoCtx, SizeWith};
use error::*;
use error;
use pread::Pread;
use pwrite::Pwrite;
use endian::Endian;

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
pub trait Gread<Ctx = Endian, E = error::Error, I = usize, TryCtx = (I, Ctx), SliceCtx = (I, I, Ctx)> : Pread<Ctx, E, I, TryCtx, SliceCtx> + TryOffsetWith<Ctx, E, I>
    where Ctx: Copy + Default + Debug,
          I: AddAssign + Copy + Add + Default + Debug,
          E: Debug,
          TryCtx: Copy + Default + Debug,
          SliceCtx: Copy + Default + Debug,
{
    #[inline]
    /// Reads _and_ unwraps a value from `self` at `offset` with the given `ctx`. **NB**: this can panic if the offset is bad, or whatever error this operates on is "thrown".
    /// For the primitive numeric values, this will read at the machine's endianness. Updates the offset
    fn gread_unsafe<'a, N: SizeWith<Ctx, Units = I> + TryFromCtx<'a, TryCtx, Error = E>>(&'a self, offset: &mut I, ctx: Ctx) -> N {
        let o = *offset;
        let count = self.try_offset::<N>(o, &ctx).unwrap();
        *offset += count;
        self.pread_unsafe(o, ctx)
    }
    #[inline]
    /// Reads a value from `self` at `offset` with a default `Ctx`. For the primitive numeric values, this will read at the machine's endianness. Updates the offset
    /// # Example
    /// ```rust
    /// use scroll::Gread;
    /// let offset = &mut 0;
    /// let bytes = [0x7fu8; 0x01];
    /// let byte = bytes.gread::<u8>(offset).unwrap();
    /// assert_eq!(*offset, 1);
    fn gread<'a, N: SizeWith<Ctx, Units = I> + TryFromCtx<'a, TryCtx, Error = E>>(&'a self, offset: &mut I) -> result::Result<N, E> {
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
    fn gread_with<'a, N: SizeWith<Ctx, Units = I> + TryFromCtx<'a, TryCtx, Error = E>>(&'a self, offset: &mut I, ctx: Ctx) -> result::Result<N, E> {
        let o = *offset;
        let count = self.try_offset::<N>(o, &ctx)?;
        let res = self.pread_unsafe(o, ctx);
        *offset += count;
        Ok(res)
    }
    /// Slices an `N` from `self` at `offset` up to `count` times, and updates the offset.
    /// # Example
    /// ```rust
    /// use scroll::Gread;
    /// let bytes: [u8; 2] = [0x48, 0x49];
    /// let offset = &mut 0;
    /// let hi: &str = bytes.gread_slice(offset, 2).unwrap();
    /// assert_eq!(hi, "HI");
    /// assert_eq!(*offset, 2);
    #[inline]
    fn gread_slice<N: ?Sized>(&self, offset: &mut I, count: I) -> result::Result<&N, E>
        where N: TryRefFromCtx<SliceCtx, Error = E> {
        let o = *offset;
        let res = self.pread_slice::<N>(o, count);
        if res.is_ok() { *offset += count;}
        res
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
        N: SizeWith<Ctx, Units = I> + TryFromCtx<'a, TryCtx, Error = E>,
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
    /// use scroll::{ctx, Gread};
    /// let mut bytes: Vec<u8> = vec![0, 0];
    /// let offset = &mut 0;
    /// let bytes_from: [u8; 2] = [0x48, 0x49];
    /// bytes_from.gread_inout_with(offset, &mut bytes, ctx::CTX).unwrap();
    /// assert_eq!(&bytes, &bytes_from);
    /// assert_eq!(*offset, 2);
    #[inline]
    fn gread_inout_with<'a, N>(&'a self, offset: &mut I, inout: &mut [N], ctx: Ctx) -> result::Result<(), E>
        where
        N: SizeWith<Ctx, Units = I> + TryFromCtx<'a, TryCtx, Error = E>,
    {
        let len = inout.len();
        for i in 0..len {
            inout[i] = self.gread_with(offset, ctx)?;
        }
        Ok(())
    }
}

impl<Ctx> TryOffsetWith<Ctx> for [u8] {
    #[inline]
    fn try_offset<N: SizeWith<Ctx, Units = usize>>(&self, offset: usize, ctx: &Ctx) -> Result<usize> {
        let size = N::size_with(ctx);
        if offset + size > self.len() {
            Err(error::Error::BadRange{range: offset..offset+size, size: self.len()})
        } else {
            Ok(size)
        }
    }
}

impl<Ctx, T> TryOffsetWith<Ctx> for T where T: AsRef<[u8]> {
    #[inline]
    fn try_offset<N: SizeWith<Ctx, Units = usize>>(&self, offset: usize, ctx: &Ctx) -> Result<usize> {
        <[u8] as TryOffsetWith<Ctx>>::try_offset::<N>(self.as_ref(), offset, ctx)
    }
}

// without this we get obscure lifetime errors from upstream clients
impl<Ctx, E> Gread<Ctx, E> for [u8] where
    [u8]: TryOffsetWith<Ctx, E>,
    Ctx: Copy + Default + Debug,
    E: Debug {}

// this gets us Gread for Buffer, Vec<u8>, etc.
impl<Ctx, E, T> Gread<Ctx, E> for T where
    T: AsRef<[u8]> + TryOffsetWith<Ctx, E>,
    Ctx: Copy + Default + Debug,
    E: Debug {}

// because Cursor doesn't impl AsRef<[u8]> and no specialization
// impl<T> TryOffsetWith for Cursor<T> where T: AsRef<[u8]> {
// //impl Gread for ::std::io::Cursor<::std::vec::Vec<u8>> {
//     fn try_offset<N>(&self, offset: usize) -> Result<usize> {
//         <[u8] as TryOffsetWith>::try_offset::<N>(&*self.get_ref().as_ref(), offset)
//     }
// }

/// The Greater Write (`Gwrite`) writes a value into its mutable insides, at a mutable offset
pub trait Gwrite<Ctx = Endian, E = error::Error, I = usize, TryCtx = (I, Ctx), SliceCtx = (I, I, Ctx)>: Pwrite<Ctx, E, I, TryCtx, SliceCtx> + TryOffsetWith<Ctx, E, I>
 where E: Debug,
       Ctx: Copy + Default + Debug,
       I: AddAssign + Copy + Add + Default + Debug,
       TryCtx: Copy + Default + Debug,
       SliceCtx: Copy + Default + Debug,
{
    #[inline]
    fn gwrite_unsafe<N: SizeWith<Ctx, Units = I> + TryIntoCtx<TryCtx, Error = E>>(&mut self, n: N, offset: &mut I, ctx: Ctx) {
        let o = *offset;
        let count = self.try_offset::<N>(o, &ctx).unwrap();
        *offset += count;
        self.pwrite_unsafe(n, o, ctx)
    }
    /// Write `n` into `self` at `offset`, with a default `Ctx`. Updates the offset.
    #[inline]
    fn gwrite<N: SizeWith<Ctx, Units = I> + TryIntoCtx<TryCtx, Error = E>>(&mut self, n: N, offset: &mut I) -> result::Result<(), E> {
        let ctx = Ctx::default();
        self.gwrite_with(n, offset, ctx)
    }
    /// Write `n` into `self` at `offset`, with the `ctx`. Updates the offset.
    #[inline]
    fn gwrite_with<N: SizeWith<Ctx, Units = I> + TryIntoCtx<TryCtx, Error = E>>(&mut self, n: N, offset: &mut I, ctx: Ctx) -> result::Result<(), E> {
        let o = *offset;
        let count = self.try_offset::<N>(o, &ctx)?;
        *offset += count;
        self.pwrite_unsafe(n, o, ctx);
        Ok(())
    }
}

impl<Ctx, E, T> Gwrite<Ctx, E> for T where
    T: AsRef<[u8]> + AsMut<[u8]> + TryOffsetWith<Ctx, E>,
    Ctx: Copy + Default + Debug,
    E: Debug {}

impl<Ctx, E> Gwrite<Ctx, E> for [u8] where
    [u8]: TryOffsetWith<Ctx, E>,
    Ctx: Copy + Default + Debug,
    E: Debug {}

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
/// impl ctx::FromCtx for Bar {
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
pub trait Cread<Ctx = super::Endian, I = usize> : Index<I> + Index<RangeFrom<I>>
 where
    Ctx: Copy + Default + Debug,
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
    fn cread<'a, N: FromCtx<Ctx, <Self as Index<RangeFrom<I>>>::Output>>(&'a self, offset: I) -> N {
        let ctx = Ctx::default();
        N::from_ctx(&self[offset..], ctx)
    }
}

impl<Ctx: Copy + Default + Debug, I, R: ?Sized + Index<I> + Index<RangeFrom<I>>> Cread<Ctx, I> for R {}

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
/// impl ctx::IntoCtx for Bar {
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
pub trait Cwrite<Ctx = super::Endian, I = usize>: Index<I> + IndexMut<RangeFrom<I>>
 where
    Ctx: Copy + Default + Debug {
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
    fn cwrite<N: IntoCtx<Ctx, <Self as Index<RangeFrom<I>>>::Output>>(&mut self, n: N, offset: I) {
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

impl<Ctx: Copy + Default + Debug, I, W: ?Sized + Index<I> + IndexMut<RangeFrom<I>>> Cwrite<Ctx, I> for W {}
