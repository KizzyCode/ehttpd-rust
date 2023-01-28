//! Extension traits for `RcVec`

use crate::utils::rcvec::RcVec;

/// Extensions for `RcVec`
pub trait RcVecExt<T> {
    /// Splits `self` on the first occurrence of `pat` and returns prefix before delimiter; `self` is updated to hold the
    /// remaining suffix after `pat`
    fn split_off(&mut self, pat: &[T]) -> Option<RcVec<T>>
    where
        T: PartialEq;
}
impl<T> RcVecExt<T> for RcVec<T> {
    fn split_off(&mut self, pat: &[T]) -> Option<RcVec<T>>
    where
        T: PartialEq,
    {
        for (offset, haystack) in self.windows(pat.len()).enumerate() {
            // Check for match
            if haystack == pat {
                let split = self.subvec(..offset).expect("invalid prefix offset");
                *self = self.subvec(offset + pat.len()..).expect("invalid suffix offset");
                return Some(split);
            }
        }
        None
    }
}

/// Extensions for `RcVec<u8>`
pub trait RcVecU8Ext {
    /// Trims leading and trailing ASCII whitespaces
    fn trim(&self) -> RcVec<u8>;
}
impl RcVecU8Ext for RcVec<u8> {
    fn trim(&self) -> RcVec<u8> {
        // Trim the leading bytes
        let leading = self.iter().take_while(|byte| byte.is_ascii_whitespace()).count();
        let trimmed = self.subvec(leading..).expect("invalid segment range");

        // Trim the trailing bytes
        let trailing = trimmed.iter().rev().take_while(|byte| byte.is_ascii_whitespace()).count();
        trimmed.subvec(..trimmed.len() - trailing).expect("invalid segment range")
    }
}
