//! Provides (mostly) stack-allocating trait implementors over different underlying sources

mod data;
mod parse;
mod sink;
mod source;

pub use crate::bytes::data::Data;
pub use crate::bytes::parse::Parse;
pub use crate::bytes::sink::Sink;
pub use crate::bytes::source::Source;
