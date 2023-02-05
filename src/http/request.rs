//! A HTTP request

use crate::{
    bytes::{Data, DataParseExt, Source},
    error,
    error::Error,
};
use std::io::Read;

/// A HTTP request
#[derive(Debug)]
pub struct Request<'a, const HEADER_SIZE_MAX: usize = 4096> {
    /// The raw header bytes
    pub header: Data,
    /// The range of the method part within the request line
    pub method: Data,
    /// The range of the target part within the request line
    pub target: Data,
    /// The range of the version part within the request line
    pub version: Data,
    /// The ranges of the key/value fields within the header
    pub fields: Vec<(Data, Data)>,
    /// The connection stream
    pub stream: &'a mut Source,
}
impl<'a, const HEADER_SIZE_MAX: usize> Request<'a, HEADER_SIZE_MAX> {
    /// Reads a HTTP request from a readable `stream`
    pub fn from_stream(stream: &'a mut Source) -> Result<Option<Self>, Error> {
        // Read the raw header or return `None` if the connection has been closed
        let header = Self::read_header(stream)?;
        if header.is_empty() {
            return Ok(None);
        }

        // Parse the start line
        let mut header_parsing = header.clone();
        let (method, target, version) = {
            let (method, target, version) = Self::parse_start_line(&mut header_parsing)?;
            (method.trimmed(), target.trimmed(), version.trimmed())
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
    fn read_header(stream: &mut Source) -> Result<Data, Error> {
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
        let header = Data::new_arcvec(header);
        Ok(header)
    }
    /// Parses the start line
    #[allow(clippy::type_complexity)]
    fn parse_start_line(header: &mut Data) -> Result<(Data, Data, Data), Error> {
        // Split the header line
        let mut line = header.split_off(b"\r\n").ok_or_else(|| error!("Truncated HTTP start line: {header}"))?;
        let method = line.split_off(b" ").ok_or_else(|| error!("Invalid HTTP start line: {line}"))?;
        let target = line.split_off(b" ").ok_or_else(|| error!("Invalid HTTP start line: {line}"))?;
        Ok((method, target, line))
    }
    /// Parses a header field
    fn parse_field(header: &mut Data) -> Result<(Data, Data), Error> {
        // Parse the field
        let mut line = header.split_off(b"\r\n").ok_or_else(|| error!("Truncated HTTP header field: {header}"))?;
        let key = line.split_off(b":").ok_or_else(|| error!("Invalid HTTP header field: {line}"))?;

        // Trim the field values
        let key = key.trimmed();
        let value = line.trimmed();
        Ok((key, value))
    }
}
