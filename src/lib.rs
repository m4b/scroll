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
//!
//! There are 3 traits for reading that you can import:
//!
//! 1. [Pread](trait.Pread.html), for reading (immutable) data at an offset;
//! 2. [Gread](trait.Gread.html), for reading data at an offset which automatically gets incremented by the size;
//! 3. [Lread](trait.Lread.html), for reading data out of a `std::io::Read` based interface, e.g., a stream. (**Note**: only available when compiled with `std`)
//!
//! Each of these interfaces also have their corresponding writer versions as well, e.g., [Pwrite](trait.Pwrite.html), [Gwrite](trait.Gwrite.html), and [Lwrite](trait.Lwrite.html), respectively.
//!
//! Most familiar will likely be the `Pread` trait (inspired from the C function), which in our case takes an immutable reference to self, an immutable offset to read at, (and _optionally_ a parsing context, more on that later), and then returns the deserialized value.
//!
//! Because self is immutable, _**all** reads can be performed in parallel_ and hence are trivially parallelizable.
//!
//! # Example
//!
//! A simple example demonstrates its flexibility:
//!
//! ```rust
//! use scroll::{ctx, Pread, LE};
//! let bytes: [u8; 4] = [0xde, 0xad, 0xbe, 0xef];
//!
//! // reads a u32 out of `b` with the endianness of the host machine, at offset 0, turbofish-style
//! let number: u32 = bytes.pread::<u32>(0).unwrap();
//! // ...or a byte, with type ascription on the binding.
//! let byte: u8 = bytes.pread(0).unwrap();
//!
//! //If the type is known another way by the compiler, say reading into a struct field, we can omit the turbofish, and type ascription altogether!
//!
//! // If we want, we can explicitly add a endianness to read with by calling `pread_with`.
//! // The following reads a u32 out of `b` with Big Endian byte order, at offset 0
//! let be_number: u32 = bytes.pread_with(0, scroll::BE).unwrap();
//! // or a u16 - specify the type either on the variable or with the beloved turbofish
//! let be_number2 = bytes.pread_with::<u16>(2, scroll::BE).unwrap();
//!
//! // Scroll has core friendly errors (no allocation). This will have the type `scroll::Error::BadOffset` because it tried to read beyond the bound
//! let byte: scroll::Result<i64> = bytes.pread(0);
//!
//! //If you know the operation can't fail, you can also use the `pread_unsafe` api:
//! let byte: u8 = bytes.pread_unsafe(0, LE);
//!
//! // Here is an example of parsing a variable length uleb128 custom datatype
//!
//! let leb128_bytes: [u8; 5] = [0xde | 128, 0xad | 128, 0xbe | 128, 0xef | 128, 0x1];
//! // parses a uleb128 (variable length encoded integer) from the above bytes
//! let uleb128: u64 = leb128_bytes.pread::<scroll::Uleb128>(0).unwrap().into();
//! assert_eq!(uleb128, 0x01def96deu64);
//!
//! // Scroll is extensible: as long as the type implements `TryWithCtx`, then you can read your type out of the byte array!
//!
//! // We can parse out custom datatypes, or types with lifetimes
//! // if they implement the conversion trait `TryFromCtx`; here we parse a C-style \0 delimited &str (safely)
//! let hello: &[u8] = b"hello_world\0more words";
//! let hello_world: &str = hello.pread(0).unwrap();
//! assert_eq!("hello_world", hello_world);
//!
//! // ... and this parses the string if its space separated!
//! let spaces: &[u8] = b"hello world some junk";
//! let world: &str = spaces.pread_with(6, ctx::SPACE).unwrap();
//! assert_eq!("world", world);
//!
//! ```
//!
//! # `std::io` API
//!
//! Scroll can also read/write simple types from a `std::io::Read` or `std::io::Write` implementor. The  built-in numeric types are taken care of for you.  If you want to read a custom type, you need to implement the [FromCtx](trait.FromCtx.html) (_how_ to parse) and [SizeWith](ctx/trait.SizeWith.html) (_how_ big the parsed thing will be) traits.  You must compile with default features. For example:
//!
//! ```rust
//! use std::io::Cursor;
//! use scroll::Lread;
//! let bytes_ = [0x01,0x00,0x00,0x00,0x00,0x00,0x00,0x00, 0xef,0xbe,0x00,0x00,];
//! let mut bytes = Cursor::new(bytes_);
//!
//! // this will bump the cursor's Seek
//! let foo = bytes.lread::<usize>().unwrap();
//! // ..ditto
//! let bar = bytes.lread::<u32>().unwrap();
//! ```
//!
//! Similarly, we can write to anything that implements `std::io::Write` quite naturally:
//!
//! ```rust
//! use scroll::{Lwrite, LE, BE};
//! use std::io::{Write, Cursor};
//!
//! let mut bytes = [0x0u8; 10];
//! let mut cursor = Cursor::new(&mut bytes[..]);
//! cursor.write_all(b"hello").unwrap();
//! cursor.lwrite_with(0xdeadbeef as u32, BE).unwrap();
//! assert_eq!(cursor.into_inner(), [0x68, 0x65, 0x6c, 0x6c, 0x6f, 0xde, 0xad, 0xbe, 0xef, 0x0]);
//! ```
//!
//! # Advanced Uses
//!
//! Scroll is designed to be highly configurable - it allows you to implement various context (`Ctx`) sensitive traits, which then grants the implementor _automatic_ uses of the `Pread`/`Gread` and/or `Pwrite`/`Gwrite` traits.
//!
//! For example, suppose we have a datatype and we want to specify how to parse or serialize this datatype out of some arbitrary
//! byte buffer. In order to do this, we need to provide a [TryFromCtx](trait.TryFromCtx.html) impl for our datatype.
//! 
//! In particular, if we do this for the `[u8]` target, using the convention `(usize, YourCtx)`, you will automatically get access to
//! calling `pread_with::<YourDatatype>` on arrays of bytes.
//! 
//! ```rust
//! use scroll::{self, ctx, Pread, BE};
//! 
//! struct Data<'a> {
//!   name: &'a str,
//!   id: u32,
//! }
//! 
//! // we could use a `(usize, endian::Scroll)` if we wanted
//! #[derive(Debug, Clone, Copy, Default)]
//! struct DataCtx { pub size: usize, pub endian: scroll::Endian }
//! 
//! // note the lifetime specified here
//! impl<'a> ctx::TryFromCtx<'a, (usize, DataCtx)> for Data<'a> {
//!   type Error = scroll::Error;
//!   // and the lifetime annotation on `&'a [u8]` here
//!   fn try_from_ctx (src: &'a [u8], (offset, DataCtx {size, endian}): (usize, DataCtx))
//!     -> Result<Self, Self::Error> {
//!     let name = src.pread_slice::<str>(offset, size)?;
//!     let id = src.pread_with(offset+size, endian)?;
//!     Ok(Data { name: name, id: id })
//!   }
//! }
//! 
//! let bytes = b"UserName\x01\x02\x03\x04";
//! let data = bytes.pread_with::<Data>(0, DataCtx { size: 8, endian: BE }).unwrap();
//! assert_eq!(data.id, 0x01020304);
//! assert_eq!(data.name.to_string(), "UserName".to_string());
//! ```
//!
//! Please see the [Pread documentation examples](trait.Pread.html#implementing-your-own-reader)

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate core;

