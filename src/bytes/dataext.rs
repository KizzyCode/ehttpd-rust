//! Some useful extensions for the `Data` type

use crate::bytes::Data;
use std::ops::{Bound, RangeBounds};

/// An extension trait to create lifetime-independent subdata "slices" over `self`
pub trait DataSliceExt
where
    Self: Sized,
{
    /// Creates a lifetime-independent subdata copy/refcopy over `self`
    ///
    /// # Note
    /// This method uses the cheapest way to clone the data by e.g. performing an `Rc::clone` on `Self::RcVec`
    fn subcopy<T>(&self, range: T) -> Option<Self>
    where
        T: RangeBounds<usize>;
}
impl DataSliceExt for Data {
    fn subcopy<T>(&self, range: T) -> Option<Self>
    where
        T: RangeBounds<usize>,
    {
        // Get variant-dependent fields
        let current_range = match self {
            Data::Empty => 0..0,
            Data::Vec(vec) => 0..vec.len(),
            Data::Static(static_) => 0..static_.len(),
            Data::ArcVec { range, .. } => range.start..range.end,
            Data::Other { range, .. } => range.start..range.end,
        };

        // Compute the bounds
        let start = match range.start_bound() {
            Bound::Included(start) => *start,
            Bound::Excluded(_) => unreachable!("excluded bounds are invalid for range starts"),
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(before_end) => before_end.saturating_add(1),
            Bound::Excluded(end) => *end,
            Bound::Unbounded => self.len(),
        };

        // Make the bounds relative to our current slice and validate them
        let start = current_range.start.checked_add(start)?;
        let end = current_range.start.checked_add(end)?;
        if start > current_range.end || end > current_range.end {
            return None;
        }

        // Create the subref
        let clone = match self {
            Data::Empty => Data::Empty,
            Data::Vec(vec) => Data::Vec(vec[start..end].to_vec()),
            Data::Static(static_) => Data::Static(&static_[start..end]),
            Data::ArcVec { backing, .. } => Data::ArcVec { backing: backing.clone(), range: start..end },
            Data::Other { data, .. } => Data::Other { data: data.opaque_clone(), range: start..end },
        };
        Some(clone)
    }
}

/// Some parsing extensions for `Data`
pub trait DataParseExt
where
    Self: Sized,
{
    /// Splits `self` on the first occurrence of `pat` and returns prefix before delimiter; `self` is updated to hold the
    /// remaining suffix after `pat`
    ///
    /// # Note
    /// This method uses the cheapest way to clone the data by e.g. performing an `Rc::clone` on `Self::RcVec`
    fn split_off(&mut self, pat: &[u8]) -> Option<Self>;
    /// Trims leading and trailing ASCII whitespaces
    fn trimmed(&self) -> Self;
}
impl DataParseExt for Data {
    fn split_off(&mut self, pat: &[u8]) -> Option<Self> {
        for (offset, haystack) in self.windows(pat.len()).enumerate() {
            // Check for match
            if haystack == pat {
                let split = self.subcopy(..offset).expect("invalid prefix offset");
                *self = self.subcopy(offset + pat.len()..).expect("invalid suffix offset");
                return Some(split);
            }
        }
        None
    }
    fn trimmed(&self) -> Self {
        // Trim the leading bytes
        let leading = self.iter().take_while(|byte| byte.is_ascii_whitespace()).count();
        let trimmed = self.subcopy(leading..).expect("invalid segment range");

        // Trim the trailing bytes
        let trailing = trimmed.iter().rev().take_while(|byte| byte.is_ascii_whitespace()).count();
        trimmed.subcopy(..trimmed.len() - trailing).expect("invalid segment range")
    }
}
