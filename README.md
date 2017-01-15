 [![Build Status](https://travis-ci.org/m4b/scroll.svg?branch=master)](https://travis-ci.org/m4b/scroll)
## Scroll - cast some magic

```
         _______________
    ()==(              (@==()
         '______________'|
           |             |
           |   ἀρετή     |
         __)_____________|
    ()==(               (@==()
         '--------------'

```

### Documentation

(not published on crates.io yet, sorry, very soon I promise)
https://docs.rs/scroll

### Usage

Add to your `Cargo.toml` (not on crates.io yet, sorry)

```toml
[dependencies]
scroll = "0.2.0"
```

### Overview

Scroll implements several traits for read/writing generic containers (byte buffers are currently implemented by default). Most familiar will likely be the `Pread` trait, which at its basic takes an immutable reference to self, an immutable offset to read at, (and a parsing context, more on that later), and then returns the deserialized value.

Because self is immutable, _all reads can be performed in parallel_ and hence are trivially parallelizable.

A simple example demonstrates its flexibility:

```rust
use scroll::Pread;
let bytes: [u8; 4] = [0xde, 0xad, 0xbe, 0xef];

// we can use the Buffer type that scroll provides, or use it on regular byte slices (or anything that impl's `AsRef<[u8]>`)
//let buffer = scroll::Buffer::new(bytes);
let b = &bytes[..];

// reads a u32 out of `b` with Big Endian byte order, at offset 0
let i: u32 = b.pread(0, scroll::BE).unwrap();
// or a u16 - specify the type either on the variable or with the beloved turbofish
let i2 = b.pread::<u16>(2, scroll::BE).unwrap();

// We can also skip the ctx by calling `pread_into`.
// for the primitive numbers, this will default to the host machine endianness (technically it is whatever default `Ctx` the target type is impl'd for)
let byte: u8 = b.pread_into(0).unwrap();
let i3: u32 = b.pread_into(0).unwrap();

// this will have the type `scroll::Error::BadOffset` because it tried to read beyond the bound
let byte: scroll::Result<i64> = b.pread_into(0);

// we can also get str and byte references from the underlying buffer/bytes using `pread_slice`
let slice = b.pread_slice::<str>(0, 2).unwrap();
let byte_slice: &[u8] = b.pread_slice(0, 2).unwrap();

// finally, we can also parse out custom datatypes if they implement the conversion trait `TryFromCtx`
let leb128_bytes: [u8; 5] = [0xde | 128, 0xad | 128, 0xbe | 128, 0xef | 128, 0x1];
// parses a uleb128 (variable length encoded integer) from the above bytes
let uleb128: u64 = leb128_bytes.pread::<scroll::Uleb128>(0, scroll::LEB128).unwrap().into();
assert_eq!(uleb128, 0x01def96deu64);
```

# Advanced Uses

Scroll is designed to be highly configurable - it allows you to implement various context (`Ctx`) sensitive traits, which then grants the implementor _automatic_ uses of the `Pread`/`Gread` and/or `Pwrite`/`Gwrite` traits.

For example, suppose we have a datatype and we want to specify how to parse or serialize this datatype out of some arbitrary
byte buffer. In order to do this, we need to provide a `TryFromCtx` impl for our datatype.

In particular, if we do this for the `[u8]` target, using the convention `(usize, YourCtx)`, you will automatically get access to
calling `pread::<YourDatatype>` on arrays of bytes.

```rust
use scroll::{self, ctx, Pread, BE};

struct Data<'a> {
  name: &'a str,
  id: u32,
}

// we could use a `(usize, endian::Scroll)` if we wanted
#[derive(Debug, Clone, Copy, Default)]
struct DataCtx { pub size: usize, pub endian: scroll::Endian }

// note the lifetime specified here
impl<'a> ctx::TryFromCtx<'a, (usize, DataCtx)> for Data<'a> {
  type Error = scroll::Error;
  // and the lifetime annotation on `&'a [u8]` here
  fn try_from_ctx (src: &'a [u8], (offset, DataCtx {size, endian}): (usize, DataCtx))
    -> Result<Self, Self::Error> {
    let name = src.pread_slice::<str>(offset, size)?;
    let id = src.pread(offset+size, endian)?;
    Ok(Data { name: name, id: id })
  }
}

let bytes = scroll::Buffer::new(b"UserName\x01\x02\x03\x04");
let data = bytes.pread::<Data>(0, DataCtx { size: 8, endian: BE }).unwrap();
assert_eq!(data.id, 0x01020304);
assert_eq!(data.name.to_string(), "UserName".to_string());
```

Please see the official documentation, or a simple [example](examples/data_ctx.rs) for more.

# Contributing

There are several open issues right now which I'd like clarified/closed before releasing on crates.io. Keep in mind, the primary use case is an immutable byte parser/reader, which `Pread` implements, and which I want backwards compability at this point.

In fact, if you look at the tests, most of them actually are just testing the APIs remain unbroken (still compiling), which is very easy to do with something this generic.

However, I believe there are some really interesting ideas to pursue, particularly in terms of the more generic contexts that scroll allows.

Any ideas, thoughts, or contributions are welcome!
