use std::io::{Result, Read, Write};
use ctx::{FromCtx, IntoCtx, SizeWith};

/// An extension trait to `std::io::Read` streams; this only deserializes simple types, like `u8`, `i32`, `f32`, `usize`, etc.
///
/// If you implement [`FromCtx`](trait.FromCtx.html) and [`SizeWith`](ctx/trait.SizeWith.html) for your type, you can then `lread::<YourType>()` on a `Read`.  Note: [`FromCtx`](trait.FromCtx.html) is only meant for very simple types, and should _never_ fail.
///
/// **NB** You should probably add `repr(packed)` or `repr(C)` and be very careful how you implement [`SizeWith`](ctx/trait.SizeWith.html), otherwise you
/// will get IO errors failing to fill entire buffer (the size you specified in `SizeWith`), or out of bound errors (depending on your impl) in `from_ctx`
///
/// # Example
/// ```rust
/// use std::io::Cursor;
/// use scroll::{self, ctx, Pread, Lread};
///
/// #[repr(packed)]
/// struct Foo {
///     foo: usize,
///     bar: u32,
/// }
///
/// impl ctx::FromCtx<scroll::Endian> for Foo {
///     fn from_ctx(bytes: &[u8], ctx: scroll::Endian) -> Self {
///         Foo { foo: bytes.pread_with::<usize>(0, ctx).unwrap(), bar: bytes.pread_with::<u32>(8, ctx).unwrap() }
///     }
/// }
///
/// impl ctx::SizeWith<scroll::Endian> for Foo {
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
/// ```
///
pub trait Lread<Ctx: Copy> : Read
{
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
    fn lread<N: FromCtx<Ctx> + SizeWith<Ctx, Units = usize>>(&mut self) -> Result<N> where Ctx: Default {
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
    fn lread_with<N: FromCtx<Ctx> + SizeWith<Ctx, Units = usize>>(&mut self, ctx: Ctx) -> Result<N> {
        let mut scratch = [0u8; 256];
        let size = N::size_with(&ctx);
        let mut buf = &mut scratch[0..size];
        self.read_exact(&mut buf)?;
        Ok(N::from_ctx(buf, ctx))
    }
}

/// Types that implement `Read` get methods defined in `Lread`
/// for free.
impl<Ctx: Copy, R: Read + ?Sized> Lread<Ctx> for R {}

/// An extension trait to `std::io::Write` streams; this only serializes simple types, like `u8`, `i32`, `f32`, `usize`, etc.
///
/// To write custom types with a single `lwrite::<YourType>` call, implement [`IntoCtx`](trait.IntoCtx.html) and [`SizeWith`](ctx/trait.SizeWith.html) for `YourType`.
pub trait Lwrite<Ctx: Copy>: Write
{
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
    fn lwrite<N: SizeWith<Ctx, Units = usize> + IntoCtx<Ctx>>(&mut self, n: N) -> Result<()> where Ctx: Default {
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
    fn lwrite_with<N: SizeWith<Ctx, Units = usize> + IntoCtx<Ctx>>(&mut self, n: N, ctx: Ctx) -> Result<()> {
        let mut buf = [0u8; 256];
        let size = N::size_with(&ctx);
        let mut buf = &mut buf[0..size];
        n.into_ctx(buf, ctx);
        self.write_all(buf)?;
        Ok(())
    }
}

/// Types that implement `Write` get methods defined in `Lwrite`
/// for free.
impl<Ctx: Copy, W: Write + ?Sized> Lwrite<Ctx> for W {}
