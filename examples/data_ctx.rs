extern crate scroll;

use scroll::{ctx, Endian, Pread, BE};

#[derive(Debug)]
struct Data<'a> {
    name: &'a str,
    id: u32,
}

impl<'a> ctx::TryFromCtx<'a, (usize, Endian)> for Data<'a> {
    type Error = scroll::Error;
    fn try_from_ctx (src: &'a [u8], (offset, endian): (usize, Endian))
                     -> Result<Self, Self::Error> {
        let name = src.pread::<&'a str>(offset)?;
        let id = src.pread_with(offset+name.len(), endian)?;
        Ok(Data { name: name, id: id })
    }
}

fn main() {
    let bytes = b"UserName\x01\x02\x03\x04\x00";
    let data = bytes.pread_with::<Data>(0, BE).unwrap();
    assert_eq!(data.id, 0x01020304);
    assert_eq!(data.name.to_string(), "UserName".to_string());
    println!("Data: {:?}", &data);
}
