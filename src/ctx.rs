use core::ptr::copy_nonoverlapping;
use core::mem::transmute;
use core::str;

use error;

/// Reads `Self` from `This` using the context `Ctx`
pub trait FromCtx<Ctx: Copy = super::Endian, This: ?Sized = [u8]> where Self: Sized {
    #[inline]
    fn from_ctx(this: &This, ctx: Ctx) -> Self;
}

/// Tries to read `Self` from `This` using the context `Ctx`
pub trait TryFromCtx<Ctx: Copy = (usize, super::Endian), This: ?Sized = [u8]> where Self: Sized {
    type Error;
    #[inline]
    fn try_from_ctx(from: &This, ctx: Ctx) -> Result<Self, Self::Error>;
}

/// Writes `Self` into `This` using the context `Ctx`
pub trait IntoCtx<Ctx: Copy = super::Endian, This: ?Sized = [u8]>: Sized {
    fn into_ctx(self, &mut This, ctx: Ctx);
}

/// Tries to write `Self` into `This` using the context `Ctx`
pub trait TryIntoCtx<Ctx: Copy = (usize, super::Endian), This: ?Sized = [u8]>: Sized {
    type Error;
    fn into_ctx(self, &mut This, ctx: Ctx) -> Result<(), Self::Error>;
}

pub trait RefFrom<This: ?Sized = [u8], I = usize> {
    type Error;
    #[inline]
    fn ref_from(from: &This, offset: I, count: I) -> Result<&Self, Self::Error>;
}

/// Tries to read a reference from `This` using the context `Ctx`
pub trait TryRefFromCtx<Ctx: Copy = (usize, usize, super::Endian), This: ?Sized = [u8]> {
    type Error;
    #[inline]
    fn try_ref_from_ctx(from: &This, ctx: Ctx) -> Result<&Self, Self::Error>;
}

// TODO: fixme
/// Tries to write `Self` into `This` using the context `Ctx`
pub trait TryRefIntoCtx<Ctx: Copy = (usize, usize, super::Endian), This: ?Sized = [u8]>: Sized {
    type Error;
    fn try_ref_into_ctx(self, &mut This, ctx: Ctx) -> Result<(), Self::Error>;
}

impl<T> TryRefFromCtx<(usize, usize, super::Endian), T> for [u8] where T: AsRef<[u8]> {
    type Error = error::Error;
    #[inline]
    fn try_ref_from_ctx(b: &T, ctx: (usize, usize, super::Endian)) -> error::Result<&[u8]> {
       let (offset, count) = (ctx.0, ctx.1);
        let b = b.as_ref();
        if offset + count > b.len () {
            Err(error::Error::BadOffset(format!("count: {}, offset: {},  len: {}", count, offset, b.len())).into())
        } else {
            Ok(&b[offset..(offset+count)])
        }
    }
}

// uses the default generics
impl TryRefFromCtx for [u8] {
    type Error = error::Error;
    #[inline]
    fn try_ref_from_ctx(b: &[u8], ctx: (usize, usize, super::Endian)) -> error::Result<&[u8]> {
        let (offset, count) = (ctx.0, ctx.1);
        if offset + count > b.len () {
            Err(error::Error::BadOffset(format!("count: {}, offset: {},  len: {}", count, offset, b.len())).into())
        } else {
            Ok(&b[offset..(offset+count)])
        }
    }
}


impl<T> RefFrom<T> for [u8] where T: AsRef<[u8]> {
    type Error = error::Error;
    #[inline]
    fn ref_from(b: &T, offset: usize, count: usize) -> error::Result<&[u8]> {
        let b = b.as_ref();
        if offset + count > b.len () {
            Err(error::Error::BadOffset(format!("count: {}, offset: {},  len: {}", count, offset, b.len())).into())
        } else {
            Ok(&b[offset..(offset+count)])
        }
    }
}

impl RefFrom for [u8] {
    type Error = error::Error;
    #[inline]
    fn ref_from(b: &[u8], offset: usize, count: usize) -> error::Result<&[u8]> {
        if offset + count > b.len () {
            Err(error::Error::BadOffset(format!("count: {}, offset: {},  len: {}", count, offset, b.len())).into())
        } else {
            Ok(&b[offset..(offset+count)])
        }
    }
}

