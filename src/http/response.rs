//! A HTTP request

use crate::{error::Error, utils::rcvec::RcVec};
use std::{
    fs::File,
    io::{self, BufReader, Cursor, Read, Seek, Write},
};

/// A HTTP body
pub enum Body {
    /// An empty body
    Empty,
    /// A file to be served
    File(BufReader<File>),
    /// Some data to be served
    Data(Cursor<Vec<u8>>),
    /// Some static data to be served
    Static(&'static [u8]),
    /// Some other stuff to be served
    Other(Box<dyn Read>),
}
impl Read for Body {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Body::Empty => Ok(0),
            Body::File(file) => file.read(buf),
            Body::Data(data) => data.read(buf),
            Body::Static(data) => data.read(buf),
            Body::Other(other) => other.read(buf),
        }
    }
}

/// A HTTP response
#[derive(Debug)]
pub struct Response<T = Body, const HEADER_SIZE_MAX: usize = 4096> {
    /// The HTTP version
    pub version: RcVec<u8>,
    /// The response status code
    pub status: RcVec<u8>,
    /// The response status reason
    pub reason: RcVec<u8>,
    /// The response header fields
    #[allow(clippy::type_complexity)]
    pub fields: Vec<(RcVec<u8>, RcVec<u8>)>,
    /// The response body
    pub body: T,
}
impl<const HEADER_SIZE_MAX: usize> Response<Body, HEADER_SIZE_MAX> {
    /// Creates a new HTTP response
    pub fn new(version: RcVec<u8>, status: RcVec<u8>, reason: RcVec<u8>) -> Self {
        Self { version, status, reason, fields: Vec::new(), body: Body::Empty }
    }
}
impl<T, const HEADER_SIZE_MAX: usize> Response<T, HEADER_SIZE_MAX> {
    /// Writes the response to the given stream
    pub fn to_stream<S>(&mut self, stream: &mut S) -> Result<(), Error>
    where
        S: Write,
        T: Read,
    {
        // Create a temporary buffer
        let mut buf = Cursor::new([0; HEADER_SIZE_MAX]);

        // Write start line
        buf.write_all(&self.version)?;
        buf.write_all(b" ")?;
        buf.write_all(&self.status)?;
        buf.write_all(b" ")?;
        buf.write_all(&self.reason)?;
        buf.write_all(b"\r\n")?;

        // Write header fields and finalize header
        for (key, value) in &self.fields {
            buf.write_all(key)?;
            buf.write_all(b": ")?;
            buf.write_all(value)?;
            buf.write_all(b"\r\n")?;
        }
        buf.write_all(b"\r\n")?;

        // Write the header
        let header_size = buf.stream_position()?;
        let buf = buf.into_inner();
        stream.write_all(&buf[..header_size as usize])?;

        // Copy the buffer
        io::copy(&mut self.body, stream)?;
        Ok(())
    }

    /// Checks if the header has `Connection: Close` set
    pub fn has_connection_close(&self) -> bool {
        // Search for `Connection` header
        for (key, value) in &self.fields {
            if key.eq_ignore_ascii_case(b"connection") {
                return value.eq_ignore_ascii_case(b"close");
            }
        }
        false
    }
}