pub mod ctx;
mod pread;
mod pwrite;
mod greater;
mod error;
mod endian;
mod leb128;
#[cfg(feature = "std")]
mod lesser;

pub use endian::*;
pub use pread::*;
pub use pwrite::*;
pub use greater::*;
pub use error::*;
pub use leb128::*;
#[cfg(feature = "std")]
pub use lesser::*;

#[cfg(test)]
mod tests {
    #[allow(overflowing_literals)]
    use super::{LE};

    // cursor needs to implement AsRef<[u8]>
    // #[test]
    // fn test_measurable_on_cursor() {
    //     use std::io::Cursor;
    //     use super::Measure;
    //     let bytes: [u8; 4] = [0xef, 0xbe, 0xad, 0xde];
    //     let cursor = Cursor::new(bytes);
    //     assert_eq!(cursor.measure(), 4);
    // }

    #[test]
    fn test_measurable() {
        use super::ctx::{CTX, SizeWith};
        assert_eq!(8, u64::size_with(&CTX));
    }

    //////////////////////////////////////////////////////////////
    // begin pread_with
    //////////////////////////////////////////////////////////////

    macro_rules! pwrite_test {
        ($write:ident, $read:ident, $deadbeef:expr) => {
            #[test]
            fn $write() {
                use super::{Pwrite, Pread, BE};
                let mut bytes: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
                let mut b = &mut bytes[..];
                b.pwrite_with::<$read>($deadbeef, 0, LE).unwrap();
                assert_eq!(b.pread_with::<$read>(0, LE).unwrap(), $deadbeef);
                b.pwrite_with::<$read>($deadbeef, 0, BE).unwrap();
                assert_eq!(b.pread_with::<$read>(0, BE).unwrap(), $deadbeef);
            }
        }
    }

