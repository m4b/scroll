use core::result;
use core::fmt::Debug;

use ctx::{TryFromCtx, TryRefFromCtx};
use error;
use endian::Endian;

//type TryCtx<I, Ctx> = (I, Ctx);
//type SliceCtx<I, Ctx> = (I, I, Ctx);

/// A very generic, contextual pread interface in Rust. Allows completely parallelized reads, as `Self` is immutable
/// Don't be scared! The `Pread` definition _is_ terrifying, but it is definitely tractable. Essentially, `E` is the error, `Ctx` the parsing context, `I` is the indexing type, `TryCtx` is the "offset + ctx" Context given to the `TryFromCtx` trait bounds, and `SliceCtx` is the "offset + size + ctx" context given to the `TryRefFromCtx` trait bound.
///
/// They all have reasonable defaults, so if you're just using this trait, you can usually get away with `fn <T: scroll::Pread>(bytes: &T) -> yourResultWhichConvertsScrollErrors`
///
/// # Implementing Your Own Reader
/// If you want to implement your own reader for a type `Foo` from some kind of buffer (say `[u8]`), then you need to implement `TryFromCtx`
///
/// ```rust
/// use scroll::{self, TryFromCtx};
///  #[derive(Debug, PartialEq, Eq)]
///  pub struct Foo(u16);
///
///  impl TryFromCtx for Foo {
///      type Error = scroll::Error;
///      fn try_from_ctx(this: &[u8], ctx: (usize, scroll::Endian)) -> Result<Self, Self::Error> {
///          use scroll::Pread;
///          let offset = ctx.0;
///          let le = ctx.1;
///          if offset > 2 { return Err((scroll::Error::BadOffset("whatever".to_string())).into()) }
///          let n = this.pread(offset, le)?;
///          Ok(Foo(n))
///      }
///  }
///
/// use scroll::Pread;
/// // you can now read `Foo`'s out of a &[u8] buffer for free, with `Pread` and `Gread` without doing _anything_ else!
/// let bytes: [u8; 4] = [0xde, 0xad, 0, 0];
/// let foo = bytes.pread_into::<Foo>(0).unwrap();
/// assert_eq!(Foo(0xadde), foo);
/// let foo2 = bytes.pread::<Foo>(0, scroll::BE).unwrap();
/// assert_eq!(Foo(0xdeadu16), foo2);
/// ```
///
/// # Advanced: Using Your Own Error in `TryFromCtx`
/// ```rust
///  use scroll::{self, TryFromCtx};
///  use std::error;
///  use std::fmt::{self, Display};
///  // make some kind of normal error which also can transform a scroll error ideally (quick_error, error_chain allow this automatically nowadays)
///  #[derive(Debug)]
///  pub struct ExternalError {}
///
///  impl Display for ExternalError {
///      fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
///          write!(fmt, "ExternalError")
///      }
///  }
///
///  impl error::Error for ExternalError {
///      fn description(&self) -> &str {
///          "ExternalError"
///      }
///      fn cause(&self) -> Option<&error::Error> { None}
///  }
///
///  impl From<scroll::Error> for ExternalError {
///      fn from(err: scroll::Error) -> Self {
///          match err {
///              _ => ExternalError{},
///          }
///      }
///  }
///  #[derive(Debug, PartialEq, Eq)]
///  pub struct Foo(u16);
///
///  impl TryFromCtx for Foo {
///      type Error = ExternalError;
///      fn try_from_ctx(this: &[u8], ctx: (usize, scroll::Endian)) -> Result<Self, Self::Error> {
///          use scroll::Pread;
///          let offset = ctx.0;
///          let le = ctx.1;
///          if offset > 2 { return Err((ExternalError {}).into()) }
///          let n = this.pread(offset, le)?;
///          Ok(Foo(n))
///      }
///  }
///
/// use scroll::Pread;
/// // the only caveat is that you now need to specify the error type in fn generic params, e.g. `fn thingee<S: scroll::Pread<ExternalError>>`
/// let bytes: [u8; 4] = [0xde, 0xad, 0, 0];
/// let foo: Result<Foo, ExternalError> = bytes.pread_into(0);
/// ```

