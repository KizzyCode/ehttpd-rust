//! Extension traits for `http::Request`

use crate::{bytes::Data, error::Error, http::Request};
use std::{path::Path, str};

/// Some HTTP request extensions
pub trait RequestExt {
    /// Gets the request target as path
    ///
    /// # Important
    /// On non-unix platforms, this function uses a `str` as intermediate representation, so the path must be valid UTF-8.
    /// If this might be a problem, you should use the raw target field directly.
    fn target_path(&self) -> Option<&Path>;

    /// Gets the field with the given name (performs an ASCII-case-insensitve comparison)
    fn field<T>(&self, name: T) -> Option<&Data>
    where
        T: AsRef<[u8]>;
    /// The request content length field if any
    fn content_length(&self) -> Result<Option<u64>, Error>;
}
impl<'a, const HEADER_SIZE_MAX: usize> RequestExt for Request<'a, HEADER_SIZE_MAX> {
    #[cfg(target_family = "unix")]
    fn target_path(&self) -> Option<&Path> {
        use std::{ffi::OsStr, os::unix::ffi::OsStrExt};

        // Create the path directly without going via `str`
        let target = OsStr::from_bytes(&self.target);
        Some(Path::new(target))
    }
    #[cfg(not(any(target_family = "unix")))]
    fn target_path(&self) -> Option<&Path> {
        // Convert the target to UTF-8 and return it as string
        let target = str::from_utf8(&self.target).ok()?;
        Some(Path::new(target))
    }

    fn field<N>(&self, name: N) -> Option<&Data>
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
    fn content_length(&self) -> Result<Option<u64>, Error> {
        // Get the content length field if set
        let Some(content_length_raw) = self.field("Content-Length") else {
            return Ok(None)
        };

        // Parse the field
        let content_length_utf8 = str::from_utf8(content_length_raw)?;
        let content_length: u64 = content_length_utf8.parse()?;
        Ok(Some(content_length))
    }
}