    pwrite_test!(p_u16, u16, 0xbeef);
    pwrite_test!(p_i16, i16, 0x7eef);
    pwrite_test!(p_u32, u32, 0xbeefbeef);
    pwrite_test!(p_i32, i32, 0x7eefbeef);
    pwrite_test!(p_u64, u64, 0xbeefbeef7eef7eef);
    pwrite_test!(p_i64, i64, 0x7eefbeef7eef7eef);

    #[test]
    fn pread_with_be() {
        use super::{Pread};
        let bytes: [u8; 2] = [0x7e, 0xef];
        let b = &bytes[..];
        let byte: u16 = <[u8] as Pread>::pread_with(b, 0, super::BE).unwrap();
        assert_eq!(0x7eef, byte);
        let bytes: [u8; 2] = [0xde, 0xad];
        let dead: u16 = bytes.pread_with(0, super::BE).unwrap();
        assert_eq!(0xdead, dead);
    }

    #[test]
    fn pread() {
        use super::{Pread};
        let bytes: [u8; 2] = [0x7e, 0xef];
        let b = &bytes[..];
        let byte: u16 = b.pread(0).unwrap();
        assert_eq!(0xef7e, byte);
    }

    #[test]
    fn pread_slice() {
        use super::{Pread};
        let bytes: [u8; 2] = [0x7e, 0xef];
        let b = &bytes[..];
        let _bytes2: Result<&str, _>  = b.pread_slice::<str>(0, 2);
        let bytes2: &[u8]  = b.pread_slice(0, 2).unwrap();
        //let bytes3: &[u8; 2]  = b.pread_slice(0, 2).unwrap();
        assert_eq!(bytes2.len(), bytes[..].len());
        for i in 0..bytes2.len() {
            assert_eq!(bytes2[i], bytes[i])
        }
    }

    #[test]
    fn pread_str() {
        use super::Pread;
        use super::ctx::{NULL, SPACE};
        let bytes: [u8; 2] = [0x2e, 0x0];
        let b = &bytes[..];
        let s: &str  = b.pread(0).unwrap();
        println!("str: {}", s);
        assert_eq!(s.len(), bytes[..].len() - 1);
        let bytes: &[u8] = b"hello, world!\0some_other_things";
        let hello_world: &str = bytes.pread_with(0, NULL).unwrap();
        println!("{:?}", &hello_world);
        assert_eq!(hello_world.len(), 13);
        let hello: &str = bytes.pread_with(0, SPACE).unwrap();
        println!("{:?}", &hello);
        assert_eq!(hello.len(), 6);
        // this could result in underflow so we just try it
        let _error = bytes.pread_with::<&str>(6, SPACE);
        let error = bytes.pread_with::<&str>(7, SPACE);
        println!("{:?}", &error);
        assert!(error.is_ok());
    }

    #[test]
    fn pread_str_weird() {
        use super::Pread;
        use super::ctx::{NULL, SPACE};
        let bytes: &[u8] = b"";
        let hello_world  = bytes.pread_with::<&str>(0, NULL);
        println!("{:?}", &hello_world);
        assert_eq!(hello_world.is_err(), true);
        let error = bytes.pread_with::<&str>(7, SPACE);
        println!("{:?}", &error);
        assert!(error.is_err());
        let bytes: &[u8] = b"\0";
        let null  = bytes.pread::<&str>(0).unwrap();
        println!("{:?}", &null);
        assert_eq!(null.len(), 0);
    }

