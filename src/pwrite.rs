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
/// // this will use the default `Ctx = scroll::Endian` and `I = usize`...
/// impl ctx::TryIntoCtx for Foo {
///     // you can use your own error here too, but you will then need to specify it in fn generic parameters
///     type Error = scroll::Error;
///     // you can write using your own context too... see `leb128.rs`
///     fn into_ctx(self, this: &mut [u8], ctx: (usize, scroll::Endian)) -> Result<(), Self::Error> {
///         use scroll::Pwrite;
///         let offset = ctx.0;
///         let le = ctx.1;
///         if offset > 2 { return Err((scroll::Error::BadOffset("whatever".to_string())).into()) }
///         this.pwrite(self.0, offset, le)?;
///         Ok(())
///     }
/// }
/// // now we can write a `Foo` into some buffer (in this case, a byte buffer, because that's what we implemented it for above)
/// use scroll::Pwrite;
/// let mut bytes: [u8; 4] = [0, 0, 0, 0];
/// bytes.pwrite(Foo(0x7f), 1, scroll::LE).unwrap();
///
pub trait Pwrite<E = error::Error, Ctx = Endian, I = usize, TryCtx = (I, Ctx), SliceCtx = (I, I, Ctx) >
 where E: Debug,
       Ctx: Copy + Default + Debug,
       I: Copy + Debug,
       TryCtx: Copy + Default + Debug,
       SliceCtx: Copy + Default + Debug,
{
    fn pwrite_unsafe<N: TryIntoCtx<TryCtx, Error = E>>(&mut self, n: N, offset: I, ctx: Ctx) {
        self.pwrite(n, offset, ctx).unwrap()
    }
    fn pwrite_into<N: TryIntoCtx<TryCtx, Error = E>>(&mut self, n: N, offset: I) -> result::Result<(), E> {
        self.pwrite(n, offset, Ctx::default())
    }
    /// Write `N` at offset `I` with context `Ctx`
    /// # Example
    /// ```
    /// use scroll::{Buffer, Pwrite, Pread, LE};
    /// let mut bytes: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
    /// bytes.pwrite::<u32>(0xbeefbeef, 0, LE).unwrap();
    /// assert_eq!(bytes.pread::<u32>(0, LE).unwrap(), 0xbeefbeef);
    fn pwrite<N: TryIntoCtx<TryCtx, Error = E>>(&mut self, n: N, offset: I, ctx: Ctx) -> result::Result<(), E>;
    //fn pwrite_slice<N: ?Sized + TrySliceFromCtx<SliceCtx, Error = E>>(&self, offset: I, count: I) -> result::Result<&N, E>;
}

impl<E, Ctx> Pwrite<E, Ctx> for [u8]
    where
    E: Debug,
    Ctx: Copy + Default + Debug
{
    // fn pwrite_unsafe<N: IntoCtx>(&mut self, n: N, offset: usize, le: bool) {
    //     n.into_ctx(&mut self[offset..], le);
    // }
    fn pwrite<N: TryIntoCtx<(usize, Ctx), Error = E>>(&mut self, n: N, offset: usize, le: Ctx) -> result::Result<(), E> {
        n.into_ctx(self, (offset, le))
    }
}

impl<T, E, Ctx> Pwrite<E, Ctx> for T where
    T: AsMut<[u8]>,
    E: Debug,
    Ctx: Copy + Debug + Default,
{
    fn pwrite<N: TryIntoCtx<(usize, Ctx), Error = E>>(&mut self, n: N, offset: usize, ctx: Ctx) -> result::Result<(), E> {
        <[u8] as Pwrite<E, Ctx>>::pwrite(self.as_mut(), n, offset, ctx)
    }
}
