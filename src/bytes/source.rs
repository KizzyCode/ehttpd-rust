//! A readable data source

use crate::bytes::data::Data;
use std::{
    fmt::{Debug, Formatter},
    fs::File,
    io::{Cursor, Read},
    net::TcpStream,
    panic::UnwindSafe,
};

/// An umbrella trait to combine `Read`, `Debug` and `Send` which are required for `Source`
pub trait AnySource {
    /// `self` as implementor of `Read`
    fn as_read_mut(&mut self) -> &mut dyn Read;
    /// `self` as implementor of `Debug`
    fn as_debug(&self) -> &dyn Debug;
}
impl<T> AnySource for T
where
    T: Read + Debug + Send,
{
    fn as_read_mut(&mut self) -> &mut dyn Read {
        self
    }
    fn as_debug(&self) -> &dyn Debug {
        self
    }
}

/// A type-abstract data source
///
/// # Rationale
/// The idea behind this type is to provide some dynamic polymorphism, but with some "fast-paths" for common types to
/// avoid the overhead of boxing and vtable-lookup (while the latter is probable negligible, the former may be significant
/// overhead if all you want is to read from some static memory).
#[non_exhaustive]
pub enum Source {
    /// An empty source
    Empty,
    /// A linear data backed source
    Data(Cursor<Data>),
    /// A file
    File(File),
    /// A TCP stream
    TcpStream(TcpStream),
    /// A catch-all/opaque variant for all types that cannot be covered by the enum's specific variants
    Other(Box<dyn AnySource + Send + UnwindSafe>),
}
impl Source {
    /// Creates a new catch-all/opaque variant from a typed object by moving it to the heap
    pub fn from_other<T>(typed: T) -> Self
    where
        T: AnySource + Send + UnwindSafe + 'static,
    {
        let boxed = Box::new(typed);
        Self::Other(boxed)
    }
}
impl Read for Source {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Source::Empty => Ok(0),
            Source::Data(data) => data.read(buf),
            Source::File(file) => file.read(buf),
            Source::TcpStream(tcp_stream) => tcp_stream.read(buf),
            Source::Other(other) => other.as_read_mut().read(buf),
        }
    }
}
impl Debug for Source {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::Empty => f.debug_tuple("Empty").finish(),
            Self::Data(arg0) => f.debug_tuple("Data").field(arg0).finish(),
            Self::File(arg0) => f.debug_tuple("File").field(arg0).finish(),
            Self::TcpStream(arg0) => f.debug_tuple("TcpStream").field(arg0).finish(),
            Self::Other(other) => f.debug_tuple("Other").field(other.as_debug()).finish(),
        }
    }
}
impl Default for Source {
    fn default() -> Self {
        Self::Empty
    }
}
impl From<Data> for Source {
    fn from(value: Data) -> Self {
        Self::Data(Cursor::new(value))
    }
}
impl From<Cursor<Data>> for Source {
    fn from(value: Cursor<Data>) -> Self {
        Self::Data(value)
    }
}
impl From<Vec<u8>> for Source {
    fn from(value: Vec<u8>) -> Self {
        Self::from(Data::Vec(value))
    }
}
impl From<&'static [u8]> for Source {
    fn from(value: &'static [u8]) -> Self {
        Self::from(Data::Static(value))
    }
}
impl<const SIZE: usize> From<&'static [u8; SIZE]> for Source {
    fn from(value: &'static [u8; SIZE]) -> Self {
        Self::from(Data::Static(value))
    }
}
impl From<String> for Source {
    fn from(value: String) -> Self {
        Self::from(Data::Vec(value.into_bytes()))
    }
}
impl From<&'static str> for Source {
    fn from(value: &'static str) -> Self {
        Self::from(Data::Static(value.as_bytes()))
    }
}
impl From<File> for Source {
    fn from(value: File) -> Self {
        Self::File(value)
    }
}
impl From<TcpStream> for Source {
    fn from(value: TcpStream) -> Self {
        Self::TcpStream(value)
    }
}
