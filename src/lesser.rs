use core::mem;
use std::io::{Result, Read, Write};
use ctx::FromCtx;

/// A Contextual extension trait to `std::io::Read` - WIP
pub trait Lread<Ctx: Copy + Default = super::Endian> : Read {
    fn lread<N: FromCtx<Ctx>>(&mut self, ctx: Ctx) -> Result<N> {
        // because we can't do this
        //let mut buf = [0; mem::size_of::<N>()];
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        let size = mem::size_of::<N>();
        Ok(N::from_ctx(&buf[0..size], ctx))
    }

    // fn lread_slice<N: TrySliceFrom<Ctx>>(&mut self, ctx: Ctx) -> Result<N> {
    //     let mut buf = [0; 8];
    //     self.read_exact(&mut buf)?;
    //     let size = mem::size_of::<N>();
    //     Ok(N::from_ctx(&buf[0..size], ctx))
    // }
}

impl<R: Read + ?Sized> Lread for R {}

trait Lwrite : Write {
}

/// Types that implement `Write` get methods defined in `Lwrite`
/// for free.
impl<W: Write + ?Sized> Lwrite for W {}
