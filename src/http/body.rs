//! A HTTP body abstraction

use std::{
    any::Any,
    fs::File,
    io::{self, Cursor, Read, Seek, SeekFrom},
};

/// An umbrella-trait to wrap implementations that both implement `Read` and `Seek`, with a fallback-coercion to `Any`
pub trait ReadSeek {
    /// Returns `self` as a mutable implementor of `Read`
    fn as_read_mut(&mut self) -> &mut dyn Read;
    /// Returns `self` as a mutable implementor of `Seek`
    fn as_seek_mut(&mut self) -> &mut dyn Seek;

    /// Returns `self` as an implementor of `Any`
    fn as_any(&self) -> &dyn Any;
    /// Returns `self` as a mutable implementor of `Any`
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
impl<T> ReadSeek for T
where
    T: Read + Seek + Any,
{
    fn as_read_mut(&mut self) -> &mut dyn Read {
        self
    }
    fn as_seek_mut(&mut self) -> &mut dyn Seek {
        self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// A HTTP body
#[non_exhaustive]
pub enum Body {
    /// An empty body
    Empty,
    /// A file to be served
    File(File),
    /// Some data to be served
    Data(Cursor<Vec<u8>>),
    /// Some static data to be served
    Static(Cursor<&'static [u8]>),
    /// Some other stuff to be served
    Other(Box<dyn ReadSeek>),
}
impl Read for Body {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Body::Empty => io::empty().read(buf),
            Body::File(file) => file.read(buf),
            Body::Data(data) => data.read(buf),
            Body::Static(data) => data.read(buf),
            Body::Other(other) => other.as_read_mut().read(buf),
        }
    }
}
impl Seek for Body {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        match self {
            Body::Empty => io::empty().seek(pos),
            Body::File(file) => file.seek(pos),
            Body::Data(data) => data.seek(pos),
            Body::Static(data) => data.seek(pos),
            Body::Other(other) => other.as_seek_mut().seek(pos),
        }
    }
}
