//! A stack-allocating, type-abstract data type

use std::{
    fmt::{Debug, Display, Formatter, Write},
    ops::{Deref, Range},
    sync::Arc,
};

/// An umbrella trait to combine `AsRef<[u8]>`, `Debug`, `Clone` and `Send` which are required for `Data`
pub trait AnyData {
    /// `self` as slice of bytes
    fn as_bytes(&self) -> &[u8];
    /// `self` as implementor of `Debug`
    fn as_debug(&self) -> &dyn Debug;
    /// Clones `self`
    fn opaque_clone(&self) -> Box<dyn AnyData + Send>;
}
impl<T> AnyData for T
where
    T: AsRef<[u8]> + Debug + Clone + Send + 'static,
{
    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
    fn as_debug(&self) -> &dyn Debug {
        self
    }
    fn opaque_clone(&self) -> Box<dyn AnyData + Send> {
        let clone = self.clone();
        Box::new(clone)
    }
}

/// A type-abstract owned data type
///
/// # Rationale
/// The idea behind this type is to provide some dynamic polymorphism, but with some "fast-paths" for common types to
/// avoid the overhead of boxing and vtable-lookup (while the latter is probable negligible, the former may be significant
/// overhead if all you want is to reference some static memory).
#[non_exhaustive]
pub enum Data {
    /// Some empty data
    Empty,
    /// Some `Vec<u8>`-backed data
    Vec(Vec<u8>),
    /// Some static data
    Static(&'static [u8]),
    /// An `Arc`ed vector to build lifetime-independent (sub)slices over the same set of elements
    ArcVec {
        /// The data backing
        backing: Arc<Vec<u8>>,
        /// The referenced data within the backing
        range: Range<usize>,
    },
    /// A catch-all/opaque variant for all types that cannot be covered by the enum's specific variants
    Other {
        /// The underlying data backing
        data: Box<dyn AnyData + Send>,
        /// The referenced data within the backing
        range: Range<usize>,
    },
}
impl Data {
    /// Creates a new reference-counted data
    pub fn new_arcvec<T>(data: T) -> Self
    where
        T: Into<Vec<u8>>,
    {
        let backing = Arc::new(data.into());
        let range = 0..backing.len();
        Self::ArcVec { backing, range }
    }
    /// Creates a new catch-all/opaque variant from a typed object by moving it to the heap
    pub fn from_other<T>(typed: T) -> Self
    where
        T: AnyData + Send + 'static,
    {
        // Box the value and init self
        let range = 0..typed.as_bytes().len();
        let untyped: Box<dyn AnyData + Send> = Box::new(typed);
        Self::Other { data: untyped, range }
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
            Self::Empty => b"",
            Self::Vec(vec) => vec,
            Self::ArcVec { backing, range } => &backing[range.start..range.end],
            Self::Static(static_) => static_,
            Self::Other { data, range } => {
                let slice = data.as_bytes();
                &slice[range.start..range.end]
            }
        }
    }
}
impl Debug for Data {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::Empty => f.debug_tuple("Empty").finish(),
            Self::Vec(arg0) => f.debug_tuple("Vec").field(arg0).finish(),
            Self::Static(arg0) => f.debug_tuple("Static").field(arg0).finish(),
            Self::ArcVec { backing, range } => {
                f.debug_struct("RcVec").field("backing", &backing).field("range", &range).finish()
            }
            Self::Other { data, range } => {
                f.debug_struct("Other").field("data", data.as_debug()).field("range", &range).finish()
            }
        }
    }
}
impl Clone for Data {
    fn clone(&self) -> Self {
        match self {
            Self::Empty => Self::Empty,
            Self::Vec(arg0) => Self::Vec(arg0.clone()),
            Self::ArcVec { backing, range } => Self::ArcVec { backing: backing.clone(), range: range.clone() },
            Self::Static(arg0) => Self::Static(arg0),
            Self::Other { data, range } => Self::Other { data: data.opaque_clone(), range: range.clone() },
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
impl Default for Data {
    fn default() -> Self {
        Self::Empty
    }
}
impl From<Vec<u8>> for Data {
    fn from(value: Vec<u8>) -> Self {
        Self::Vec(value)
    }
}
impl From<&'static [u8]> for Data {
    fn from(value: &'static [u8]) -> Self {
        Self::Static(value)
    }
}
impl<const SIZE: usize> From<&'static [u8; SIZE]> for Data {
    fn from(value: &'static [u8; SIZE]) -> Self {
        Self::Static(value)
    }
}
impl From<String> for Data {
    fn from(value: String) -> Self {
        Self::Vec(value.into_bytes())
    }
}
impl From<&'static str> for Data {
    fn from(value: &'static str) -> Self {
        Self::Static(value.as_bytes())
    }
}
