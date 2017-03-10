use core::fmt::{self, Display};
use core::result;
use core::ops::Range;

#[cfg(feature = "std")]
use std::io;
#[cfg(feature = "std")]
use std::error;

#[derive(Debug)]
/// A custom Scroll error
pub enum Error {
    BadOffset(usize),
    BadRange(Range<usize>, usize),
    BadInput(Range<usize>, usize, &'static str),
    #[cfg(feature = "std")]
    Custom(String),
    #[cfg(feature = "std")]
    IO(io::Error),
}

#[cfg(feature = "std")]
impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::BadOffset(_) => { "BadOffset" }
            Error::BadRange(_, _) => { "BadRange" }
            Error::BadInput(_, _, _msg) => { "BadInput" }
            Error::Custom(_) => { "Custom" }
            Error::IO(_) => { "IO" }
        }
    }
    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::BadOffset(_) => { None }
            Error::BadRange(_, _) => { None }
            Error::BadInput(_, _, _) => { None }
            Error::Custom(_) => { None }
            Error::IO(ref io) => { io.cause() }
        }
    }
}

#[cfg(feature = "std")]
impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IO(err)
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::BadOffset(ref offset) => { write! (fmt, "bad offset {}", offset) },
            Error::BadRange(ref range, ref size) => {
                write!(fmt, "requested range [{}..{}) from object of len {}", range.start, range.end, size)
            },
            Error::BadInput(ref range, ref size, msg) => {
                write!(fmt, "{} - range [{}..{}), len {}", msg, range.start, range.end, size)
            },
            #[cfg(feature = "std")]
            Error::Custom(ref msg) => { write! (fmt, "{}", msg) },
            #[cfg(feature = "std")]
            Error::IO(ref err) => { write!(fmt, "{}", err) },
        }
    }
}

pub type Result<T> = result::Result<T, Error>;
