
// #[cfg(test)]
// mod tests {
//     #[allow(overflowing_literals)]
//     use super::{LE};

//     #[test]
//     fn test_measure_with_bytes() {
//         use super::ctx::MeasureWith;
//         let bytes: [u8; 4] = [0xef, 0xbe, 0xad, 0xde];
//         assert_eq!(bytes.measure_with(&()), 4);
//     }

//     #[test]
//     fn test_measurable() {
//         use super::ctx::SizeWith;
//         assert_eq!(8, u64::size_with(&LE));
//     }

//     //////////////////////////////////////////////////////////////
//     // begin pread_with
//     //////////////////////////////////////////////////////////////

//     macro_rules! pwrite_test {
//         ($write:ident, $read:ident, $deadbeef:expr) => {
//             #[test]
//             fn $write() {
//                 use super::{Pwrite, Pread, BE};
//                 let mut bytes: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
//                 let mut b = &mut bytes[..];
//                 b.pwrite_with::<$read>($deadbeef, 0, LE).unwrap();
//                 assert_eq!(b.pread_with::<$read>(0, LE).unwrap(), $deadbeef);
//                 b.pwrite_with::<$read>($deadbeef, 0, BE).unwrap();
//                 assert_eq!(b.pread_with::<$read>(0, BE).unwrap(), $deadbeef);
//             }
//         }
//     }

//     pwrite_test!(pwrite_and_pread_roundtrip_u16, u16, 0xbeef);
//     pwrite_test!(pwrite_and_pread_roundtrip_i16, i16, 0x7eef);
//     pwrite_test!(pwrite_and_pread_roundtrip_u32, u32, 0xbeefbeef);
//     pwrite_test!(pwrite_and_pread_roundtrip_i32, i32, 0x7eefbeef);
//     pwrite_test!(pwrite_and_pread_roundtrip_u64, u64, 0xbeefbeef7eef7eef);
//     pwrite_test!(pwrite_and_pread_roundtrip_i64, i64, 0x7eefbeef7eef7eef);

//     #[test]
//     fn pread_with_be() {
//         use super::{Pread};
//         let bytes: [u8; 2] = [0x7e, 0xef];
//         let b = &bytes[..];
//         let byte: u16 = b.pread_with(0, super::BE).unwrap();
//         assert_eq!(0x7eef, byte);
//         let bytes: [u8; 2] = [0xde, 0xad];
//         let dead: u16 = bytes.pread_with(0, super::BE).unwrap();
//         assert_eq!(0xdead, dead);
//     }

//     #[test]
//     fn pread() {
//         use super::{Pread};
//         let bytes: [u8; 2] = [0x7e, 0xef];
//         let b = &bytes[..];
//         let byte: u16 = b.pread(0).unwrap();
//         assert_eq!(0xef7e, byte);
//     }

//     #[test]
//     fn pread_slice() {
//         use super::{Pread};
//         use super::ctx::StrCtx;
//         let bytes: [u8; 2] = [0x7e, 0xef];
//         let b = &bytes[..];
//         let iserr: Result<&str, _>  = b.pread_with(0, StrCtx::Length(3));
//         assert!(iserr.is_err());
//         // let bytes2: &[u8]  = b.pread_with(0, 2).unwrap();
//         // assert_eq!(bytes2.len(), bytes[..].len());
//         // for i in 0..bytes2.len() {
//         //     assert_eq!(bytes2[i], bytes[i])
//         // }
//     }

//     #[test]
//     fn pread_str() {
//         use super::Pread;
//         use super::ctx::*;
//         let bytes: [u8; 2] = [0x2e, 0x0];
//         let b = &bytes[..];
//         let s: &str  = b.pread(0).unwrap();
//         println!("str: {}", s);
//         assert_eq!(s.len(), bytes[..].len() - 1);
//         let bytes: &[u8] = b"hello, world!\0some_other_things";
//         let hello_world: &str = bytes.pread_with(0, StrCtx::Delimiter(NULL)).unwrap();
//         println!("{:?}", &hello_world);
//         assert_eq!(hello_world.len(), 13);
//         let hello: &str = bytes.pread_with(0, StrCtx::Delimiter(SPACE)).unwrap();
//         println!("{:?}", &hello);
//         assert_eq!(hello.len(), 6);
//         // this could result in underflow so we just try it
//         let _error = bytes.pread_with::<&str>(6, StrCtx::Delimiter(SPACE));
//         let error = bytes.pread_with::<&str>(7, StrCtx::Delimiter(SPACE));
//         println!("{:?}", &error);
//         assert!(error.is_ok());
//     }