pub trait Pread<E = error::Error, Ctx = Endian, I = usize, TryCtx = (I, Ctx), SliceCtx = (I, I, Ctx) >
 where E: Debug,
       Ctx: Copy + Default + Debug,
       I: Copy + Debug,
       TryCtx: Copy + Default + Debug,
       SliceCtx: Copy + Default + Debug,
{
    #[inline]
    /// Implement this if you need a faster version for use by `Gread`
    fn pread_unsafe<N: TryFromCtx<TryCtx, Error = E>>(&self, offset: I, ctx: Ctx) -> N {
        self.pread(offset, ctx).unwrap()
    }
    #[inline]
    /// Reads a value from `self` at `offset` with a default `Ctx`. For the primitive numeric values, this will read at the machine's endianness.
    /// # Example
    /// ```rust
    /// use scroll::Pread;
    /// let bytes = [0x7fu8; 0x01];
    /// let byte = bytes.pread_into::<u8>(0).unwrap();
    fn pread_into<N: TryFromCtx<TryCtx, Error = E>>(&self, offset: I) -> result::Result<N, E> {
        self.pread(offset, Ctx::default())
    }
    #[inline]
    /// Reads a value from `self` at `offset` with the given `ctx`
    /// # Example
    /// ```rust
    /// use scroll::Pread;
    /// let bytes: [u8; 2] = [0xde, 0xad];
    /// let dead: u16 = bytes.pread(0, scroll::BE).unwrap();
    /// assert_eq!(dead, 0xdeadu16);
    fn pread<N: TryFromCtx<TryCtx, Error = E>>(&self, offset: I, ctx: Ctx) -> result::Result<N, E>;
    /// Slices an `N` from `self` at `offset` up to `count` times
    #[inline]
    /// # Example
    /// ```rust
    /// use scroll::Pread;
    /// let bytes: [u8; 2] = [0x48, 0x49];
    /// let hi: &str = bytes.pread_slice(0, 2).unwrap();
    /// assert_eq!(hi, "HI");
    /// let bytes2 = bytes.pread_slice::<[u8]>(0, 2).unwrap();
    /// assert_eq!(bytes, bytes2);
    fn pread_slice<N: ?Sized + TryRefFromCtx<SliceCtx, Error = E>>(&self, offset: I, count: I) -> result::Result<&N, E>;
}

impl<E, Ctx> Pread<E, Ctx> for [u8]
    where
    E: Debug,
    Ctx: Debug + Copy + Default {
    #[inline]
    fn pread_unsafe<N: TryFromCtx<(usize, Ctx), Error = E>>(&self, offset: usize, le: Ctx) -> N {
        TryFromCtx::try_from_ctx(self, (offset, le)).unwrap()
    }
    #[inline]
    fn pread<N: TryFromCtx<(usize, Ctx), Error = E>>(&self, offset: usize, le: Ctx) -> result::Result<N, E> {
        TryFromCtx::try_from_ctx(self, (offset, le))
    }
    #[inline]
    fn pread_slice<N: ?Sized + TryRefFromCtx<(usize, usize, Ctx), Error = E>>(&self, offset: usize, count: usize) -> result::Result<&N, E> {
        TryRefFromCtx::try_ref_from_ctx(self, (offset, count, Ctx::default()))
    }
}

impl<E, Ctx, T> Pread<E, Ctx> for T
    where
    E: Debug,
    Ctx: Debug + Copy + Default,
    T: AsRef<[u8]> {
    #[inline]
    fn pread_unsafe<N: TryFromCtx<(usize, Ctx), Error = E>>(&self, offset: usize, le: Ctx) -> N {
        <[u8] as Pread<E, Ctx>>::pread_unsafe::<N>(self.as_ref(), offset, le)
        //FromCtx::try_from_ctx(self.as_ref(), offset, le)
    }
    #[inline]
    fn pread<N: TryFromCtx<(usize, Ctx), Error = E>>(&self, offset: usize, le: Ctx) -> result::Result<N, E> {
        TryFromCtx::try_from_ctx(self.as_ref(), (offset, le))
        //<[u8] as Pread<E, Ctx>>::pread::<N>(self.as_ref(), offset, le)
        //(*self.as_ref()).pread(offset, le)
    }
    #[inline]
    //fn pread_slice<N: ?Sized + TrySliceFrom<Error = E>>(&self, offset: usize, count: usize) -> result::Result<&N, E> {
    fn pread_slice<N: ?Sized + TryRefFromCtx<(usize, usize, Ctx), Error = E>>(&self, offset: usize, count: usize) -> result::Result<&N, E> {
        //(*self.as_ref()).pread_slice(offset, count)
        //TrySliceFrom::slice_from(self, offset, count)
        //<[u8] as Pread<E>>::pread_slice::<N>(self.as_ref(), offset, count)
        <[u8] as Pread<E, Ctx>>::pread_slice::<N>(self.as_ref(), offset, count)
    }
}

