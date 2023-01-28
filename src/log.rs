//! A logging facility

use std::sync::atomic::{AtomicUsize, Ordering};

/// Will log everything
pub const ALL: usize = 0;
/// The log level for debug messages
pub const DEBUG: usize = 1;
/// The log level for info messages
pub const INFO: usize = 2;
/// The log level for warning messages
pub const WARN: usize = 3;
/// The log level for error messages
pub const ERROR: usize = 4;

/// The global log level
static LEVEL: AtomicUsize = AtomicUsize::new(3);
/// The global log level
pub fn level() -> usize {
    LEVEL.load(Ordering::SeqCst)
}
/// Sets the global log level
pub fn set_level(level: usize) {
    LEVEL.store(level, Ordering::SeqCst)
}

/// A logger to log messages with a specific level
#[doc(hidden)]
pub struct Logger<const LEVEL: usize>;
impl<const LEVEL: usize> std::fmt::Write for Logger<LEVEL> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        if LEVEL >= level() {
            eprint!("{s}");
        }
        Ok(())
    }
}

/// Logs a message at the given level
#[doc(hidden)]
#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)*) => {
        use std::fmt::Write;
        let mut logger = $crate::log::Logger::<{ $level }>;
        let _ = writeln!(&mut logger, $($arg)*);
    };
}

/// Logs a message at debug level
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => { $crate::log!($crate::log::DEBUG, $($arg)*) };
}
/// Logs a message at info level
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => { $crate::log!($crate::log::INFO, $($arg)*) };
}
/// Logs a message at warning level
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => { $crate::log!($crate::log::WARN, $($arg)*) };
}
/// Logs a message at error level
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => { $crate::log!($crate::log::ERROR, $($arg)*) };
}
