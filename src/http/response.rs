//! A HTTP request

use crate::bytes::{Data, Source};
use crate::error::Error;
use std::fs::File;
use std::io::{self, BufReader, Seek, SeekFrom, Write};

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
// Core functionality
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
                return value.eq_ignore_ascii_case(b"close");
            }
        }
        false
    }
}
// Useful helpers
impl<const HEADER_SIZE_MAX: usize> Response<HEADER_SIZE_MAX> {
    /// Creates a new HTTP response with the given status code and reason and sets an empty body
    pub fn new_status_reason<T>(status: u16, reason: T) -> Self
    where
        T: Into<Data>,
    {
        // Create basic request
        let version = Data::from(b"HTTP/1.1");
        let status = Data::from(status.to_string());
        let reason = reason.into();
        let mut this = Self::new(version, status, reason);

        // Set content-length to 0
        this.set_content_length(0);
        this
    }
    /// Creates a new `200 OK` HTTP response with an empty body
    pub fn new_200_ok() -> Self {
        Self::new_status_reason(200, "OK")
    }

    /// Creates a new `307 Temporary Redirect` HTTP response with an empty body and the `Location`-header field set to
    /// the given location
    pub fn new_307_temporaryredirect<T>(location: T) -> Self
    where
        T: Into<Data>,
    {
        let mut this = Self::new_status_reason(307, "Temporary Redirect");
        this.set_field("Location", location);
        this
    }

    /// Creates a new `400 Bad Request` HTTP response with an empty body
    pub fn new_400_badrequest() -> Self {
        Self::new_status_reason(400, "Bad Request")
    }
    /// Creates a new `401 Unauthorized` HTTP response  with an empty body and the `WWW-Authenticate`-header field set
    /// to the given requirement
    pub fn new_401_unauthorized<T>(requirement: T) -> Self
    where
        T: Into<Data>,
    {
        let mut this = Self::new_status_reason(401, "Unauthorized");
        this.set_field("WWW-Authenticate", requirement);
        this
    }
    /// Creates a new `403 Forbidden` HTTP response with an empty body
    pub fn new_403_forbidden() -> Self {
        Self::new_status_reason(403, "Forbidden")
    }
    /// Creates a new `404 Not Found` HTTP response with an empty body
    pub fn new_404_notfound() -> Self {
        Self::new_status_reason(404, "Not Found")
    }
    /// Creates a new `405 Method Not Allowed` HTTP response with an empty body
    pub fn new_405_methodnotallowed() -> Self {
        Self::new_status_reason(405, "Method Not Allowed")
    }
    /// Creates a new `413 Payload Too Large` HTTP response with an empty body
    pub fn new_413_payloadtoolarge() -> Self {
        Self::new_status_reason(413, "Payload Too Large")
    }
    /// Creates a new `416 Range Not Satisfiable` HTTP response with an empty body
    pub fn new_416_rangenotsatisfiable() -> Self {
        Self::new_status_reason(416, "Range Not Satisfiable")
    }

    /// Creates a new `500 Internal Server Error` HTTP response with an empty body
    pub fn new_500_internalservererror() -> Self {
        Self::new_status_reason(500, "Internal Server Error")
    }

    /// Sets the field with the given name (performs an ASCII-case-insensitve comparison for replacement)
    pub fn set_field<K, V>(&mut self, key: K, value: V)
    where
        K: Into<Data>,
        V: Into<Data>,
    {
        // Convert the field into vecs
        let key = key.into();
        let value = value.into();

        // Remove any field with the same name and set the field
        self.fields.retain(|(existing, _)| !key.eq_ignore_ascii_case(existing));
        self.fields.push((key, value));
    }
    /// Sets the body content type
    pub fn set_content_type<T>(&mut self, type_: T)
    where
        T: Into<Data>,
    {
        self.set_field("Content-Type", type_)
    }
    /// Sets the body content length
    pub fn set_content_length(&mut self, len: u64) {
        self.set_field("Content-Length", len.to_string())
    }
    /// Sets the connection header to `Close`
    pub fn set_connection_close(&mut self) {
        self.set_field("Connection", "Close")
    }

    /// Returns the content length if it is set
    pub fn content_length(&self) -> Result<Option<u64>, Error> {
        // Search for `Content-Length` header
        for (key, value) in &self.fields {
            if key.eq_ignore_ascii_case(b"Content-Length") {
                // Decode the value
                let value = str::from_utf8(value)?;
                let content_length: u64 = value.parse()?;
                return Ok(Some(content_length));
            }
        }
        Ok(None)
    }

    /// Sets the given data as body content and updates the `Content-Length` header accordingly
    pub fn set_body_data<T>(&mut self, data: T)
    where
        T: Into<Data>,
    {
        let data = data.into();
        self.set_content_length(data.len() as u64);
        self.body = Source::from(data);
    }
    /// Sets the given file as body content and updates the `Content-Length` header accordingly
    ///
    /// # Note
    /// Please note that this function also respects the file's current seek offset; so if you are at offset `7` out of
    /// `15`, the content length is set to `8`.
    pub fn set_body_file<T>(&mut self, file: T) -> Result<(), Error>
    where
        T: Into<File>,
    {
        // Get the current position and the total length
        let mut file = file.into();
        #[allow(clippy::seek_from_current)]
        let pos = file.seek(SeekFrom::Current(0))?;
        let len = file.seek(SeekFrom::End(0))?;

        // Recover the original position and set the length
        if pos != len {
            file.seek(SeekFrom::Start(pos))?;
        }
        self.set_content_length(len - pos);

        // Set the body
        self.body = BufReader::new(file).into();
        Ok(())
    }

    /// Turns the current `GET`-response into a `HEAD`-response by discarding the body without modifying content length
    /// etc.
    pub fn make_head(&mut self) {
        self.body = Source::default();
    }
}
