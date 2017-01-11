# Scroll - cast some magic

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

[Documentation](https://docs.rs/scroll)

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
// this will default to host machine endianness (technically it is whatever default `Ctx` the target type is impl'd for)
let byte: u8 = b.pread_into(0).unwrap();
// this will have the type `scroll::Error::BadOffset` because it tried to read beyond the bound
let byte: scroll::Result<i64> = b.pread_into(0);
let slice = b.pread_slice::<str>(0, 2).unwrap();
let byte_slice: &[u8] = b.pread_slice(0, 2).unwrap();
let leb128_bytes: [u8; 5] = [0xde | 128, 0xad | 128, 0xbe | 128, 0xef | 128, 0x1];
// parses a uleb128 (variable length encoded integer) from the above bytes
let uleb128 = leb128_bytes.pread::<u64>(0, scroll::LEB128).unwrap();
assert_eq!(uleb128, 0x01def96deu64);
// can't currently default read a u64 because it can't differentiate the context, e.g., whether it should be parsed as a `scroll::LEB128` or `scroll::LE` or `scroll::BE`
let i = leb128_bytes.pread_into::<u32>(0).unwrap();
assert_eq!(i, 4022250974u32);
```

Because self is immutable, _all reads can be performed in parallel_ and hence are trivially parallelizable.

# Advanced Uses

Scroll is designed to be highly configurable - it allows you to implement various `Ctx` traits, which then grants the implementor _automatic_ uses of the `Pread`/`Gread` and/or `Pwrite`/`Gwrite` traits.

Please see the official documentation for more.