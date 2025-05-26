use scroll::{Cread, Cwrite, IOread, IOwrite, Pread, Pwrite, BE, LE};
use scroll_derive::{IOread, IOwrite, Pread, Pwrite, SizeWith};

use scroll::ctx::SizeWith;

macro_rules! test {
    ( struct $name:ident { $( $field:ident: $t:ty, )* } ) => {
	// check we can exist inside a macro_rules
	// https://github.com/m4b/scroll/pull/75
        #[derive(Pread, Pwrite)]
        pub struct $name {
            $( pub $field: $t, )*
        }
    };
}

test! {
    struct Test {
        field: [u16; 40],
    }
}

#[derive(Debug, PartialEq, Pread, Pwrite)]
struct Data {
    id: u32,
    timestamp: f64,
}

#[test]
fn test_data() {
    let bytes = [0xefu8, 0xbe, 0xad, 0xde, 0, 0, 0, 0, 0, 0, 224, 63];
    let data: Data = bytes.pread_with(0, LE).unwrap();
    println!("data: {data:?}");
    assert_eq!(data.id, 0xdeadbeefu32);
    assert_eq!(data.timestamp, 0.5f64);
    let mut bytes2 = vec![0; ::std::mem::size_of::<Data>()];
    bytes2.pwrite_with(data, 0, LE).unwrap();
    let data: Data = bytes.pread_with(0, LE).unwrap();
    let data2: Data = bytes2.pread_with(0, LE).unwrap();
    assert_eq!(data, data2);
}

#[derive(Debug, PartialEq, Pread, Pwrite)]
struct Data2 {
    name: [u8; 32],
}

#[test]
fn test_array() {
    let bytes = [0u8; 64];
    let data: Data2 = bytes.pread_with(0, LE).unwrap();
    println!("data: {data:?}");
}

#[derive(Debug, PartialEq, Pread, Pwrite, SizeWith)]
struct Data3 {
    name: u32,
}

#[test]
fn test_sizewith() {
    let bytes = [0u8; 64];
    let data: Data3 = bytes.gread_with(&mut 0, LE).unwrap();
    println!("data: {data:?}");
}

#[derive(Debug, PartialEq, IOread, IOwrite, SizeWith)]
struct Data4 {
    name: u32,
    j: u16,
    arr: [u8; 2],
}

#[test]
fn test_ioread() {
    let bytes = [0, 1, 2, 3, 0xde, 0xed, 0xbe, 0xaf];
    let data: Data4 = bytes.cread_with(0, LE);
    println!("data: {data:?}");
    assert_eq!(data.name, 50462976);
    assert_eq!(data.j, 0xedde);
    assert_eq!(data.arr, [0xbe, 0xaf]);
}

#[test]
fn test_iowrite() {
    let bytes = [0, 1, 2, 3, 0xde, 0xed, 0xbe, 0xaf];
    let data: Data4 = bytes.cread_with(0, LE);
    println!("data: {data:?}");
    assert_eq!(data.name, 50462976);
    assert_eq!(data.j, 0xedde);
    assert_eq!(data.arr, [0xbe, 0xaf]);

    let mut bytes_null = [0u8; 8];
    bytes_null.cwrite_with(&data, 0, LE);
    println!("bytes_null: {bytes_null:?}");
    println!("bytes     : {bytes:?}");
    assert_eq!(bytes_null, bytes);

    let mut bytes_null = [0u8; 8];
    bytes_null.cwrite_with(data, 0, LE);
    println!("bytes_null: {bytes_null:?}");
    println!("bytes     : {bytes:?}");
    assert_eq!(bytes_null, bytes);
}

#[derive(Debug, PartialEq, Pread, SizeWith)]
#[repr(C)]
struct Data5 {
    name: u32,
    j: u16,
    arr1: [u8; 2],
    arr2: [u16; 2],
}

#[test]
fn test_pread_arrays() {
    let bytes = [0, 1, 2, 3, 0, 0, 0xde, 0xed, 0xad, 0xde, 0xef, 0xbe];
    let data: Data5 = bytes.pread_with(0, LE).unwrap();
    println!("data: {data:?}");
    assert_eq!(data.name, 50462976);
    assert_eq!(data.arr1, [0xde, 0xed]);
    assert_eq!(data.arr2, [0xdead, 0xbeef]);
    let offset = &mut 0;
    let data: Data5 = bytes.gread_with(offset, LE).unwrap();
    println!("data: {data:?}");
    assert_eq!(data.name, 50462976);
    assert_eq!(data.arr1, [0xde, 0xed]);
    assert_eq!(data.arr2, [0xdead, 0xbeef]);
    assert_eq!(*offset, ::std::mem::size_of::<Data5>());
}

#[derive(Debug, PartialEq, Pread, SizeWith)]
#[repr(C)]
struct Data6 {
    id: u32,
    name: [u8; 5],
}

#[test]
fn test_array_copy() {
    let bytes = [0xde, 0xed, 0xef, 0xbe, 0x68, 0x65, 0x6c, 0x6c, 0x0];
    let data: Data6 = bytes.pread_with(0, LE).unwrap();
    let name: &str = data.name.pread(0).unwrap();
    println!("data: {data:?}");
    println!("data.name: {name:?}");
    assert_eq!(data.id, 0xbeefedde);
    assert_eq!(name, "hell");
}

