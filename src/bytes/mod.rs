//! Provides (mostly) stack-allocating trait implementors over different underlying sources

mod data;
mod dataext;
mod source;

pub use crate::bytes::{
    data::Data,
    dataext::{DataParseExt, DataSliceExt},
    source::Source,
};
