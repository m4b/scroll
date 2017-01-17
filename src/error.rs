use core::fmt::{self, Display};
use core::result;

#[cfg(feature = "std")]
use std::io;
#[cfg(feature = "std")]
use std::error;

#[derive(Debug)]
/// A custom Scroll error
pub enum Error {
    BadOffset(String),
    BadInput(String),
    #[cfg(feature = "std")]
    IO(io::Error),
}

#[cfg(feature = "std")]
impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::BadOffset(_) => { "BadOffset" }
            Error::BadInput(_) => { "BadInput" }
            Error::IO(_) => { "IO" }
        }
    }
    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::IO(ref io) => { io.cause() }
            Error::BadOffset(_) => { None }
            Error::BadInput(_) => { None }
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
            Error::BadOffset(ref msg) => { write! (fmt, "{}", msg) },
            Error::BadInput(ref msg) => { write! (fmt, "{}", msg) },
            #[cfg(feature = "std")]
            Error::IO(ref err) => { write!(fmt, "{}", err) },
        }
    }
}

pub type Result<T> = result::Result<T, Error>;