    #[test]
    fn pwrite_str_and_bytes() {
        use super::{Pread, Pwrite};
        use super::ctx::{self, SPACE};
        let astring: &str = "lol hello_world lal\0ala imabytes";
        let mut buffer = [0u8; 33];
        buffer.pwrite(astring, 0).unwrap();
        {
            let hello_world = buffer.pread_with::<&str>(4, SPACE).unwrap();
            assert_eq!(hello_world, "hello_world");
        }
        let bytes: &[u8] = b"more\0bytes";
        buffer.pwrite(bytes, 0).unwrap();
        let bytes2 = bytes.pread_slice::<[u8]>(0, bytes.len()).unwrap();
        assert_eq!(bytes2.len(), bytes.len());
        let more = bytes.pread_with::<&str>(0, ctx::NULL).unwrap();
        assert_eq!(more, "more");
        let bytes = bytes.pread_with::<&str>(more.len() + 1, ctx::NULL).unwrap();
        assert_eq!(bytes, "bytes");
    }

    use std::error;
    use std::fmt::{self, Display};

    #[derive(Debug)]
    pub struct ExternalError {}

    impl Display for ExternalError {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            write!(fmt, "ExternalError")
        }
    }

    impl error::Error for ExternalError {
        fn description(&self) -> &str {
            "ExternalError"
        }
        fn cause(&self) -> Option<&error::Error> { None}
    }

    impl From<super::Error> for ExternalError {
        fn from(err: super::Error) -> Self {
            //use super::Error::*;
            match err {
                _ => ExternalError{},
            }
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    pub struct Foo(u16);

    impl super::ctx::TryIntoCtx for Foo {
        type Error = ExternalError;
        fn try_into_ctx(self, this: &mut [u8], ctx: (usize, super::Endian)) -> Result<(), Self::Error> {
            use super::Pwrite;
            let offset = ctx.0;
            let le = ctx.1;
            if offset > 2 { return Err((ExternalError {}).into()) }
            this.pwrite_with(self.0, offset, le)?;
            Ok(())
        }
    }

    impl<'a> super::ctx::TryFromCtx<'a> for Foo {
        type Error = ExternalError;
        fn try_from_ctx(this: &'a [u8], ctx: (usize, super::Endian)) -> Result<Self, Self::Error> {
            use super::Pread;
            let offset = ctx.0;
            let le = ctx.1;
            if offset > 2 { return Err((ExternalError {}).into()) }
            let n = this.pread_with(offset, le)?;
            Ok(Foo(n))
        }
    }

    #[test]
    fn pread_with_iter_bytes() {
        use super::{Pread};
        let mut bytes_to: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
        let bytes_from: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
        let mut bytes_to = &mut bytes_to[..];
        let bytes_from = &bytes_from[..];
        for i in 0..bytes_from.len() {
            bytes_to[i] = bytes_from.pread(i).unwrap();
        }
        assert_eq!(bytes_to, bytes_from);
    }

    //////////////////////////////////////////////////////////////
    // end pread_with
    //////////////////////////////////////////////////////////////

    //////////////////////////////////////////////////////////////
    // begin gread_with
    //////////////////////////////////////////////////////////////

    macro_rules! g_test {
        ($read:ident, $deadbeef:expr, $typ:ty) => {
            #[test]
            fn $read() {
                use super::Gread;
                let bytes: [u8; 8] = [0xf, 0xe, 0xe, 0xb, 0xd, 0xa, 0xe, 0xd];
                let mut offset = 0;
                let deadbeef: $typ = bytes.gread_with(&mut offset, LE).unwrap();
                assert_eq!(deadbeef, $deadbeef as $typ);
                assert_eq!(offset, ::std::mem::size_of::<$typ>());
            }
        }
    }

    g_test!(simple_gread_u16, 0xe0f, u16);
    g_test!(simple_gread_u32, 0xb0e0e0f, u32);
    g_test!(simple_gread_u64, 0xd0e0a0d0b0e0e0f, u64);
    g_test!(simple_gread_i64, 940700423303335439, i64);

    macro_rules! simple_float_test {
        ($read:ident, $deadbeef:expr, $typ:ty) => {
            #[test]
            fn $read() {
                use super::Gread;
                let bytes: [u8; 8] = [0u8, 0, 0, 0, 0, 0, 224, 63];
                let mut offset = 0;
                let deadbeef: $typ = bytes.gread_with(&mut offset, LE).unwrap();
                assert_eq!(deadbeef, $deadbeef as $typ);
                assert_eq!(offset, ::std::mem::size_of::<$typ>());
            }
        };
    }

    simple_float_test!(gread_f32, 0.0, f32);
    simple_float_test!(gread_f64, 0.5, f64);

    macro_rules! g_read_write_test {
        ($read:ident, $val:expr, $typ:ty) => {
            #[test]
            fn $read() {
                use super::{LE, BE, Gread, Gwrite};
                let mut buffer = [0u8; 16];
                let mut offset = &mut 0;
                buffer.gwrite_with($val.clone(), offset, LE).unwrap();
                let mut o2 = &mut 0;
                let val: $typ = buffer.gread_with(o2, LE).unwrap();
                assert_eq!(val, $val);
                assert_eq!(*offset, ::std::mem::size_of::<$typ>());
                assert_eq!(*o2, ::std::mem::size_of::<$typ>());
                assert_eq!(*o2, *offset);
                buffer.gwrite_with($val.clone(), offset, BE).unwrap();
                let val: $typ = buffer.gread_with(o2, BE).unwrap();
                assert_eq!(val, $val);
            }
        };
    }

    g_read_write_test!(gread_gwrite_f64_1, 0.25f64, f64);
    g_read_write_test!(gread_gwrite_f64_2, 0.5f64, f64);
    g_read_write_test!(gread_gwrite_f64_3, 0.064, f64);

    g_read_write_test!(gread_gwrite_f32_1, 0.25f32, f32);
    g_read_write_test!(gread_gwrite_f32_2, 0.5f32, f32);
    g_read_write_test!(gread_gwrite_f32_3, 0.0f32, f32);

    g_read_write_test!(gread_gwrite_i64_1, 0i64, i64);
    g_read_write_test!(gread_gwrite_i64_2, -1213213211111i64, i64);
    g_read_write_test!(gread_gwrite_i64_3, -3000i64, i64);

    g_read_write_test!(gread_gwrite_i32_1, 0i32, i32);
    g_read_write_test!(gread_gwrite_i32_2, -1213213232, i32);
    g_read_write_test!(gread_gwrite_i32_3, -3000i32, i32);

    // useful for ferreting out problems with impls
    #[test]
    fn gread_with_iter_bytes() {
        use super::{Gread};
        let mut bytes_to: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
        let bytes_from: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
        let mut bytes_to = &mut bytes_to[..];
        let bytes_from = &bytes_from[..];
        let mut offset = &mut 0;
        for i in 0..bytes_from.len() {
            bytes_to[i] = bytes_from.gread(&mut offset).unwrap();
        }
        assert_eq!(bytes_to, bytes_from);
        assert_eq!(*offset, bytes_to.len());
    }

    #[test]
    fn gread_inout() {
        use super::{Gread};
        let mut bytes_to: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
        let bytes_from: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
        let bytes = &bytes_from[..];
        let mut offset = &mut 0;
        bytes.gread_inout(offset, &mut bytes_to[..]).unwrap();
        assert_eq!(bytes_to, bytes_from);
        assert_eq!(*offset, bytes_to.len());
    }

    #[test]
    fn gread_with_byte() {
        use super::{Gread};
        let bytes: [u8; 1] = [0x7f];
        let b = &bytes[..];
        let mut offset = &mut 0;
        let byte: u8 = b.gread(offset).unwrap();
        assert_eq!(0x7f, byte);
        assert_eq!(*offset, 1);
    }

    #[test]
    fn gread_slice() {
        use super::{Gread};
        let bytes: [u8; 2] = [0x7e, 0xef];
        let b = &bytes[..];
        let mut offset = &mut 0;
        let res = b.gread_slice::<str>(offset, 3);
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

    /////////////////////////////////////////////////////////////////
    // end gread_with
    /////////////////////////////////////////////////////////////////
}
