//! Provides (mostly) stack-allocating trait implementors over different underlying sources

mod data;
mod parseext;
mod sink;
mod source;

pub use crate::bytes::data::Data;
pub use crate::bytes::parseext::ParseExt;
pub use crate::bytes::sink::Sink;
pub use crate::bytes::source::Source;
