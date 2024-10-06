#![doc = include_str!("../README.md")]

pub mod bytes;
pub mod error;
pub mod http;
pub mod server;

// Re-export the server for convenience if enabled
#[cfg(feature = "server")]
pub use crate::server::*;
