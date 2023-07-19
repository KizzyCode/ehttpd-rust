//! A writeable data sink

use std::{
    fmt::{Debug, Formatter},
    fs::File,
    io::{self, Write},
    net::TcpStream,
    panic::UnwindSafe,
};

/// An umbrella trait to combine `Write`, `Debug` and `Send` which are required for `Sink`
pub trait AnySink {
    /// `self` as implementor of `Write`
    fn as_write_mut(&mut self) -> &mut dyn Write;
    /// `self` as implementor of `Debug`
    fn as_debug(&self) -> &dyn Debug;
}
impl<T> AnySink for T
where
    T: Write + Debug + Send,
{
    fn as_write_mut(&mut self) -> &mut dyn Write {
        self
    }
    fn as_debug(&self) -> &dyn Debug {
        self
    }
}

/// A type-abstract data sink
///
/// # Rationale
/// The idea behind this type is to provide some dynamic polymorphism, but with some "fast-paths" for common types to
/// avoid the overhead of boxing and vtable-lookup (while the latter is probable negligible, the former may be significant
/// overhead if all you want is to write to some preallocated memory).
#[non_exhaustive]
pub enum Sink {
    /// A writer which will move data into the void
    Null,
    /// A vector sink
    Vector(Vec<u8>),
    /// A file
    File(File),
    /// A TCP stream
    TcpStream(TcpStream),
    /// A catch-all/opaque variant for all types that cannot be covered by the enum's specific variants
    Other(Box<dyn AnySink + Send + UnwindSafe>),
}
impl Sink {
    /// Creates a new catch-all/opaque variant from a typed object by moving it to the heap
    pub fn from_other<T>(typed: T) -> Self
    where
        T: AnySink + Send + UnwindSafe + 'static,
    {
        let boxed = Box::new(typed);
        Self::Other(boxed)
    }
}
impl Write for Sink {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Sink::Null => io::sink().write(buf),
            Sink::Vector(vector) => vector.write(buf),
            Sink::File(file) => file.write(buf),
            Sink::TcpStream(tcp_stream) => tcp_stream.write(buf),
            Sink::Other(other) => other.as_write_mut().write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Sink::Null => Ok(()),
            Sink::Vector(vector) => vector.flush(),
            Sink::File(file) => file.flush(),
            Sink::TcpStream(tcp_stream) => tcp_stream.flush(),
            Sink::Other(other) => other.as_write_mut().flush(),
        }
    }
}
impl Debug for Sink {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::Null => f.debug_tuple("Null").finish(),
            Self::Vector(arg0) => f.debug_tuple("Vector").field(arg0).finish(),
            Self::File(arg0) => f.debug_tuple("File").field(arg0).finish(),
            Self::TcpStream(arg0) => f.debug_tuple("TcpStream").field(arg0).finish(),
            Self::Other(other) => f.debug_tuple("Other").field(other.as_debug()).finish(),
        }
    }
}
impl Default for Sink {
    fn default() -> Self {
        Self::Null
    }
}
impl From<Vec<u8>> for Sink {
    fn from(value: Vec<u8>) -> Self {
        Self::Vector(value)
    }
}
impl From<File> for Sink {
    fn from(value: File) -> Self {
        Self::File(value)
    }
}
impl From<TcpStream> for Sink {
    fn from(value: TcpStream) -> Self {
        Self::TcpStream(value)
    }
}
