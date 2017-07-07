//! # Scroll
//!
//! ```text, no_run
//!         _______________
//!    ()==(              (@==()
//!         '______________'|
//!           |             |
//!           |   ἀρετή     |
//!         __)_____________|
//!    ()==(               (@==()
//!         '--------------'
//!
//! ```
//!
//! Scroll is a library for efficiently and easily reading/writing types from byte arrays. All the builtin types are supported, e.g., `u32`, `i8`, etc., where the type is specified as a type parameter, or type inferred when possible. In addition, it supports zero-copy reading of string slices, or any other kind of slice.  The library can be used in a no_std context as well; the [Error](enum.Error.html) type only has the `IO` and `String` variants if the default features are used, and is `no_std` safe when compiled without default features.

extern crate byte;

use byte::*;

mod leb128;

pub trait BytesExt<Ctx> {
    fn cread<'a, T>(&'a self, offset: usize) -> T
        where T: TryRead<'a, Ctx>,
              Ctx: Default;

    fn cread_with<'a, T>(&'a self, offset: usize, ctx: Ctx) -> T where T: TryRead<'a, Ctx>;

    fn pread<'a, T>(&'a self, offset: usize) -> Result<T>
        where T: TryRead<'a, Ctx>,
              Ctx: Default;

    fn pread_with<'a, T>(&'a self, offset: usize, ctx: Ctx) -> Result<T> where T: TryRead<'a, Ctx>;
}

impl<Ctx> BytesExt<Ctx> for [u8] {
    fn cread<'a, T>(&'a self, offset: usize) -> T
        where T: TryRead<'a, Ctx>,
              Ctx: Default
    {
        self.cread_with(offset, Default::default())
    }

    fn cread_with<'a, T>(&'a self, offset: usize, ctx: Ctx) -> T
        where T: TryRead<'a, Ctx>
    {
        self.pread_with(offset, ctx).expect("cread faild")
    }

    fn pread<'a, T>(&'a self, offset: usize) -> Result<T>
        where T: TryRead<'a, Ctx>,
              Ctx: Default
    {
        self.pread_with(offset, Default::default())
    }

    fn pread_with<'a, T>(&'a self, offset: usize, ctx: Ctx) -> Result<T>
        where T: TryRead<'a, Ctx>
    {
        use byte::*;

        let mut offset = offset;
        self.read_with(&mut offset, ctx)
    }
}
