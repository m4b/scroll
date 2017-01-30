use core::convert::{AsRef, AsMut};
use core::mem::size_of;
use core::result;
use core::fmt::Debug;
use core::ops::{Add, AddAssign};

use ctx::{TryFromCtx, TryRefFromCtx, TryIntoCtx};
use error::*;
use error;
use pread::Pread;
use pwrite::Pwrite;
use endian::Endian;

/// Attempt to add an offset for a given `N`'s size, used to compute error values in `Gread`, _or_ return the `N`'s size in units the same as the offset
///
/// NB: this trait's name is likely to be changed, tweaked slightly, if you are implementing an entire `Pread` stack, beware this could change
pub trait TryOffset<E = error::Error, I = usize> {
    /// Given the `offset`, see if a size + offset can safely be performed on `Self`, and return the resulting computed size
    fn try_offset<N>(&self, offset: I) -> result::Result<I, E>;
}

/// The Greater Read (`Gread`) reads a value at a mutable offset, and increments the offset by the size of the interpreted value.
///
/// `Gread` implements an immutable `Self`, `mutable` reference offset incrementor which uses `Pread` as its base.
/// If you are writing a custom `Gread` interface,
/// you should only need to implement `Pread` for a particular
/// `Ctx`, `Error`, `Index` target, _and_ implement `TryOffset` to explain to the trait how it should increment the mutable offset,
/// and then a simple blanket `impl Gread<E, I, Ctx> for YourType`, etc.
pub trait Gread<E = error::Error, Ctx = Endian, I = usize, TryCtx = (I, Ctx), SliceCtx = (I, I, Ctx)> : Pread<E, Ctx, I> + TryOffset<E, I>
    where Ctx: Copy + Default + Debug,
          I: AddAssign + Copy + Add + Default + Debug,
          E: Debug,
          TryCtx: Copy + Debug,
          SliceCtx: Copy + Default + Debug,
{
    #[inline]
    fn gread_unsafe<'a, N: TryFromCtx<'a, (I, Ctx), Error = E>>(&'a self, offset: &mut I, ctx: Ctx) -> N {
        let o = *offset;
        let count = self.try_offset::<N>(o).unwrap();
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
    fn gread<'a, N: TryFromCtx<'a, (I, Ctx), Error = E>>(&'a self, offset: &mut I) -> result::Result<N, E> {
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
    fn gread_with<'a, N: TryFromCtx<'a, (I, Ctx), Error = E>>(&'a self, offset: &mut I, ctx: Ctx) -> result::Result<N, E> {
        let o = *offset;
        let count = self.try_offset::<N>(o)?;
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
        where N: TryRefFromCtx<(I, I, Ctx), Error = E> {
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
    fn gread_inout<'a, N>(&'a self, offset: &mut I, inout: &mut [N]) -> result::Result<(), E>
        where
        N: TryFromCtx<'a, (I, Ctx), Error = E>,
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
    fn gread_inout_with<'a, N>(&'a self, offset: &mut I, inout: &mut [N], ctx: Ctx) -> result::Result<(), E>
        where
        N: TryFromCtx<'a, (I, Ctx), Error = E>,
    {
        let len = inout.len();
        for i in 0..len {
            inout[i] = self.gread_with(offset, ctx)?;
        }
        Ok(())
    }
}

impl TryOffset for [u8] {
    #[inline]
    fn try_offset<N>(&self, offset: usize) -> Result<usize> {
        let size = size_of::<N>();
        if offset + size > self.len() {
            Err(Error::BadOffset(format!("offset: {} size: {} len: {}", offset, size, self.len())).into())
        } else {
            Ok(size)
        }
    }
}

impl<T> TryOffset for T where T: AsRef<[u8]> {
    fn try_offset<N>(&self, offset: usize) -> Result<usize> {
        <[u8] as TryOffset>::try_offset::<N>(self.as_ref(), offset)
    }
}

// this gets us Gread for Buffer, Vec<u8>, etc.
impl<E, Ctx, T> Gread<E, Ctx> for T where T: AsRef<[u8]> + TryOffset<E>, Ctx: Copy + Default + Debug, E: Debug {}

// because Cursor doesn't impl AsRef<[u8]> and no specialization
// impl<T> TryOffset for Cursor<T> where T: AsRef<[u8]> {
// //impl Gread for ::std::io::Cursor<::std::vec::Vec<u8>> {
//     fn try_offset<N>(&self, offset: usize) -> Result<usize> {
//         <[u8] as TryOffset>::try_offset::<N>(&*self.get_ref().as_ref(), offset)
//     }
// }

/// The Greater Write (`Gwrite`) writes a value into its mutable insides, at a mutable offset
pub trait Gwrite<E = error::Error, Ctx = Endian, I = usize, TryCtx = (I, Ctx), SliceCtx = (I, I, Ctx)>: Pwrite<E, Ctx, I> + TryOffset<E, I>
 where E: Debug,
       Ctx: Copy + Default + Debug,
       I: AddAssign + Copy + Add + Default + Debug,
       TryCtx: Copy + Default + Debug,
       SliceCtx: Copy + Default + Debug,
{
    #[inline]
    fn gwrite_unsafe<N: TryIntoCtx<(I, Ctx), Error = E>>(&mut self, n: N, offset: &mut I, ctx: Ctx) {
        let o = *offset;
        let count = self.try_offset::<N>(o).unwrap();
        *offset += count;
        self.pwrite_unsafe(n, o, ctx)
    }
    fn gwrite<N: TryIntoCtx<(I, Ctx), Error = E>>(&mut self, n: N, offset: &mut I) -> result::Result<(), E> {
        let ctx = Ctx::default();
        self.gwrite_with(n, offset, ctx)
    }
    fn gwrite_with<N: TryIntoCtx<(I, Ctx), Error = E>>(&mut self, n: N, offset: &mut I, ctx: Ctx) -> result::Result<(), E> {
   let o = *offset;
        let count = self.try_offset::<N>(o)?;
        *offset += count;
        self.pwrite_unsafe(n, o, ctx);
        Ok(())
    }
}

impl<T> Gwrite for T where T: AsRef<[u8]> + AsMut<[u8]> {}
