//! A HTTP adapter

pub mod body;
pub mod request;
pub mod requestext;
pub mod response;
pub mod responseext;

pub use crate::http::{request::Request, response::Response};
