//! Extension traits for `http::Response`

use crate::{
    bytes::{Data, Source},
    error::Error,
    http::response::Response,
};

/// Some HTTP response extensions
pub trait ResponseExt
where
    Self: Sized,
{
    /// Creates a new HTTP response with the given status code and reason
    fn new_status_reason<T>(status: u16, reason: T) -> Self
    where
        T: Into<Vec<u8>>;
    /// Creates a new `200 OK` HTTP response
    fn new_200_ok() -> Self;
    /// Creates a new `403 Forbidden` HTTP response
    fn new_403_forbidden() -> Self;
    /// Creates a new `404 Not Found` HTTP response
    fn new_404_notfound() -> Self;

    /// Sets the field with the given name (performs an ASCII-case-insensitve comparison for replacement)
    fn set_field<K, V>(&mut self, key: K, value: V)
    where
        K: Into<Vec<u8>>,
        V: Into<Vec<u8>>;
    /// Sets the body content length
    fn set_content_length(&mut self, len: u64);
    /// Sets the connection header to `Close`
    fn set_connection_close(&mut self);

    /// Returns the content length if it is set
    fn content_length(&self) -> Result<Option<u64>, Error>;

    /// Sets the given file as body content
    ///
    /// Note: This also sets the `Content-Length` header if the body length can be determined automatically. Use
    /// `self.content_length` to check if the content length has been set. However, unless you use an opaque data source,
    /// you can assume that the body length can be determined automatically.
    fn set_body(&mut self, body: Source) -> Result<(), Error>;
}
impl<const HEADER_SIZE_MAX: usize> ResponseExt for Response<HEADER_SIZE_MAX> {
    fn new_status_reason<T>(status: u16, reason: T) -> Self
    where
        T: Into<Vec<u8>>,
    {
        let version = b"HTTP/1.1".to_vec();
        let status = status.to_string().into_bytes();
        let reason = reason.into();
        Self::new(Data::Vec(version), Data::Vec(status), Data::Vec(reason))
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
        K: Into<Vec<u8>>,
        V: Into<Vec<u8>>,
    {
        // Convert the field into vecs
        let key = key.into();
        let value = value.into();

        // Remove any field with the same name and set the field
        self.fields.retain(|(existing, _)| !key.eq_ignore_ascii_case(existing));
        self.fields.push((Data::Vec(key), Data::Vec(value)));
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

    fn set_body(&mut self, mut body: Source) -> Result<(), Error> {
        // Set the length if known
        if let Some(len) = body.get_len()? {
            self.set_content_length(len);
        }

        // Set the body
        self.body = body;
        Ok(())
    }
}
