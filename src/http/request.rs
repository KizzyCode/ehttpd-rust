//! A HTTP request

use crate::{
    error,
    error::Error,
    utils::{
        rcvec::RcVec,
        rcvecext::{RcVecExt, RcVecU8Ext},
    },
};
use std::{
    io::{BufReader, Read},
    net::TcpStream,
};

/// A HTTP request
#[derive(Debug)]
pub struct Request<T = BufReader<TcpStream>, const HEADER_SIZE_MAX: usize = 4096> {
    /// The raw header bytes
    pub header: RcVec<u8>,
    /// The range of the method part within the request line
    pub method: RcVec<u8>,
    /// The range of the target part within the request line
    pub target: RcVec<u8>,
    /// The range of the version part within the request line
    pub version: RcVec<u8>,
    /// The ranges of the key/value fields within the header
    pub fields: Vec<(RcVec<u8>, RcVec<u8>)>,
    /// The connection stream
    pub stream: T,
}
impl<T, const HEADER_SIZE_MAX: usize> Request<T, HEADER_SIZE_MAX> {
    /// Reads a HTTP request from a readable `stream`
    pub fn from_stream(mut stream: T) -> Result<Option<Self>, Error>
    where
        T: Read,
    {
        // Read the raw header or return `None` if the connection has been closed
        let header = Self::read_header(&mut stream)?;
        if header.is_empty() {
            return Ok(None);
        }

        // Parse the start line
        let mut header_parsing = header.clone();
        let (method, target, version) = {
            let (method, target, version) = Self::parse_start_line(&mut header_parsing)?;
            (method.trim(), target.trim(), version.trim())
        };

        // Parse the fields
        let mut fields = Vec::new();
        while !header_parsing.eq(b"\r\n") {
            // Parse field
            let (key, value) = Self::parse_field(&mut header_parsing)?;
            fields.push((key, value));
        }

        Ok(Some(Self { header, method, target, version, fields, stream }))
    }

    /// Reads the entire HTTP header from the stream
    fn read_header(stream: &mut T) -> Result<RcVec<u8>, Error>
    where
        T: Read,
    {
        // Read the header
        let mut header = Vec::with_capacity(HEADER_SIZE_MAX);
        'read_loop: for byte in stream.bytes() {
            // Read the next byte
            let byte = byte?;
            header.push(byte);

            // Check if we have the header
            if header.ends_with(b"\r\n\r\n") {
                break 'read_loop;
            }
            if header.len() == HEADER_SIZE_MAX {
                return Err(error!("HTTP header is too large"));
            }
        }

        // Create the RcVec
        header.shrink_to_fit();
        let header = RcVec::new(header);
        Ok(header)
    }
    /// Parses the start line
    #[allow(clippy::type_complexity)]
    fn parse_start_line(header: &mut RcVec<u8>) -> Result<(RcVec<u8>, RcVec<u8>, RcVec<u8>), Error> {
        // Split the header line
        let mut line = header.split_off(b"\r\n").ok_or(error!("Truncated HTTP start line: {header}"))?;
        let method = line.split_off(b" ").ok_or(error!("Invalid HTTP start line: {line}"))?;
        let target = line.split_off(b" ").ok_or(error!("Invalid HTTP start line: {line}"))?;
        Ok((method, target, line))
    }
    /// Parses a header field
    fn parse_field(header: &mut RcVec<u8>) -> Result<(RcVec<u8>, RcVec<u8>), Error> {
        // Parse the field
        let mut line = header.split_off(b"\r\n").ok_or(error!("Truncated HTTP header field: {header}"))?;
        let key = line.split_off(b":").ok_or(error!("Invalid HTTP header field: {line}"))?;

        // Trim the field values
        let key = key.trim();
        let value = line.trim();
        Ok((key, value))
    }
}