// impl<E, T> Pread<E> for Cursor<T> where T: AsRef<[u8]>, E: Debug {
//     #[inline]
//     fn pread_unsafe<N: FromCtx>(&self, offset: usize, le: super::Endian) -> N {
//         <[u8] as Pread<E>>::pread_unsafe::<N>(self.get_ref().as_ref(), offset, le)
//             //(*self.get_ref().as_ref()).pread_unsafe(offset, le)
//     }
//     #[inline]
//     fn pread<N: TryFromCtx<(usize, super::Endian), Error = E>>(&self, offset: usize, le: super::Endian) -> result::Result<N, E> {
//         (*self.get_ref().as_ref()).pread(offset, le)
//     }
//     #[inline]
//     fn pread_slice<N: ?Sized + TrySliceFrom<Error = E>>(&self, offset: usize, count: usize) -> result::Result<&N, E> {
//         (*self.get_ref().as_ref()).pread_slice(offset, count)
//     }
// }

// /// Implements a parallelized Pread + PwriteCompat bytes buffer
// pub mod parallel {

//     //use crossbeam;
//     use std::collections::HashSet;
//     use buffer;
//     use std::ops;
//     use std::io;
//     use std::result;
//     use std::slice;
//     use std::fs::OpenOptions;

//     use measure::Measure;
//     use greater::GreaterWrite;

//     type Range = ops::Range<usize>;

//     quick_error! {
//         #[derive(Debug)]
//         pub enum Error {
//             Io(err: io::Error) { from () }
//             EmptyFlush { from() }
//         }
//     }

//     type Result<T> = result::Result<T, Error>;

//     #[derive(Debug, Clone)]
//     pub enum Data {
//         U8(u8),
//         U16(u16),
//         U32(u32),
//         U64(u64),
//         Bytes(String),
//     }

//     impl Measure for Data {
//         fn measure(&self) -> usize {
//             match *self {
//                 Data::U8(_) => 1,
//                 Data::U16(_) => 2,
//                 Data::U32(_) => 4,
//                 Data::U64(_) => 8,
//                 Data::Bytes(ref string) => string.len(),
//             }
//         }
//     }

//     type Inner = buffer::Buffer;

//     /// A buffer for parallel reading and parallel writing
//     /// Describes the interface necessary to perform an effectless offset computation
//     /// on the underlying _writer_
//     /// NOTE: if every offset and write len computation is not an intersecting range of some other range in the in-flight writes
//     /// The write can be performed _completely_ in parallel with _no_ locking, by definition, as every write
//     /// is guaranteed to be free from the range of any other write.
//     //#[derive(Debug, Default)]
//     pub struct Buffer {
//         /// map of current in flight write computations
//         //ranges: HashMap<u64, Vec<Range>>,
//         ranges: HashSet<Vec<Range>>,
//         commits: Vec<(Data, Range)>,
//         //commits: crossbeam::sync::MsQueue<(Data, Range)>,
//         //commits: crossbeam::sync::chase_lev::Deque<(Data, Range)>,
//         //commits: crossbeam::sync::chase_lev::Worker<(Data, Range)>,
//         //flusher: crossbeam::sync::chase_lev::Stealer<(Data, Range)>,
//         /// underlying bytes
//         inner: Inner,
//         len: usize,
//         ncommits: usize,
//         /// debug buffer
//         _debug: Vec<String>,
//     }

