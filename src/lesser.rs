use std::fmt::Debug;
use std::io::{Result, Read, Write};
use ctx::{FromCtx, SizeWith};
use error::{self};

/// An extension trait to `std::io::Read`; this only deserializes simple objects, like u8, u32, f32, usize, etc.
///
/// If you implement `FromCtx` for your type, you can then `lread::<YourType>()` on a `Read`.  Note: `FromCtx` is only meant for very simple types, and should _never_ fail.
///
/// # Example
///
/// **NB** You should probably add `repr(packed)` or `repr(C)` and be very careful how you implement `SizeWith`, otherwise you
/// will get IO errors failing to fill entire buffer (the size you specified in SizeWith)
///
///```rust
/// use std::io::Cursor;
/// use scroll::{self, ctx, Pread, Lread};
///
/// #[repr(packed)]
/// struct Foo {
///     foo: usize,
///     bar: u32,
/// }
///
/// impl ctx::FromCtx for Foo {
///     fn from_ctx(bytes: &[u8], ctx: scroll::Endian) -> Self {
///         Foo { foo: bytes.pread_unsafe::<usize>(0, ctx), bar: bytes.pread_unsafe::<u32>(8, ctx) }
///     }
/// }
///
/// impl ctx::SizeWith for Foo {
///     type Units = usize;
///     // our parsing context doesn't influence our size
///     fn size_with(_: &scroll::Endian) -> Self::Units {
///         ::std::mem::size_of::<Foo>()
///     }
/// }
///
/// let bytes_ = [0x0b,0x0b,0x00,0x00,0x00,0x00,0x00,0x00, 0xef,0xbe,0x00,0x00,];
/// let mut bytes = Cursor::new(bytes_);
/// let foo = bytes.lread::<usize>().unwrap();
/// let bar = bytes.lread::<u32>().unwrap();
/// assert_eq!(foo, 0xb0b);
/// assert_eq!(bar, 0xbeef);
/// let error = bytes.lread::<f64>();
/// assert!(error.is_err());
/// let mut bytes = Cursor::new(bytes_);
/// let foo_ = bytes.lread::<Foo>().unwrap();
/// assert_eq!(foo_.foo, foo);
/// assert_eq!(foo_.bar, bar);
///```
pub trait Lread<Ctx = super::Endian, E = error::Error> : Read
 where E: Debug,
       Ctx: Copy + Default + Debug,
{
    /// Reads `N` from `Self`, with a default parsing context.
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
    fn lread<N: FromCtx<Ctx> + SizeWith<Ctx, Units = usize>>(&mut self) -> Result<N> {
        let ctx = Ctx::default();
        self.lread_with(ctx)
    }

    /// Reads `N` from `Self`, with the parsing context `ctx`.
    /// NB: this will panic if the type you're reading has a size greater than 256. Plans are to have this allocate in larger cases.
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
    fn lread_with<N: FromCtx<Ctx> + SizeWith<Ctx, Units = usize>>(&mut self, ctx: Ctx) -> Result<N> {
        let mut scratch = [0u8; 256];
        let size = N::size_with(&ctx);
        let mut buf = &mut scratch[0..size];
        self.read_exact(&mut buf)?;
        Ok(N::from_ctx(buf, ctx))
    }
}

impl<R: Read + ?Sized> Lread for R {}

trait Lwrite : Write {
}

/// Types that implement `Write` get methods defined in `Lwrite`
/// for free.
impl<W: Write + ?Sized> Lwrite for W {}
