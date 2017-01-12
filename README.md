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

Because self is immutable, _all reads can be performed in parallel_ and hence are trivially parallelizable.

# Advanced Uses

Scroll is designed to be highly configurable - it allows you to implement various `Ctx` traits, which then grants the implementor _automatic_ uses of the `Pread`/`Gread` and/or `Pwrite`/`Gwrite` traits.

Please see the official documentation, or a simple [example](examples/data_ctx.rs) for more.

# Contributing

There are several open issues right now which I'd like clarified/closed before releasing on crates.io. Keep in mind, the primary use case is an immutable byte parser/reader, which `Pread` implements, and which I want backwards compability at this point.

In fact, if you look at the tests, most of them actually are just testing the APIs remain unbroken (still compiling), which is very easy to do with something this generic.

However, I believe there are some really interesting ideas to pursue, particularly in terms of the more generic contexts that scroll allows.

Any ideas, thoughts, or contributions are welcome!
