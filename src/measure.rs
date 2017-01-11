//#[cfg(feature = "std")]
//use std::io::Cursor;

/// Whether `Self` has a length that is measurable, and in what `Self::Units`
pub trait Measure {
    type Units;
    #[inline]
    fn measure(&self) -> Self::Units;
}

impl<T> Measure for AsRef<Vec<T>> {
    type Units = usize;
    #[inline]
    fn measure(&self) -> usize {
        self.as_ref().len()
    }
}

impl<T> Measure for T where T: AsRef<[u8]> {
    type Units = usize;
    #[inline]
    fn measure(&self) -> usize {
        self.as_ref().len()
    }
}

// requires specialization
// #[cfg(feature = "std")]
// impl<T: Measure> Measure for Cursor<T> {
//     #[inline]
//     fn measure(&self) -> usize {
//         self.get_ref().measure()
//     }
// }
