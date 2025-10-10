//! An owned, type-abstract writeable data sink

use std::{
    any::Any,
    fmt::{Debug, Formatter},
    fs::File,
    io::{self, BufWriter, Write},
    net::TcpStream,
};

/// An owned, type-abstract data sink
pub struct Sink {
    /// The underlying writer
    sink: Box<dyn Any + Send + Sync>,
    /// Vtable mapper to expose the underlying `Write` trait
    vtable_as_writer: fn(&mut Box<dyn Any + Send + Sync>) -> &mut dyn Write,
    /// Vtable mapper to expose the underlying `Debug` trait
    vtable_as_debug: fn(&Box<dyn Any + Send + Sync>) -> &dyn Debug,
}
impl Sink {
    /// Wraps the given sink
    pub fn new<T>(sink: T) -> Self
    where
        T: Write + Debug + Send + Sync + 'static,
    {
        Self {
            sink: Box::new(sink),
            vtable_as_writer: |sink| sink.downcast_mut::<T>().expect("vtable type confusion"),
            vtable_as_debug: |sink| sink.downcast_ref::<T>().expect("vtable type confusion"),
        }
    }
}
impl Write for Sink {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (self.vtable_as_writer)(&mut self.sink).write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        (self.vtable_as_writer)(&mut self.sink).flush()
    }
}
impl Debug for Sink {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let writer = (self.vtable_as_debug)(&self.sink);
        f.debug_struct("AnyWriter")
            .field("writer", writer)
            .field("as_writer", &self.vtable_as_writer)
            .field("as_debug", &self.vtable_as_debug)
            .finish()
    }
}
impl Default for Sink {
    fn default() -> Self {
        Self::new(io::empty())
    }
}
impl From<Vec<u8>> for Sink {
    fn from(sink: Vec<u8>) -> Self {
        Self::new(sink)
    }
}
impl From<BufWriter<File>> for Sink {
    fn from(sink: BufWriter<File>) -> Self {
        Self::new(sink)
    }
}
impl From<BufWriter<TcpStream>> for Sink {
    fn from(sink: BufWriter<TcpStream>) -> Self {
        Self::new(sink)
    }
}
