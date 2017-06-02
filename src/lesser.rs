use std::fmt::Debug;
use std::io::{Result, Read, Write};
use ctx::{FromCtx, SizeWith};
use error::{self};

/// An extension trait to `std::io::Read`; this only deserializes simple objects, like u8, u32, f32, usize, etc.
///
/// If you implement `FromCtx` for your type, you can then `lread::<YourType>()` on a `Read`.
pub trait Lread<Ctx = super::Endian, E = error::Error> : Read
 where E: Debug,
       Ctx: Copy + Default + Debug,
{
    fn lread<N: FromCtx<Ctx> + SizeWith<Ctx, Units = usize>>(&mut self) -> Result<N> {
        let ctx = Ctx::default();
        self.lread_with(ctx)
    }
    /// NB: this will allocate if the type you're reading has a size greater than 256
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