//     impl Buffer {
//         pub fn new(buffer: buffer::Buffer) -> Buffer {
//             //let (commits, flusher) = crossbeam::sync::chase_lev::deque();
//             let commits = Vec::new();
//             Buffer {
//                 inner: buffer, ranges: HashSet::new(),
//                 _debug: Vec::new(),
//                 len: 0,
//                 ncommits: 0,
//                 //commits: commits, flusher: flusher,
//                 commits: commits,
//             }
//         }

//         pub fn len(&self) -> usize {
//             self.len
//         }

//         //pub fn commit(&mut self, data: Data, range: Range) {
//         pub fn commit(&mut self, data: Data, offset: usize) {
//             let count = data.measure();
//             self.len += count;
//             self.ncommits += 1;
//             let range = offset..(offset + count);
//             // todo: check commit conflicts here
//             self.commits.push((data, range));
//         }

//         //<greater::GreaterWrite>
//         fn write(memmap: &[u8], len: usize, data: Data, range: Range) -> io::Result<()> {
//             let memmap = memmap as *const _ as *mut u8;
//             let memmap = unsafe { slice::from_raw_parts_mut(memmap, len) };
//             let offset = range.start;
//             match data {
//                 Data::U8(n) =>  memmap.write_u8           (offset, n),
//                 Data::U16(n) => memmap.write_u16          (offset, n, true),
//                 Data::U32(n) => memmap.write_u32          (offset, n, true),
//                 Data::U64(n) => memmap.write_u64          (offset, n, true),
//                 Data::Bytes(string) => memmap.write_slice (offset, string.as_bytes()),
//             }
//         }

//         pub fn flush(self, name: &str) -> Result<()> {
//             use rayon::prelude::*;
//             use memmap;
//             if self.len != 0 {
//                 //let file = File::create(name)?;
//                 let file = OpenOptions::new()
//                     .read(true)
//                     .write(true)
//                     .create(true)
//                     .open(name)?;
//                 println!("file: {:?}", file);
//                 file.set_len(self.len as u64)?;
//                 println!("len: {:?}", file.metadata().unwrap().len());
//                 let memmap = memmap::Mmap::open(&file, memmap::Protection::ReadWrite)?;
//                 println!("memmamp: {:?}", memmap);
//                 let memmap_slice = unsafe { memmap.as_slice() };
//                 let commits = self.commits;
//                 let len = self.len;
//                 commits.into_par_iter().for_each( | (data, range) | {
//                     //println!("data: {:?}", data);
//                     Buffer::write(memmap_slice, len, data, range).unwrap();
//                 });
//                 memmap.flush()?;
//                 Ok(())
//             } else {
//                 return Err(Error::EmptyFlush)
//             }
//         }
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use buffer::Buffer;

//     #[test]
//     fn test_parallel_buffer() {
//         let crt1: Vec<u8> = include!("../crt1.rs");
//         let len2 = crt1.len();
//         let mut buffer = parallel::Buffer::new(Buffer::new(crt1));

//         buffer.commit(parallel::Data::U16(0xdead), 0);
//         buffer.commit(parallel::Data::U16(0xbeef), 2);
//         buffer.commit(parallel::Data::Bytes("lol i'm literally the best".to_owned()), 4);
//         let len1 = buffer.len();
//         buffer.flush("test.data").unwrap();
//         assert!(true);
//     }

//     #[test]
//     fn test_parallel_buffer2() {
//         use std::fs::File;
//         //let libc: Vec<u8> = include!("../libc.rs");
//         let mut file = File::open("libc.so.6").unwrap();
//         println!("file: {:?}", file);
//         let b = Buffer::try_from(file).unwrap();
//         let libc: Vec<u8> = b.into_inner();
//         let len2 = libc.len();
//         let mut buffer = parallel::Buffer::new(Buffer::new(&libc));
//         let mut offset = 0;
//         for byte in libc {
//             buffer.commit(parallel::Data::U8(byte), offset);
//             offset += 1;
//         }
//         let len1 = buffer.len();
//         buffer.flush("libc.test").unwrap();
//         assert!(len1 == len2);
//     }
// }