// TODO: map the utf8_unchecked into a BadInput value and remove unsafety
impl<T> RefFrom<T> for str where T: AsRef<[u8]> {
    type Error = error::Error;
    #[inline]
    fn ref_from(b: &T, offset: usize, count: usize) -> error::Result<&str> {
        let b = b.as_ref();
        if offset + count > b.len () {
            Err(error::Error::BadOffset(format!("count: {}, offset: {},  len: {}", count, offset, b.len())).into())
        } else {
            Ok (unsafe { str::from_utf8_unchecked(&b[offset..(offset+count)]) })
        }
    }
}

impl RefFrom for str {
    type Error = error::Error;
    #[inline]
    fn ref_from(b: &[u8], offset: usize, count: usize) -> error::Result<&str> {
        if offset + count > b.len () {
            Err(error::Error::BadOffset(format!("count: {}, offset: {},  len: {}", count, offset, b.len())).into())
        } else {
            Ok (unsafe { str::from_utf8_unchecked(&b[offset..(offset+count)]) })
        }
    }
}

impl TryRefFromCtx for str {
    type Error = error::Error;
    #[inline]
    fn try_ref_from_ctx(b: &[u8], ctx: (usize, usize, super::Endian)) -> error::Result<&str> {
        let (offset, count) = (ctx.0, ctx.1);
        if offset + count > b.len () {
            Err(error::Error::BadOffset(format!("count: {}, offset: {},  len: {}", count, offset, b.len())).into())
        } else {
            Ok (unsafe { str::from_utf8_unchecked(&b[offset..(offset+count)]) })
        }
    }
}

macro_rules! signed_to_unsigned {
    (i8) =>  {u8 };
    (u8) =>  {u8 };
    (i16) => {u16};
    (u16) => {u16};
    (i32) => {u32};
    (u32) => {u32};
    (i64) => {u64};
    (u64) => {u64};
    (f32) => {u32};
    (f64) => {u64};
}

macro_rules! write_into {
    ($typ:ty, $size:expr, $n:expr, $dst:expr, $endian:expr) => ({
        unsafe {
            let bytes = transmute::<_, [u8; $size]>(if $endian.is_little() { $n.to_le() } else { $n.to_be() });
            copy_nonoverlapping((&bytes).as_ptr(), $dst.as_mut_ptr(), $size);
        }
    });
}

macro_rules! into_ctx_impl {
    ($typ:tt, $size:expr, $ctx:ty) => {
        impl IntoCtx for $typ {
            #[inline]
            fn into_ctx(self, dst: &mut [u8], le: super::Endian) {
                write_into!(signed_to_unsigned!($typ), $size, self, dst, le);
            }
        }
        impl TryIntoCtx<(usize, $ctx)> for $typ where $typ: IntoCtx<$ctx> {
            type Error = error::Error;
            #[inline]
            fn into_ctx(self, dst: &mut [u8], ctx: (usize, super::Endian)) -> error::Result<()> {
                let offset = ctx.0;
                let le = ctx.1;
                if offset + $size > dst.len () {
                    return Err(error::Error::BadOffset(format!("size_of: {}, offset: {},  len: {}", $size, offset, dst.len())).into())
                }
                <$typ as IntoCtx<$ctx>>::into_ctx(self, &mut dst[offset..(offset+$size)], le);
                Ok(())
            }
        }
    }
}

