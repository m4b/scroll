use core::result;
use core::fmt::Debug;

use ctx::{TryIntoCtx};
use error;
use endian::Endian;

/// Writes into `Self` at an offset of type `I` using a `Ctx`
///
/// To implement writing into an arbitrary byte buffer, implement `TryIntoCtx`
/// # Example
/// ```rust
/// use scroll::{self, ctx};
/// #[derive(Debug, PartialEq, Eq)]
/// pub struct Foo(u16);
///
/// // this will use the default `DefaultCtx = scroll::Endian` and `I = usize`...
/// impl ctx::TryIntoCtx for Foo {
///     // you can use your own error here too, but you will then need to specify it in fn generic parameters
///     type Error = scroll::Error;
///     // you can write using your own context too... see `leb128.rs`
///     fn try_into_ctx(self, this: &mut [u8], ctx: (usize, scroll::ctx::DefaultCtx)) -> Result<(), Self::Error> {
///         use scroll::Pwrite;
///         let offset = ctx.0;
///         let le = ctx.1;
///         if offset > 2 { return Err((scroll::Error::Custom("whatever".to_string())).into()) }
///         this.pwrite_with(self.0, offset, le)?;
///         Ok(())
///     }
/// }
/// // now we can write a `Foo` into some buffer (in this case, a byte buffer, because that's what we implemented it for above)
/// use scroll::Pwrite;
/// let mut bytes: [u8; 4] = [0, 0, 0, 0];
/// bytes.pwrite_with(Foo(0x7f), 1, scroll::LE).unwrap();
///
pub trait Pwrite<Ctx = Endian, E = error::Error, I = usize, TryCtx = (I, Ctx), SliceCtx = (I, I, Ctx) >
 where E: Debug,
       Ctx: Copy + Default + Debug,
       I: Copy + Debug,
       TryCtx: Copy + Default + Debug,
       SliceCtx: Copy + Default + Debug,
{
    fn pwrite_unsafe<N: TryIntoCtx<TryCtx, Error = E>>(&mut self, n: N, offset: I, ctx: Ctx) {
        self.pwrite_with(n, offset, ctx).unwrap()
    }
    fn pwrite<N: TryIntoCtx<TryCtx, Error = E>>(&mut self, n: N, offset: I) -> result::Result<(), E> {
        self.pwrite_with(n, offset, Ctx::default())
    }
    /// Write `N` at offset `I` with context `Ctx`
    /// # Example
    /// ```
    /// use scroll::{Buffer, Pwrite, Pread, LE};
    /// let mut bytes: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
    /// bytes.pwrite_with::<u32>(0xbeefbeef, 0, LE).unwrap();
    /// assert_eq!(bytes.pread_with::<u32>(0, LE).unwrap(), 0xbeefbeef);
    fn pwrite_with<N: TryIntoCtx<TryCtx, Error = E>>(&mut self, n: N, offset: I, ctx: Ctx) -> result::Result<(), E>;
    //fn pwrite_slice<N: ?Sized + TrySliceFromCtx<SliceCtx, Error = E>>(&self, offset: I, count: I) -> result::Result<&N, E>;
}

impl<Ctx, E> Pwrite<Ctx, E> for [u8]
    where
    E: Debug,
    Ctx: Copy + Default + Debug
{
    // fn pwrite_unsafe<N: IntoCtx>(&mut self, n: N, offset: usize, le: bool) {
    //     n.into_ctx(&mut self[offset..], le);
    // }
    fn pwrite_with<N: TryIntoCtx<(usize, Ctx), Error = E>>(&mut self, n: N, offset: usize, le: Ctx) -> result::Result<(), E> {
        n.try_into_ctx(self, (offset, le))
    }
}

impl<T, Ctx, E> Pwrite<Ctx, E> for T where
    T: AsMut<[u8]>,
    E: Debug,
    Ctx: Copy + Debug + Default,
{
    fn pwrite_with<N: TryIntoCtx<(usize, Ctx), Error = E>>(&mut self, n: N, offset: usize, ctx: Ctx) -> result::Result<(), E> {
        <[u8] as Pwrite<Ctx, E>>::pwrite_with(self.as_mut(), n, offset, ctx)
    }
}
