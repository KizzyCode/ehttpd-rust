//! A stack-allocating, type-abstract data type

use std::{
    any::Any,
    fmt::{Debug, Display, Formatter},
    ops::{Deref, Range},
    sync::Arc,
};

/// A stack-allocating, type-abstract data type
#[non_exhaustive]
pub enum Data {
    /// Some empty data
    Empty,
    /// Some `Vec<u8>`-backed data
    Vec(Vec<u8>),
    /// Some static data
    Static(&'static [u8]),
    /// An `Arc`ed vector to build lifetime-independent (sub)slices over the same set of elements
    ///
    /// # Note
    /// In general, this variant should not be created "by hand"; use `Self::new_arcvec` instead
    ArcVec {
        /// The data backing
        backing: Arc<Vec<u8>>,
        /// The referenced data within the backing
        range: Range<usize>,
    },
    /// A catch-all/opaque variant for all types that cannot be covered by the enum's specific variants
    ///
    /// # Note
    /// In general, this variant should not be created "by hand"; use `Self::new_other` instead
    Other {
        /// The underlying data backing
        data: Box<dyn Any + Send>,
        /// The referenced data within the backing
        range: Range<usize>,
        /// A pointer to type-specific implementation to recover the original type and coerce it to `&dyn AsRef<[u8]>`
        #[doc(hidden)]
        as_ref_u8: fn(&Box<dyn Any + Send>) -> &dyn AsRef<[u8]>,
        /// A pointer to type-specific implementation to recover the original type and coerce it to `&dyn Debug`
        #[doc(hidden)]
        as_debug: fn(&Box<dyn Any + Send>) -> &dyn Debug,
        /// A pointer to type-specific implementation to recover the original type and perform a `Clone::clone` operation
        #[doc(hidden)]
        do_clone: fn(&Box<dyn Any + Send>) -> Box<dyn Any + Send>,
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
    pub fn new_other<T>(typed: T) -> Self
    where
        T: AsRef<[u8]> + Debug + Clone + Send + 'static,
    {
        /// The specific implementation to recover `&dyn Any` as `&T` and coerce it to `&dyn AsRef<[u8]>`
        fn as_ref_u8<T>(untyped: &Box<dyn Any + Send>) -> &dyn AsRef<[u8]>
        where
            T: AsRef<[u8]> + 'static,
        {
            let typed: &T = untyped.downcast_ref().expect("failed to recover type");
            typed
        }
        /// The specific implementation to recover `&dyn Any` as `&T` and coerce it to `&dyn Debug`
        fn as_debug<T>(untyped: &Box<dyn Any + Send>) -> &dyn Debug
        where
            T: Debug + 'static,
        {
            let typed: &T = untyped.downcast_ref().expect("failed to recover type");
            typed
        }
        /// The specific implementation to recover `&dyn Any` as `&T` clone it
        fn do_clone<T>(untyped: &Box<dyn Any + Send>) -> Box<dyn Any + Send>
        where
            T: Clone + Send + 'static,
        {
            let typed: &T = untyped.downcast_ref().expect("failed to recover type");
            let cloned = typed.clone();
            Box::new(cloned)
        }

        // Box the value and init self
        let range = 0..typed.as_ref().len();
        let untyped: Box<dyn Any + Send> = Box::new(typed);
        Self::Other {
            data: untyped,
            range,
            as_ref_u8: as_ref_u8::<T>,
            as_debug: as_debug::<T>,
            do_clone: do_clone::<T>,
        }
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
            Self::Other { data, range, as_ref_u8, .. } => {
                let slice = as_ref_u8(data).as_ref();
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
            Self::Other { data, range, as_debug, .. } => {
                f.debug_struct("Other").field("data", as_debug(data)).field("range", &range).finish()
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
            Self::Other { data, range, as_ref_u8, as_debug, do_clone } => Self::Other {
                data: do_clone(data),
                range: range.clone(),
                as_ref_u8: *as_ref_u8,
                as_debug: *as_debug,
                do_clone: *do_clone,
            },
        }
    }
}
impl Display for Data {
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