macro_rules! from_ctx_impl {
    ($typ:tt, $size:expr, $ctx:ty) => {
        impl FromCtx<$ctx> for $typ {
            #[inline]
            fn from_ctx(src: &[u8], le: $ctx) -> Self {
                let mut data: signed_to_unsigned!($typ) = 0;
                unsafe {
                    copy_nonoverlapping(
                        src.as_ptr(),
                        &mut data as *mut signed_to_unsigned!($typ) as *mut u8,
                        $size);
                }
                (if le.is_little() { data.to_le() } else { data.to_be() }) as $typ
            }
        }

        impl TryFromCtx<(usize, $ctx)> for $typ where $typ: FromCtx<$ctx> {
            type Error = error::Error;
            #[inline]
            fn try_from_ctx(src: &[u8], ctx: (usize, $ctx)) -> error::Result<Self> {
                let offset = ctx.0;
                let le = ctx.1;
                if offset + $size > src.len () {
                    return Err(error::Error::BadOffset(format!("value size: {}, offset: {}, src len: {}", $size, offset, src.len())).into())
                }
                Ok(FromCtx::from_ctx(&src[offset..(offset + $size)], le))
            }
        }
        // as ref
        impl<T> FromCtx<$ctx, T> for $typ where T: AsRef<[u8]> {
            #[inline]
            fn from_ctx(src: &T, le: $ctx) -> Self {
                let src = src.as_ref();
                let mut data: signed_to_unsigned!($typ) = 0;
                unsafe {
                    copy_nonoverlapping(
                        src.as_ptr(),
                        &mut data as *mut signed_to_unsigned!($typ) as *mut u8,
                        $size);
                }
                (if le.is_little() { data.to_le() } else { data.to_be() }) as $typ
            }
        }

        impl<T> TryFromCtx<(usize, $ctx), T> for $typ where $typ: FromCtx<$ctx, T>, T: AsRef<[u8]> {
            type Error = error::Error;
            #[inline]
            fn try_from_ctx(src: &T, ctx: (usize, $ctx)) -> error::Result<Self> {
                let src = src.as_ref();
                let offset = ctx.0;
                let le = ctx.1;
                if offset + $size > src.len () {
                    return Err(error::Error::BadOffset(format!("value size: {}, offset: {}, src len: {}", $size, offset, src.len())).into())
                }
                Ok(FromCtx::from_ctx(&src[offset..(offset + $size)], le))
            }
        }

    };
}

macro_rules! ctx_impl {
    ($typ:tt, $size:expr) => {
        from_ctx_impl!($typ, $size, super::Endian);
     };
}

ctx_impl!(u8, 1);
ctx_impl!(i8, 1);
ctx_impl!(u16, 2);
ctx_impl!(i16, 2);
ctx_impl!(u32, 4);
ctx_impl!(i32, 4);
ctx_impl!(u64, 8);
ctx_impl!(i64, 8);
ctx_impl!(f32, 4);
ctx_impl!(f64, 8);

into_ctx_impl!(u8,  1, super::Endian);
into_ctx_impl!(i8,  1, super::Endian);
into_ctx_impl!(u16, 2, super::Endian);
into_ctx_impl!(i16, 2, super::Endian);
into_ctx_impl!(u32, 4, super::Endian);
into_ctx_impl!(i32, 4, super::Endian);
into_ctx_impl!(u64, 8, super::Endian);
into_ctx_impl!(i64, 8, super::Endian);

impl IntoCtx for f32 {
    #[inline]
    fn into_ctx(self, dst: &mut [u8], le: super::Endian) {
        write_into!(u32, 4, transmute::<f32, u32>(self), dst, le);
    }
}

impl TryIntoCtx<(usize, super::Endian)> for f32 where f32: IntoCtx<super::Endian> {
    type Error = error::Error;
    #[inline]
    fn into_ctx(self, dst: &mut [u8], ctx: (usize, super::Endian)) -> error::Result<()> {
        let offset = ctx.0;
        let le = ctx.1;
        if offset + 4 > dst.len () {
            return Err(error::Error::BadOffset(format!("size_of: {}, offset: {},  len: {}", 4, offset, dst.len())).into())
        }
        <f32 as IntoCtx<super::Endian>>::into_ctx(self, &mut dst[offset..(offset+4)], le);
        Ok(())
    }
}

impl IntoCtx for f64 {
    #[inline]
    fn into_ctx(self, dst: &mut [u8], le: super::Endian) {
        write_into!(u64, 8, transmute::<f64, u64>(self), dst, le);
    }
}
impl TryIntoCtx<(usize, super::Endian)> for f64 where f64: IntoCtx<super::Endian> {
    type Error = error::Error;
    #[inline]
    fn into_ctx(self, dst: &mut [u8], ctx: (usize, super::Endian)) -> error::Result<()> {
        let offset = ctx.0;
        let le = ctx.1;
        if offset + 8 > dst.len () {
            return Err(error::Error::BadOffset(format!("size_of: {}, offset: {},  len: {}", 8, offset, dst.len())).into())
        }
        <f64 as IntoCtx<super::Endian>>::into_ctx(self, &mut dst[offset..(offset+8)], le);
        Ok(())
    }
}
