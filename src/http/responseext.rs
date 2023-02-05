//! Extension traits for `http::Response`

use crate::{
    bytes::{Data, Source},
    error::Error,
    http::response::Response,
};
use std::{
    borrow::BorrowMut,
    fs::File,
    io::{Seek, SeekFrom},
};

/// Some HTTP response extensions
pub trait ResponseExt
where
    Self: Sized,
{
    /// Creates a new HTTP response with the given status code and reason
    fn new_status_reason<T>(status: u16, reason: T) -> Self
    where
        T: Into<Data>;
    /// Creates a new `200 OK` HTTP response
    fn new_200_ok() -> Self;
    /// Creates a new `403 Forbidden` HTTP response
    fn new_403_forbidden() -> Self;
    /// Creates a new `404 Not Found` HTTP response
    fn new_404_notfound() -> Self;

    /// Sets the field with the given name (performs an ASCII-case-insensitve comparison for replacement)
    fn set_field<K, V>(&mut self, key: K, value: V)
    where
        K: Into<Data>,
        V: Into<Data>;
    /// Sets the body content length
    fn set_content_length(&mut self, len: u64);
    /// Sets the connection header to `Close`
    fn set_connection_close(&mut self);

    /// Returns the content length if it is set
    fn content_length(&self) -> Result<Option<u64>, Error>;

    /// Sets the given file as body content
    ///
    /// # Note
    /// This function **DOES NOT** set the "Content-Length" header, it's up to you to set it manually
    fn set_body(&mut self, body: Source);
    /// Sets the given data as body content and updates the `Content-Length` header accordingly
    fn set_body_data<T>(&mut self, data: T)
    where
        T: Into<Data>;
    /// Sets the given file as body content and updates the `Content-Length` header accordingly
    ///
    /// # Note
    /// Please note that this function also respects the file's current seek offset; so if you are at offset `7` out of
    /// `15`, the content length is set to `8`.
    fn set_body_file<T>(&mut self, file: T) -> Result<(), Error>
    where
        T: Into<Source> + BorrowMut<File>;
}
impl<const HEADER_SIZE_MAX: usize> ResponseExt for Response<HEADER_SIZE_MAX> {
    fn new_status_reason<T>(status: u16, reason: T) -> Self
    where
        T: Into<Data>,
    {
        let version = Data::from(b"HTTP/1.1");
        let status = Data::from(status.to_string());
        let reason = reason.into();
        Self::new(version, status, reason)
    }
    fn new_200_ok() -> Self {
        Self::new_status_reason(200, "OK")
    }
    fn new_403_forbidden() -> Self {
        Self::new_status_reason(403, "Forbidden")
    }
    fn new_404_notfound() -> Self {
        Self::new_status_reason(404, "Not Found")
    }

    fn set_field<K, V>(&mut self, key: K, value: V)
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
    fn set_content_length(&mut self, len: u64) {
        self.set_field("Content-Length", len.to_string())
    }
    fn set_connection_close(&mut self) {
        self.set_field("Connection", "Close")
    }

    fn content_length(&self) -> Result<Option<u64>, Error> {
        // Search for `Content-Length` header
        for (key, value) in &self.fields {
            if key.eq_ignore_ascii_case(b"Content-Length") {
                // Decode the value
                let value = std::str::from_utf8(value)?;
                let content_length: u64 = value.parse()?;
                return Ok(Some(content_length));
            }
        }
        Ok(None)
    }

    fn set_body(&mut self, body: Source) {
        self.body = body;
    }
    fn set_body_data<T>(&mut self, data: T)
    where
        T: Into<Data>,
    {
        let data = data.into();
        self.set_content_length(data.len() as u64);
        self.set_body(Source::from(data))
    }
    fn set_body_file<T>(&mut self, mut file: T) -> Result<(), Error>
    where
        T: Into<Source> + BorrowMut<File>,
    {
        // Get the current position and the total length
        let file_real = file.borrow_mut();
        #[allow(clippy::seek_from_current)]
        let pos = file_real.seek(SeekFrom::Current(0))?;
        let len = file_real.seek(SeekFrom::End(0))?;

        // Recover the original position and set the length
        if pos != len {
            file_real.seek(SeekFrom::Start(pos))?;
        }
        self.set_content_length(len - pos);

        // Set the body
        let file = file.into();
        self.set_body(file);
        Ok(())
    }
}
