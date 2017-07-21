use core::ops::{Index, IndexMut, RangeFrom};

use ctx::{IntoCtx};

// /// Core-read - core, no_std friendly trait for reading basic traits from byte buffers. Cannot fail unless the buffer is too small, in which case an assert fires and the program panics.
// ///
// /// If your type implements [FromCtx](trait.FromCtx.html) then you can `cread::<YourType>(offset)`.
// ///
// /// # Example
// ///
// /// ```rust
// /// use scroll::{ctx, Cread};
// ///
// /// #[repr(packed)]
// /// struct Bar {
// ///     foo: i32,
// ///     bar: u32,
// /// }
// ///
// /// impl ctx::FromCtx<scroll::Endian> for Bar {
// ///     fn from_ctx(bytes: &[u8], ctx: scroll::Endian) -> Self {
// ///         use scroll::Cread;
// ///         Bar { foo: bytes.cread_with(0, ctx), bar: bytes.cread_with(4, ctx) }
// ///     }
// /// }
// ///
// /// let bytes = [0xff, 0xff, 0xff, 0xff, 0xef,0xbe,0xad,0xde,];
// /// let bar = bytes.cread::<Bar>(0);
// /// assert_eq!(bar.foo, -1);
// /// assert_eq!(bar.bar, 0xdeadbeef);
// /// ```
// pub trait Cread<Ctx, I = usize> : Index<I> + Index<RangeFrom<I>>
//  where
//     Ctx: Copy,
// {
// }

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
