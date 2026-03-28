//! Some useful extensions for data types

use crate::bytes::Data;
use crate::err;
use crate::error::Error;
use std::str::FromStr;

/// Some parsing extensions
pub trait Parse
where
    Self: Sized,
{
    /// Splits `self` on the first occurrence of `pat` and returns prefix before delimiter; `self` is updated to hold
    /// the remaining suffix after `pat`
    ///
    /// # Note
    /// This method uses the cheapest way to clone the data by e.g. performing an `Rc::clone` on `Self::RcVec`
    fn split_off(&mut self, pat: &[u8]) -> Option<Self>;
    /// Trims leading and trailing ASCII whitespaces
    #[must_use]
    fn trim(&self) -> Self;

    /// Parses `self` via the [`FromStr`]-trait
    fn parse<Type>(&self) -> Result<Type, Error>
    where
        Type: FromStr,
        Type::Err: std::error::Error + Send + 'static;
}
impl Parse for Data {
    fn split_off(&mut self, pat: &[u8]) -> Option<Self> {
        for (offset, haystack) in self.windows(pat.len()).enumerate() {
            // Check for match
            if haystack == pat {
                let split = self.subcopy(..offset).expect("invalid prefix offset");
                *self = self.subcopy(offset + pat.len()..).expect("invalid suffix offset");
                return Some(split);
            }
        }
        None
    }
    fn trim(&self) -> Self {
        // Trim the leading bytes
        let leading = self.iter().take_while(|byte| byte.is_ascii_whitespace()).count();
        let trimmed = self.subcopy(leading..).expect("invalid segment range");

        // Trim the trailing bytes
        let trailing = trimmed.iter().rev().take_while(|byte| byte.is_ascii_whitespace()).count();
        trimmed.subcopy(..trimmed.len() - trailing).expect("invalid segment range")
    }

    fn parse<Type>(&self) -> Result<Type, Error>
    where
        Type: FromStr,
        Type::Err: std::error::Error + Send + 'static,
    {
        // Forward to &[u8] implementation
        self.as_ref().parse()
    }
}
impl Parse for &[u8] {
    fn split_off(&mut self, pat: &[u8]) -> Option<Self> {
        for (offset, haystack) in self.windows(pat.len()).enumerate() {
            // Check for match
            if haystack == pat {
                let split = &self[..offset];
                *self = &self[offset + pat.len()..];
                return Some(split);
            }
        }
        None
    }
    fn trim(&self) -> Self {
        // We can just use the slice implementation
        self.trim_ascii()
    }

    fn parse<Type>(&self) -> Result<Type, Error>
    where
        Type: FromStr,
        Type::Err: std::error::Error + Send + 'static,
    {
        // Parse data literal
        let str_ = str::from_utf8(self)?;
        str_.parse::<Type>().map_err(|e| err!(with: e, "failed to parse data"))
    }
}