#[derive(Debug, PartialEq, Eq, Pread, Pwrite, SizeWith)]
struct Data7A {
    pub y: u64,
    pub x: u32,
}

#[derive(Debug, PartialEq, Eq, Pread, Pwrite, SizeWith)]
struct Data7B {
    pub a: Data7A,
}

#[test]
fn test_nested_struct() {
    let b = Data7B {
        a: Data7A { y: 1, x: 2 },
    };
    let size = Data7B::size_with(&LE);
    assert_eq!(size, 12);
    let mut bytes = vec![0; size];
    let written = bytes.pwrite_with(&b, 0, LE).unwrap();
    assert_eq!(written, size);
    let mut read = 0;
    let b2: Data7B = bytes.gread_with(&mut read, LE).unwrap();
    assert_eq!(read, size);
    assert_eq!(b, b2);
}

#[derive(Debug, PartialEq, Eq, Pread, Pwrite, IOread, IOwrite, SizeWith)]
#[repr(C)]
struct Data8<T, Y> {
    ids: [T; 3],
    xyz: Y,
}
#[test]
fn test_generics() {
    let mut bytes = [0xde, 0xed, 0xef, 0x10, 0x10];
    let data: Data8<u8, u16> = bytes.pread_with(0, LE).unwrap();
    assert_eq!(data.ids, [0xde, 0xed, 0xef]);
    assert_eq!(data.xyz, 0x1010);
    let data: Data8<u8, u16> = bytes.cread_with(0, LE);
    assert_eq!(data.ids, [0xde, 0xed, 0xef]);
    assert_eq!(data.xyz, 0x1010);
    let size = Data8::<u8, u16>::size_with(&LE);
    let written = bytes.pwrite_with(&data, 0, LE).unwrap();
    assert_eq!(written, size);
}

#[derive(Debug, PartialEq, Eq, Pread, Pwrite, IOread, IOwrite, SizeWith)]
struct Data9(u8, u16);

#[test]
fn test_newtype() {
    let mut bytes = [0xde, 0x10, 0x10];
    let data: Data9 = bytes.pread_with(0, LE).unwrap();
    assert_eq!(data.0, 0xde);
    assert_eq!(data.1, 0x1010);
    let data: Data9 = bytes.cread_with(0, LE);
    assert_eq!(data.0, 0xde);
    assert_eq!(data.1, 0x1010);
    let size = Data9::size_with(&LE);
    let written = bytes.pwrite_with(&data, 0, LE).unwrap();
    assert_eq!(written, size);
}

#[derive(Debug, PartialEq, Eq, Pread, Pwrite, IOread, IOwrite, SizeWith)]
struct Data10I(u8, u16);

#[derive(Debug, Pwrite)]
struct Data10<'b> {
    pub inner: &'b Data10I,
    pub name: &'b [u8],
}

#[test]
fn test_reference() {
    let inner = Data10I(255, 1);
    let data = Data10 {
        inner: &inner,
        name: b"name",
    };
    let mut bytes = vec![0; 32];
    assert_eq!(bytes.pwrite_with(&data, 0, LE).unwrap(), 7);
    assert_eq!(bytes[..7], *b"\xff\x01\x00name");
}

#[derive(Debug, PartialEq, Pwrite, Pread, IOwrite, IOread, SizeWith)]
struct Data11 {
    pub a: u16,
    #[scroll(ctx = LE)]
    pub b: u16,
    #[scroll(ctx = BE)]
    pub c: u16,
}

#[test]
fn test_custom_ctx_derive() {
    let buf = [1, 2, 3, 4, 5, 6];
    let data = buf.pread_with(0, LE).unwrap();
    let data2 = Data11 {
        a: 0x0201,
        b: 0x0403,
        c: 0x0506,
    };
    assert_eq!(data, data2);
    let mut bytes = vec![0; 32];
    assert_eq!(bytes.pwrite_with::<&Data11>(&data, 0, LE).unwrap(), 6);
    assert_eq!(bytes[..Data11::size_with(&LE)], buf[..]);
    let mut bytes = std::io::Cursor::new(bytes);
    assert_eq!(data2, bytes.ioread_with(LE).unwrap());
    bytes.set_position(0);
    bytes.iowrite_with(data, BE).unwrap();
    bytes.set_position(0);
    assert_eq!(data2, bytes.ioread_with(BE).unwrap());
    bytes.set_position(0);
    let data3 = Data11 {
        a: 0x0102,
        b: 0x0403,
        c: 0x0506,
    };
    assert_eq!(data3, bytes.ioread_with(LE).unwrap());
}

#[derive(Debug, Pread, SizeWith)]
struct Data12 {
    ids: [Data11; 1],
}

#[test]
fn test_array_with_nested_pread_data() {
    let bytes = [0xde, 0xed, 0xef, 0x10, 0x10, 0x01];
    let data: Data12 = bytes.pread_with(0, LE).unwrap();
    assert_eq!(data.ids[0].a, 0xedde);
    assert_eq!(data.ids[0].b, 0x10ef);
    assert_eq!(data.ids[0].c, 0x1001);
}
