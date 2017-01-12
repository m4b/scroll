extern crate scroll;

use scroll::{ctx, Pread, BE};

#[derive(Debug)]
struct Data {
    name: String,
    id: u32,
}

#[derive(Debug, Clone, Copy, Default)]
struct DataCtx {
    pub size: usize,
    pub endian: scroll::Endian
}

impl ctx::TryFromCtx<(usize, DataCtx)> for Data {
    type Error = scroll::Error;
    fn try_from_ctx (src: &[u8], (offset, DataCtx {size, endian}): (usize, DataCtx))
                     -> Result<Self, Self::Error> {
        let name = src.pread_slice::<str>(offset, size)?.to_string();
        let id = src.pread(offset+size, endian)?;
        Ok(Data { name: name, id: id })
    }
}

fn main() {
    let bytes = scroll::Buffer::new(b"UserName\x01\x02\x03\x04");
    let data = bytes.pread::<Data>(0, DataCtx { size: 8, endian: BE }).unwrap();
    assert_eq!(data.id, 0x01020304);
    assert_eq!(data.name, "UserName".to_string());
    println!("Data: {:?}", &data);
}
