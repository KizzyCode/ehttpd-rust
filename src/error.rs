//! Implements the crate's error type

use std::backtrace::{Backtrace, BacktraceStatus};
use std::convert::Infallible;
use std::fmt::{self, Display, Formatter};
use std::num::ParseIntError;
use std::ops::Deref;
use std::str::Utf8Error;

/// Creates a new error
#[macro_export]
macro_rules! err {
    (with: $error:expr, $($arg:tt)*) => {{
        let error = format!($($arg)*);
        let source = Box::new($error);
        $crate::error::Error::new(error, Some(source))
    }};
    ($($arg:tt)*) => {{
        let error = format!($($arg)*);
        $crate::error::Error::new(error, None)
    }};
}

/// The crates error type
#[derive(Debug)]
pub struct Error {
    /// The error description
    pub error: String,
    /// The underlying error
    pub source: Option<Box<dyn std::error::Error + Send>>,
    /// The backtrace
    pub backtrace: Backtrace,
}
impl Error {
    /// Creates a new error
    #[doc(hidden)]
    pub fn new(error: String, source: Option<Box<dyn std::error::Error + Send>>) -> Self {
        let backtrace = Backtrace::capture();
        Self { error, source, backtrace }
    }

    /// Whether the error has captured a backtrace or not
    pub fn has_backtrace(&self) -> bool {
        self.backtrace.status() == BacktraceStatus::Captured
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // Print the error
        writeln!(f, "{}", self.error)?;

        // Print the source
        if let Some(source) = &self.source {
            writeln!(f, " caused by: {source}")?;
        }
        Ok(())
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        let boxed = self.source.as_ref()?;
        Some(boxed.deref())
    }
}
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        err!(with: value, "An I/O error occurred")
    }
}
impl From<Utf8Error> for Error {
    fn from(value: Utf8Error) -> Self {
        err!(with: value, "Value is not valid UTF-8")
    }
}
impl From<ParseIntError> for Error {
    fn from(value: ParseIntError) -> Self {
        err!(with: value, "Value is not a valid integer")
    }
}
impl From<Infallible> for Error {
    fn from(_: Infallible) -> Self {
        unreachable!("infallible variant can never be constructed")
    }
}