//     #[test]
//     fn pread_str_weird() {
//         use super::Pread;
//         use super::ctx::*;
//         let bytes: &[u8] = b"";
//         let hello_world = bytes.pread_with::<&str>(0, StrCtx::Delimiter(NULL));
//         println!("1 {:?}", &hello_world);
//         assert_eq!(hello_world.is_err(), true);
//         let error = bytes.pread_with::<&str>(7, StrCtx::Delimiter(SPACE));
//         println!("2 {:?}", &error);
//         assert!(error.is_err());
//         let bytes: &[u8] = b"\0";
//         let null  = bytes.pread::<&str>(0).unwrap();
//         println!("3 {:?}", &null);
//         assert_eq!(null.len(), 0);
//     }

//     #[test]
//     fn pwrite_str_and_bytes() {
//         use super::{Pread, Pwrite};
//         use super::ctx::*;
//         let astring: &str = "lol hello_world lal\0ala imabytes";
//         let mut buffer = [0u8; 33];
//         buffer.pwrite(astring, 0).unwrap();
//         {
//             let hello_world = buffer.pread_with::<&str>(4, StrCtx::Delimiter(SPACE)).unwrap();
//             assert_eq!(hello_world, "hello_world");
//         }
//         let bytes: &[u8] = b"more\0bytes";
//         buffer.pwrite(bytes, 0).unwrap();
//         let more = bytes.pread_with::<&str>(0, StrCtx::Delimiter(NULL)).unwrap();
//         assert_eq!(more, "more");
//         let bytes = bytes.pread_with::<&str>(more.len() + 1, StrCtx::Delimiter(NULL)).unwrap();
//         assert_eq!(bytes, "bytes");
//     }

//     use std::error;
//     use std::fmt::{self, Display};

//     #[derive(Debug)]
//     pub struct ExternalError {}

//     impl Display for ExternalError {
//         fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
//             write!(fmt, "ExternalError")
//         }
//     }

//     impl error::Error for ExternalError {
//         fn description(&self) -> &str {
//             "ExternalError"
//         }
//         fn cause(&self) -> Option<&error::Error> { None}
//     }

//     impl From<super::Error> for ExternalError {
//         fn from(err: super::Error) -> Self {
//             //use super::Error::*;
//             match err {
//                 _ => ExternalError{},
//             }
//         }
//     }

//     #[derive(Debug, PartialEq, Eq)]
//     pub struct Foo(u16);

//     impl super::ctx::TryIntoCtx<super::Endian> for Foo {

//         type Error = ExternalError;
//         fn try_into_ctx(self, this: &mut [u8], le: super::Endian) -> Result<(), Self::Error> {
//             use super::Pwrite;
//             if this.len() < 2 { return Err((ExternalError {}).into()) }
//             this.pwrite_with(self.0, 0, le)?;
//             Ok(())
//         }
//     }

//     impl<'a> super::ctx::TryFromCtx<'a, super::Endian> for Foo {
//         type Error = ExternalError;
//         fn try_from_ctx(this: &'a [u8], le: super::Endian) -> Result<Self, Self::Error> {
//             use super::Pread;
//             if this.len() > 2 { return Err((ExternalError {}).into()) }
//             let n = this.pread_with(0, le)?;
//             Ok(Foo(n))
//         }
//     }

//     #[test]
//     fn pread_with_iter_bytes() {
//         use super::{Pread};
//         let mut bytes_to: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
//         let bytes_from: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
//         let mut bytes_to = &mut bytes_to[..];
//         let bytes_from = &bytes_from[..];
//         for i in 0..bytes_from.len() {
//             bytes_to[i] = bytes_from.pread(i).unwrap();
//         }
//         assert_eq!(bytes_to, bytes_from);
//     }

//     //////////////////////////////////////////////////////////////
//     // end pread_with
//     //////////////////////////////////////////////////////////////

//     //////////////////////////////////////////////////////////////
//     // begin gread_with
//     //////////////////////////////////////////////////////////////
//     macro_rules! g_test {
//         ($read:ident, $deadbeef:expr, $typ:ty) => {
//             #[test]
//             fn $read() {
//                 use super::Gread;
//                 let bytes: [u8; 8] = [0xf, 0xe, 0xe, 0xb, 0xd, 0xa, 0xe, 0xd];
//                 let mut offset = 0;
//                 let deadbeef: $typ = bytes.gread_with(&mut offset, LE).unwrap();
//                 assert_eq!(deadbeef, $deadbeef as $typ);
//                 assert_eq!(offset, ::std::mem::size_of::<$typ>());
//             }
//         }
//     }

//     g_test!(simple_gread_u16, 0xe0f, u16);
//     g_test!(simple_gread_u32, 0xb0e0e0f, u32);
//     g_test!(simple_gread_u64, 0xd0e0a0d0b0e0e0f, u64);
//     g_test!(simple_gread_i64, 940700423303335439, i64);

//     macro_rules! simple_float_test {
//         ($read:ident, $deadbeef:expr, $typ:ty) => {
//             #[test]
//             fn $read() {
//                 use super::Gread;
//                 let bytes: [u8; 8] = [0u8, 0, 0, 0, 0, 0, 224, 63];
//                 let mut offset = 0;
//                 let deadbeef: $typ = bytes.gread_with(&mut offset, LE).unwrap();
//                 assert_eq!(deadbeef, $deadbeef as $typ);
//                 assert_eq!(offset, ::std::mem::size_of::<$typ>());
//             }
//         };
//     }

