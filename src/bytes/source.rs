//! An owned, type-abstract readable data source

use crate::bytes::data::Data;
use std::any::Any;
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::{BufReader, Cursor, Read};
use std::net::TcpStream;

/// An owned, type-abstract data source
pub struct Source {
    /// The underlying reader
    source: Box<dyn Any + Send + Sync>,
    /// Vtable mapper to expose the underlying `Write` trait
    vtable_as_reader: fn(&mut Box<dyn Any + Send + Sync>) -> &mut dyn Read,
    /// Vtable mapper to expose the underlying `Debug` trait
    vtable_as_debug: fn(&Box<dyn Any + Send + Sync>) -> &dyn Debug,
}
impl Source {
    /// Wraps the given source
    pub fn new<T>(source: T) -> Self
    where
        T: Read + Debug + Send + Sync + 'static,
    {
        Self {
            source: Box::new(source),
            vtable_as_reader: |source| source.downcast_mut::<T>().expect("vtable type confusion"),
            vtable_as_debug: |source| source.downcast_ref::<T>().expect("vtable type confusion"),
        }
    }
}
impl Debug for Source {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let source = (self.vtable_as_debug)(&self.source);
        f.debug_struct("Source")
            .field("source", source)
            .field("vtable_as_reader", &self.vtable_as_reader)
            .field("vtable_as_debug", &self.vtable_as_debug)
            .finish()
    }
}
impl Read for Source {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        (self.vtable_as_reader)(&mut self.source).read(buf)
    }
}
impl Default for Source {
    fn default() -> Self {
        Self::new(std::io::empty())
    }
}
impl From<Data> for Source {
    fn from(value: Data) -> Self {
        Self::new(Cursor::new(value))
    }
}
impl From<Cursor<Data>> for Source {
    fn from(value: Cursor<Data>) -> Self {
        Self::new(value)
    }
}
impl From<Vec<u8>> for Source {
    fn from(value: Vec<u8>) -> Self {
        Self::new(Cursor::new(value))
    }
}
impl From<&'static [u8]> for Source {
    fn from(value: &'static [u8]) -> Self {
        Self::new(Cursor::new(value))
    }
}
impl<const SIZE: usize> From<&'static [u8; SIZE]> for Source {
    fn from(value: &'static [u8; SIZE]) -> Self {
        Self::new(Cursor::new(value))
    }
}
impl From<String> for Source {
    fn from(value: String) -> Self {
        Self::new(Cursor::new(value))
    }
}
impl From<&'static str> for Source {
    fn from(value: &'static str) -> Self {
        Self::new(Cursor::new(value))
    }
}
impl From<BufReader<File>> for Source {
    fn from(value: BufReader<File>) -> Self {
        Self::new(value)
    }
}
impl From<BufReader<TcpStream>> for Source {
    fn from(value: BufReader<TcpStream>) -> Self {
        Self::new(value)
    }
}
