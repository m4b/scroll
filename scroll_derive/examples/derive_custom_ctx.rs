use scroll_derive::{Pread, Pwrite, SizeWith};

#[derive(Debug, PartialEq)]
struct CustomCtx {
    buf: Vec<u8>,
}
impl CustomCtx {
    fn len() -> usize {
        3 + 2
    }
}
impl<'a> TryFromCtx<'a, usize> for CustomCtx {
    type Error = scroll::Error;

    fn try_from_ctx(from: &'a [u8], ctx: usize) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let buf = from.gread_with::<&[u8]>(offset, ctx)?.to_owned();
        Ok((Self { buf }, *offset))
    }
}
impl<'a> TryIntoCtx<usize> for &'a CustomCtx {
    type Error = scroll::Error;
    fn try_into_ctx(self, dst: &mut [u8], ctx: usize) -> Result<usize, Self::Error> {
        let offset = &mut 0;
        for i in 0..(ctx.min(self.buf.len())) {
            dst.gwrite(self.buf[i], offset)?;
        }
        Ok(*offset)
    }
}
impl SizeWith<usize> for CustomCtx {
    fn size_with(ctx: &usize) -> usize {
        *ctx
    }
}

#[derive(Debug, PartialEq, Pread, Pwrite, SizeWith)]
#[repr(C)]
struct Data {
    id: u32,
    timestamp: f64,
    #[scroll(ctx = BE)]
    arr: [u16; 2],
    #[scroll(ctx = CustomCtx::len())]
    custom_ctx: CustomCtx,
}

use scroll::{
    ctx::{SizeWith, TryFromCtx, TryIntoCtx},
    Pread, Pwrite, BE, LE,
};

fn main() {
    let bytes = [
        0xefu8, 0xbe, 0xad, 0xde, 0, 0, 0, 0, 0, 0, 224, 63, 0xad, 0xde, 0xef, 0xbe, 0xaa, 0xbb,
        0xcc, 0xdd, 0xee,
    ];
    let data: Data = bytes.pread_with(0, LE).unwrap();
    println!("data: {data:?}");
    assert_eq!(data.id, 0xdeadbeefu32);
    assert_eq!(data.arr, [0xadde, 0xefbe]);
    let mut bytes2 = vec![0; ::std::mem::size_of::<Data>()];
    bytes2.pwrite_with(data, 0, LE).unwrap();
    let data: Data = bytes.pread_with(0, LE).unwrap();
    let data2: Data = bytes2.pread_with(0, LE).unwrap();
    assert_eq!(data, data2);

    /*
    let data: Data = bytes.cread_with(0, LE);
       assert_eq!(data, data2);
    */
}
