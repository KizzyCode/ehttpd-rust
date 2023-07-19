//! Provides (mostly) stack-allocating trait implementors over different underlying sources

mod data;
mod dataext;
mod sink;
mod source;

pub use crate::bytes::{
    data::Data,
    dataext::{DataParseExt, DataSliceExt},
    sink::{AnySink, Sink},
    source::{AnySource, Source},
};
