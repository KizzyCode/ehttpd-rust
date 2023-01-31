//! Extension traits for `http::Response`

use crate::{
    error::Error,
    http::{body::Body, response::Response},
    utils::rcvec::RcVec,
};
use std::{fs::File, io::Cursor};

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
}
impl ResponseExt for Response<Body> {
    fn new_status_reason<R>(status: u16, reason: R) -> Self
    where
        R: Into<Vec<u8>>,
    {
        let version = b"HTTP/1.1".to_vec();
        let status = status.to_string().into_bytes();
        let reason = reason.into();
        Self::new(RcVec::new(version), RcVec::new(status), RcVec::new(reason))
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
        self.fields.retain(|(key, _)| !key.eq_ignore_ascii_case(key));
        self.fields.push((RcVec::new(key), RcVec::new(value)));
    }
    fn set_content_length(&mut self, len: u64) {
        self.set_field("Content-Length", len.to_string())
    }
    fn set_connection_close(&mut self) {
        self.set_field("Connection", "Close")
    }
}

/// File-system related extensions for `http::Response`
pub trait ResponseBodyExt {
    /// Sets the given file as body content
    ///
    /// Note: This also sets the `Content-Length` header
    fn set_body_file(&mut self, path: &str) -> Result<(), Error>;
    /// Sets the given data as body contents
    ///
    /// Note: This also sets the `Content-Length` header
    fn set_body_data(&mut self, data: Vec<u8>);
    /// Sets the given static data as body contents
    ///
    /// Note: This also sets the `Content-Length` header
    fn set_body_static(&mut self, data: &'static [u8]);
}
impl ResponseBodyExt for Response<Body> {
    fn set_body_file(&mut self, path: &str) -> Result<(), Error> {
        // Open the file and get it's size
        let file = File::open(path)?;
        let metadata = file.metadata()?;

        // Set the content length and file as body
        self.set_content_length(metadata.len());
        self.body = Body::File(file);
        Ok(())
    }
    fn set_body_data(&mut self, data: Vec<u8>) {
        self.set_content_length(data.len() as u64);
        self.body = Body::Data(Cursor::new(data));
    }
    fn set_body_static(&mut self, data: &'static [u8]) {
        self.set_content_length(data.len() as u64);
        self.body = Body::Static(Cursor::new(data));
    }
}
