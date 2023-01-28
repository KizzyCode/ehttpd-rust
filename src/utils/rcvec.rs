//! Provides an `Rc`ed vector to build lifetime-independent slices over the same set of elements

use std::{
    fmt::{Display, Formatter},
    ops::{Bound, Deref, Range, RangeBounds},
    rc::Rc,
};

/// An `Rc`ed vector to build lifetime-independent slices over the same set of elements
#[derive(Debug, Clone)]
pub struct RcVec<T> {
    /// The data backing
    backing: Rc<Vec<T>>,
    /// The referenced data within the backing
    range: Range<usize>,
}
impl<T> RcVec<T> {
    /// Creates a new RcVec by wrapping `vec`
    pub fn new(vec: Vec<T>) -> Self {
        let backing = Rc::new(vec);
        let range = 0..backing.len();
        Self { backing, range }
    }

    /// Creates a new subvec over `self`
    pub fn subvec<R>(&self, range: R) -> Option<Self>
    where
        R: RangeBounds<usize>,
    {
        // Compute the bounds
        let start = match range.start_bound() {
            Bound::Included(start) => *start,
            Bound::Excluded(before_start) => *before_start + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(before_end) => *before_end + 1,
            Bound::Excluded(end) => *end,
            Bound::Unbounded => self.len(),
        };

        // Make the bounds relative to our current slice and validate them
        let start = self.range.start + start;
        let end = self.range.start + end;
        if start < self.range.start || end > self.range.end {
            return None;
        }

        // Create the subref
        Some(Self { backing: self.backing.clone(), range: start..end })
    }

    /// The underlying backing and the referenced range
    pub fn inner(&self) -> (&Vec<T>, &Range<usize>) {
        (&self.backing, &self.range)
    }
    /// Destructures `self` and returns the underlying backing as well as the referenced range
    pub fn try_into_inner(self) -> Result<(Vec<T>, Range<usize>), Self> {
        match Rc::try_unwrap(self.backing) {
            Ok(backing) => Ok((backing, self.range)),
            Err(backing) => Err(Self { backing, range: self.range }),
        }
    }
}
impl<T> Deref for RcVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.backing[self.range.start..self.range.end]
    }
}
impl Display for RcVec<u8> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        for byte in self.as_ref() {
            // Check if the byte is printable
            let printable = byte.is_ascii_alphanumeric();
            match printable {
                true => write!(f, "{}", *byte as char)?,
                false => write!(f, r#"\x{:02x}"#, *byte)?,
            }
        }
        Ok(())
    }
}