//     simple_float_test!(gread_f32, 0.0, f32);
//     simple_float_test!(gread_f64, 0.5, f64);

//     macro_rules! g_read_write_test {
//         ($read:ident, $val:expr, $typ:ty) => {
//             #[test]
//             fn $read() {
//                 use super::{LE, BE, Gread, Gwrite};
//                 let mut buffer = [0u8; 16];
//                 let mut offset = &mut 0;
//                 buffer.gwrite_with($val.clone(), offset, LE).unwrap();
//                 let mut o2 = &mut 0;
//                 let val: $typ = buffer.gread_with(o2, LE).unwrap();
//                 assert_eq!(val, $val);
//                 assert_eq!(*offset, ::std::mem::size_of::<$typ>());
//                 assert_eq!(*o2, ::std::mem::size_of::<$typ>());
//                 assert_eq!(*o2, *offset);
//                 buffer.gwrite_with($val.clone(), offset, BE).unwrap();
//                 let val: $typ = buffer.gread_with(o2, BE).unwrap();
//                 assert_eq!(val, $val);
//             }
//         };
//     }

//     g_read_write_test!(gread_gwrite_f64_1, 0.25f64, f64);
//     g_read_write_test!(gread_gwrite_f64_2, 0.5f64, f64);
//     g_read_write_test!(gread_gwrite_f64_3, 0.064, f64);

//     g_read_write_test!(gread_gwrite_f32_1, 0.25f32, f32);
//     g_read_write_test!(gread_gwrite_f32_2, 0.5f32, f32);
//     g_read_write_test!(gread_gwrite_f32_3, 0.0f32, f32);

//     g_read_write_test!(gread_gwrite_i64_1, 0i64, i64);
//     g_read_write_test!(gread_gwrite_i64_2, -1213213211111i64, i64);
//     g_read_write_test!(gread_gwrite_i64_3, -3000i64, i64);

//     g_read_write_test!(gread_gwrite_i32_1, 0i32, i32);
//     g_read_write_test!(gread_gwrite_i32_2, -1213213232, i32);
//     g_read_write_test!(gread_gwrite_i32_3, -3000i32, i32);

//     // useful for ferreting out problems with impls
//     #[test]
//     fn gread_with_iter_bytes() {
//         use super::{Gread};
//         let mut bytes_to: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
//         let bytes_from: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
//         let mut bytes_to = &mut bytes_to[..];
//         let bytes_from = &bytes_from[..];
//         let mut offset = &mut 0;
//         for i in 0..bytes_from.len() {
//             bytes_to[i] = bytes_from.gread(&mut offset).unwrap();
//         }
//         assert_eq!(bytes_to, bytes_from);
//         assert_eq!(*offset, bytes_to.len());
//     }

//     #[test]
//     fn gread_inout() {
//         use super::{Gread};
//         let mut bytes_to: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
//         let bytes_from: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
//         let bytes = &bytes_from[..];
//         let mut offset = &mut 0;
//         bytes.gread_inout(offset, &mut bytes_to[..]).unwrap();
//         assert_eq!(bytes_to, bytes_from);
//         assert_eq!(*offset, bytes_to.len());
//     }

//     #[test]
//     fn gread_with_byte() {
//         use super::{Gread};
//         let bytes: [u8; 1] = [0x7f];
//         let b = &bytes[..];
//         let mut offset = &mut 0;
//         let byte: u8 = b.gread(offset).unwrap();
//         assert_eq!(0x7f, byte);
//         assert_eq!(*offset, 1);
//     }

/*
    #[test]
    fn gread_slice() {
        use super::{Gread, ctx};
        let bytes: [u8; 2] = [0x7e, 0xef];
        let b = &bytes[..];
        let mut offset = &mut 0;
        let res = b.gread_with::<&str>(offset, StrCtx::Length(3));
        assert!(res.is_err());
        *offset = 0;
        let astring: [u8; 3] = [0x45, 042, 0x44];
        let string = astring.gread_slice::<str>(offset, 2);
        match &string {
            &Ok(_) => {},
            &Err(ref err) => {println!("{}", &err); panic!();}
        }
        assert_eq!(string.unwrap(), "E*");
        *offset = 0;
        let bytes2: &[u8]  = b.gread_slice(offset, 2).unwrap();
        assert_eq!(*offset, 2);
        assert_eq!(bytes2.len(), bytes[..].len());
        for i in 0..bytes2.len() {
            assert_eq!(bytes2[i], bytes[i])
        }
    }
    */
/////////////////////////////////////////////////////////////////
// end gread_with
/////////////////////////////////////////////////////////////////
// }
