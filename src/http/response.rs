//! A HTTP request

use crate::{
    bytes::{Data, Source},
    error::Error,
};
use std::io::{self, Write};

/// A HTTP response
#[derive(Debug)]
pub struct Response<const HEADER_SIZE_MAX: usize = 4096> {
    /// The HTTP version
    pub version: Data,
    /// The response status code
    pub status: Data,
    /// The response status reason
    pub reason: Data,
    /// The response header fields
    pub fields: Vec<(Data, Data)>,
    /// The response body
    pub body: Source,
}
impl<const HEADER_SIZE_MAX: usize> Response<HEADER_SIZE_MAX> {
    /// Creates a new HTTP response
    pub fn new(version: Data, status: Data, reason: Data) -> Self {
        Self { version, status, reason, fields: Vec::new(), body: Source::default() }
    }

    /// Writes the response to the given stream
    pub fn to_stream<T>(&mut self, stream: &mut T) -> Result<(), Error>
    where
        T: Write,
    {
        // Create a temporary buffer
        let mut buf = Vec::with_capacity(HEADER_SIZE_MAX);

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

        // Write the header, and copy the body
        stream.write_all(&buf)?;
        io::copy(&mut self.body, stream)?;
        Ok(())
    }

    /// Checks if the header has `Connection: Close` set
    pub fn has_connection_close(&self) -> bool {
        // Search for `Connection` header
        for (key, value) in &self.fields {
            if key.eq_ignore_ascii_case(b"Connection") {
                return value.eq_ignore_ascii_case(b"Close");
            }
        }
        false
    }
}
