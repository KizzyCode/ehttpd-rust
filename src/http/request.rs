//! A HTTP request

use crate::bytes::{Data, Parse, Source};
use crate::err;
use crate::error::Error;
use std::io::Read;
use std::path::Path;

/// A HTTP request
#[derive(Debug)]
pub struct Request<'a, const HEADER_SIZE_MAX: usize = 4096> {
    /// The raw header bytes
    pub header: Data,
    /// The method part within the request line
    pub method: Data,
    /// The target part within the request line
    pub target: Data,
    /// The version part within the request line
    pub version: Data,
    /// The key/value fields within the header
    pub fields: Vec<(Data, Data)>,
    /// The connection stream
    pub stream: &'a mut Source,
}
// Core functionality
impl<'a, const HEADER_SIZE_MAX: usize> Request<'a, HEADER_SIZE_MAX> {
    /// Reads a HTTP request from a readable `stream`
    pub fn from_stream(stream: &'a mut Source) -> Result<Option<Self>, Error> {
        // Read the raw header or return `None` if the connection has been closed
        let header = Self::read_header(stream)?;
        if header.is_empty() {
            return Ok(None);
        }

        // Parse the start line
        let mut header_parsing = header.clone();
        let (method, target, version) = {
            let (method, target, version) = Self::parse_start_line(&mut header_parsing)?;
            (method.trim(), target.trim(), version.trim())
        };

        // Parse the fields
        let mut fields = Vec::new();
        while !header_parsing.eq(b"\r\n") {
            let (key, value) = Self::parse_field(&mut header_parsing)?;
            fields.push((key, value));
        }

        // Init self
        Ok(Some(Self { header, method, target, version, fields, stream }))
    }

    /// Reads the entire HTTP header from the stream
    fn read_header(stream: &mut Source) -> Result<Data, Error> {
        // Read the header
        let mut header = Vec::with_capacity(HEADER_SIZE_MAX);
        'read_loop: for byte in stream.bytes() {
            // Read the next byte
            let byte = byte?;
            header.push(byte);

            // Check if we have the header
            if header.ends_with(b"\r\n\r\n") {
                break 'read_loop;
            }
            if header.len() == HEADER_SIZE_MAX {
                return Err(err!("HTTP header is too large"));
            }
        }

        // Create the RcVec
        header.shrink_to_fit();
        let header = Data::from(header);
        Ok(header)
    }
    /// Parses the start line
    #[allow(clippy::type_complexity)]
    fn parse_start_line(header: &mut Data) -> Result<(Data, Data, Data), Error> {
        // Split the header line
        let mut line = header.split_off(b"\r\n").ok_or_else(|| err!("Truncated HTTP start line: {header}"))?;
        let method = line.split_off(b" ").ok_or_else(|| err!("Invalid HTTP start line: {line}"))?;
        let target = line.split_off(b" ").ok_or_else(|| err!("Invalid HTTP start line: {line}"))?;
        Ok((method, target, line))
    }
    /// Parses a header field
    fn parse_field(header: &mut Data) -> Result<(Data, Data), Error> {
        // Parse the field
        let mut line = header.split_off(b"\r\n").ok_or_else(|| err!("Truncated HTTP header field: {header}"))?;
        let key = line.split_off(b":").ok_or_else(|| err!("Invalid HTTP header field: {line}"))?;

        // Trim the field values
        let key = key.trim();
        let value = line.trim();
        Ok((key, value))
    }
}
// Useful helpers
impl<'a, const HEADER_SIZE_MAX: usize> Request<'a, HEADER_SIZE_MAX> {
    /// Gets the request target as path
    ///
    /// # Important
    /// On non-unix platforms, this function uses a `str` as intermediate representation, so the path must be valid
    /// UTF-8. If this might be a problem, you should use the raw target field directly.
    #[cfg(target_family = "unix")]
    pub fn target_path(&self) -> Option<&Path> {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        // Create the path directly without going via `str`
        let target = OsStr::from_bytes(&self.target);
        Some(Path::new(target))
    }
    /// Gets the request target as path
    ///
    /// # Important
    /// On non-unix platforms, this function uses a `str` as intermediate representation, so the path must be valid
    /// UTF-8. If this might be a problem, you should use the raw target field directly.
    #[cfg(not(any(target_family = "unix")))]
    pub fn target_path(&self) -> Option<&Path> {
        // Convert the target to UTF-8 and return it as string
        let target = str::from_utf8(&self.target).ok()?;
        Some(Path::new(target))
    }

    /// Gets the field with the given name (performs an ASCII-case-insensitve comparison)
    pub fn field<N>(&self, name: N) -> Option<&Data>
    where
        N: AsRef<[u8]>,
    {
        for (key, value) in &self.fields {
            // Perform a case-insensitive comparison since HTTP header field names are not case-sensitive
            if key.eq_ignore_ascii_case(name.as_ref()) {
                return Some(value);
            }
        }
        None
    }
    /// The request content length field if any
    pub fn content_length(&self) -> Result<Option<u64>, Error> {
        // Get the content length field if set
        let Some(content_length_raw) = self.field("Content-Length") else {
            // Content-length field is unset
            return Ok(None);
        };

        // Parse the field
        let content_length_utf8 = str::from_utf8(content_length_raw)?;
        let content_length: u64 = content_length_utf8.parse()?;
        Ok(Some(content_length))
    }

    /// Reads the request body into memory *if a content-length header is set* and *transfer-encoding is not chunked*;
    /// returns `None` otherwise
    pub fn read_body_data(&mut self, content_length_max: u64) -> Result<Option<Data>, Error> {
        // Check if the transfer encoding is chunked
        let is_chunked = self.field("Transfer-Encoding").map(|encoding| encoding.eq_ignore_ascii_case(b"chunked"));
        let (None | Some(false)) = is_chunked else {
            // Chunked transfer-encoding is not supported
            return Ok(None);
        };

        // Check if a content-length header is set
        let Some(content_length) = self.content_length()? else {
            // Indeterminate lengths are not supported
            return Ok(None);
        };

        // Validate content length
        let true = content_length <= content_length_max else {
            // Body is too large
            return Err(err!("HTTP body is too large"));
        };

        // Read body
        let mut body = Vec::new();
        let body_len = self.stream.take(content_length).read_to_end(&mut body)? as u64;
        let true = body_len == content_length else {
            // Truncated body
            return Err(err!("Truncated HTTP body"))?;
        };

        // Return body
        let body = Data::from(body);
        Ok(Some(body))
    }

    /// Checks if the header has `Connection: Close` set
    pub fn has_connection_close(&self) -> bool {
        // Search for `Connection` header
        for (key, value) in &self.fields {
            if key.eq_ignore_ascii_case(b"Connection") {
                return value.eq_ignore_ascii_case(b"close");
            }
        }
        false
    }
}
