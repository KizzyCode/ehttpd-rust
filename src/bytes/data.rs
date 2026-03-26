//! An owned, type-abstract data type

use std::any::Any;
use std::fmt::{Debug, Display, Formatter, Write};
use std::mem;
use std::ops::{Bound, Deref, Range, RangeBounds};
use std::sync::Arc;

/// A type-abstract owned data type
///
/// # Rationale
/// The idea behind this type is to provide some dynamic polymorphism, but with "fast-paths" for common types to avoid
/// the overhead of unneeded memory allocations. Furthermore, it supports "cheap-cloning" via stack- or [`Arc`]-based
/// backings, so data can be efficiently shared in lifetime-indepent locations.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum Data {
    /// Some static data
    #[doc(hidden)]
    Static(&'static [u8]),
    /// An `Arc`ed slice to build lifetime-independent (sub)slices over the same set of elements
    #[doc(hidden)]
    Heap {
        /// The heap-allocated buffer
        data: Arc<[u8]>,
        /// The referenced data within the backing
        range: Range<usize>,
    },
}
impl Data {
    /// Creates a [`Data`] variant with the given `data`
    ///
    /// # Performance Considerations
    /// This function potentially allocates on heap. If you have static or stack-sized data, you might gain performance
    /// by calling the specialized constructors [`Self::new_empty`] or [`Self::new_static`].
    pub fn new<T>(mut data: T) -> Self
    where
        T: Into<Vec<u8>> + 'static,
    {
        let any_data: &mut dyn Any = &mut data;
        if let Some(data) = any_data.downcast_mut::<Self>() {
            // Avoid double-packing of `Self`
            mem::take(data)
        } else {
            // Pack the given data object
            let data: Arc<[u8]> = Arc::from(data.into());
            Self::Heap { range: 0..data.len(), data }
        }
    }
    /// Creates an empty data variant
    pub const fn new_empty() -> Self {
        Self::Static(b"")
    }
    /// Creates a static data variant
    pub const fn new_static(static_: &'static [u8]) -> Self {
        Self::Static(static_)
    }

    /// Creates a lifetime-independent subdata copy/refcopy over `self`
    ///
    /// # Note
    /// This method uses the cheapest way to clone the data by e.g. performing an `Rc::clone` on `Self::RcVec`
    pub fn subcopy<T>(&self, range: T) -> Option<Self>
    where
        T: RangeBounds<usize>,
    {
        // Get variant-dependent fields
        let current_range = match self {
            Data::Static(static_) => 0..static_.len(),
            Data::Heap { range, .. } => range.start..range.end,
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
            Data::Static(static_) => Data::Static(&static_[start..end]),
            Data::Heap { data: heap, .. } => Data::Heap { data: heap.clone(), range: start..end },
        };
        Some(clone)
    }
}
impl Deref for Data {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
impl AsRef<[u8]> for Data {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Static(static_) => static_,
            Self::Heap { data: heap, range } => &heap[range.start..range.end],
        }
    }
}
impl Display for Data {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        for byte in self.as_ref() {
            // Check if the byte is printable
            let printable = byte.is_ascii_alphanumeric() || *byte == b' ';
            match printable {
                true => f.write_char(*byte as char)?,
                false => write!(f, r#"\x{:02x}"#, *byte)?,
            }
        }
        Ok(())
    }
}
impl PartialEq<[u8]> for Data {
    fn eq(&self, other: &[u8]) -> bool {
        self.as_ref().eq(other)
    }
}
impl<const SIZE: usize> PartialEq<[u8; SIZE]> for Data {
    fn eq(&self, other: &[u8; SIZE]) -> bool {
        self.as_ref().eq(other)
    }
}
impl PartialEq<str> for Data {
    fn eq(&self, other: &str) -> bool {
        self.as_ref().eq(other.as_bytes())
    }
}
impl PartialEq<&[u8]> for Data {
    fn eq(&self, other: &&[u8]) -> bool {
        self.eq(*other)
    }
}
impl<const SIZE: usize> PartialEq<&[u8; SIZE]> for Data {
    fn eq(&self, other: &&[u8; SIZE]) -> bool {
        self.eq(*other)
    }
}
impl PartialEq<&str> for Data {
    fn eq(&self, other: &&str) -> bool {
        self.eq(*other)
    }
}
impl Default for Data {
    fn default() -> Self {
        Self::new_empty()
    }
}
impl From<Vec<u8>> for Data {
    fn from(value: Vec<u8>) -> Self {
        Self::new(value)
    }
}
impl From<&'static [u8]> for Data {
    fn from(value: &'static [u8]) -> Self {
        Self::new_static(value)
    }
}
impl<const SIZE: usize> From<&'static [u8; SIZE]> for Data {
    fn from(value: &'static [u8; SIZE]) -> Self {
        Self::new_static(value)
    }
}
impl From<String> for Data {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}
impl From<&'static str> for Data {
    fn from(value: &'static str) -> Self {
        Self::new_static(value.as_bytes())
    }
}
