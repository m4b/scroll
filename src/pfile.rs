use std::fs::File;
use std::io;
use std::ops::{Deref, DerefMut};

use memmap::{Protection, Mmap};

pub struct Pfile {
    inner: Mmap,
}

impl Pfile {
    pub fn with(path: &str) -> io::Result<Self> {
        let inner = Mmap::open_path(path, Protection::Read)?;
        Ok(Pfile { inner: inner })
    }
    pub fn new(fd: File) -> io::Result<Self> {
        let inner = Mmap::open(&fd, Protection::Read)?;
        Ok(Pfile { inner: inner })
    }

    pub fn new2(fd: File) -> io::Result<Self> {
        let inner = Mmap::open(&fd, Protection::ReadWrite)?;
        Ok(Pfile { inner: inner })
    }

    pub fn len(&self) -> usize {
        unsafe { self.inner.as_slice().len() }
    }
}

impl AsRef<[u8]> for Pfile {
    fn as_ref (&self) -> &[u8] {
        unsafe { self.inner.as_slice() }
    }
}

impl AsMut<[u8]> for Pfile {
    fn as_mut (&mut self) -> &mut [u8] {
        unsafe { self.inner.as_mut_slice() }
    }
}

impl Deref for Pfile {
    type Target = [u8];
    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { self.inner.as_slice() }
    }
}

impl DerefMut for Pfile {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.inner.as_mut_slice() }
    }
}
