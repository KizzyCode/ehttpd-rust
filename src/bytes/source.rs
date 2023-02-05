//! A readable data source

use crate::{bytes::data::Data, error::Error};
use std::{
    any::Any,
    cmp,
    fmt::{Debug, Formatter},
    fs::File,
    io::{BufReader, Cursor, Read, Seek, SeekFrom},
    net::TcpStream,
};

/// A stack-allocating type-abstract readable data source
#[non_exhaustive]
pub enum Source {
    /// An empty source
    Empty,
    /// A linear data backed source
    Data(Cursor<Data>),
    /// A file
    File(File),
    /// A buffered file
    BufferedFile(BufReader<File>),
    /// A TCP stream
    TcpStream(TcpStream),
    /// A buffered TCP stream
    BufferedTcpStream(BufReader<TcpStream>),
    /// A catch-all/opaque variant for all types that cannot be covered by the enum's specific variants
    ///
    /// # Note
    /// In general, this variant should not be created "by hand"; use `Self::new_other` instead
    Other {
        /// The underlying data backing
        data: Box<dyn Any + Send>,
        /// A pointer to type-specific implementation to recover the original type and coerce it to `&mut dyn Read`
        #[doc(hidden)]
        as_read_mut: fn(&mut Box<dyn Any + Send>) -> &mut dyn Read,
        /// A pointer to type-specific implementation to recover the original type and coerce it to `&dyn Debug`
        #[doc(hidden)]
        as_debug: fn(&Box<dyn Any + Send>) -> &dyn Debug,
    },
}
impl Source {
    /// Creates a new catch-all/opaque variant from a typed object by moving it to the heap
    pub fn new_other<T>(typed: T) -> Self
    where
        T: Read + Debug + Send + 'static,
    {
        /// The specific implementation to recover `&dyn Any` as `&T` and coerce it to `&dyn AsRef<[u8]>`
        fn as_read_mut<T>(untyped: &mut Box<dyn Any + Send>) -> &mut dyn Read
        where
            T: Read + 'static,
        {
            let typed: &mut T = untyped.downcast_mut().expect("failed to recover type");
            typed
        }
        /// The specific implementation to recover `&dyn Any` as `&T` and coerce it to `&dyn Debug`
        fn as_debug<T>(untyped: &Box<dyn Any + Send>) -> &dyn Debug
        where
            T: Debug + 'static,
        {
            let typed: &T = untyped.downcast_ref().expect("failed to recover type");
            typed
        }

        // Box the value and init self
        let untyped: Box<dyn Any + Send> = Box::new(typed);
        Self::Other { data: untyped, as_read_mut: as_read_mut::<T>, as_debug: as_debug::<T> }
    }

    /// Skips the given amount of bytes by reading and discarding them
    ///
    /// # Note
    /// This function should only be used for non-seekable sources since reading all the bytes is ususally much less
    /// efficient than just seeking
    pub fn skip(&mut self, mut skip: usize) -> std::io::Result<()> {
        // A 16k buffer for efficient skipping
        let mut buf = vec![0; 16 * 1024];
        while skip > 0 {
            // Read/discard bytes
            let to_read = cmp::min(skip, buf.len());
            self.read_exact(&mut buf[..to_read])?;
            skip -= to_read;
        }
        Ok(())
    }

    /// Tries to get the data source length if possible
    pub fn get_len(&mut self) -> Result<Option<u64>, Error> {
        /// Returns the length for `Seek`able types
        fn seek_remaining<T>(seeker: &mut T) -> Result<u64, Error>
        where
            T: Seek,
        {
            // Get the current position and the total length
            #[allow(clippy::seek_from_current)]
            let pos = seeker.seek(SeekFrom::Current(0))?;
            let len = seeker.seek(SeekFrom::End(0))?;

            // Recover the original position
            if pos != len {
                seeker.seek(SeekFrom::Start(pos))?;
            }
            Ok(len - pos)
        }

        // Get the remaining length
        match self {
            Source::Empty => Ok(Some(0)),
            Source::Data(data) => {
                let remaining = seek_remaining(data)?;
                Ok(Some(remaining))
            }
            Source::File(file) => {
                let remaining = seek_remaining(file)?;
                Ok(Some(remaining))
            }
            Source::BufferedFile(buffered_file) => {
                let remaining = seek_remaining(buffered_file)?;
                let buffered = buffered_file.buffer().len() as u64;
                Ok(Some(buffered + remaining))
            }
            Source::TcpStream(_) => Ok(None),
            Source::BufferedTcpStream(_) => Ok(None),
            Source::Other { .. } => Ok(None),
        }
    }
}
impl Read for Source {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Source::Empty => Ok(0),
            Source::Data(data) => data.read(buf),
            Source::File(file) => file.read(buf),
            Source::BufferedFile(buffered_file) => buffered_file.read(buf),
            Source::TcpStream(tcp_stream) => tcp_stream.read(buf),
            Source::BufferedTcpStream(buffered_tcp_stream) => buffered_tcp_stream.read(buf),
            Source::Other { data, as_read_mut, .. } => as_read_mut(data).read(buf),
        }
    }
}
impl Debug for Source {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::Empty => f.debug_tuple("Empty").finish(),
            Self::Data(arg0) => f.debug_tuple("Data").field(arg0).finish(),
            Self::File(arg0) => f.debug_tuple("File").field(arg0).finish(),
            Self::BufferedFile(arg0) => f.debug_tuple("BufferedFile").field(arg0).finish(),
            Self::TcpStream(arg0) => f.debug_tuple("TcpStream").field(arg0).finish(),
            Self::BufferedTcpStream(arg0) => f.debug_tuple("BufferedTcpStream").field(arg0).finish(),
            Self::Other { data, as_debug, .. } => f.debug_struct("Other").field("data", as_debug(data)).finish(),
        }
    }
}
impl Default for Source {
    fn default() -> Self {
        Self::Empty
    }
}
impl From<Data> for Source {
    fn from(value: Data) -> Self {
        Self::Data(Cursor::new(value))
    }
}
impl From<Cursor<Data>> for Source {
    fn from(value: Cursor<Data>) -> Self {
        Self::Data(value)
    }
}
impl From<Vec<u8>> for Source {
    fn from(value: Vec<u8>) -> Self {
        Self::from(Data::Vec(value))
    }
}
impl From<&'static [u8]> for Source {
    fn from(value: &'static [u8]) -> Self {
        Self::from(Data::Static(value))
    }
}
impl<const SIZE: usize> From<&'static [u8; SIZE]> for Source {
    fn from(value: &'static [u8; SIZE]) -> Self {
        Self::from(Data::Static(value))
    }
}
impl From<String> for Source {
    fn from(value: String) -> Self {
        Self::from(Data::Vec(value.into_bytes()))
    }
}
impl From<&'static str> for Source {
    fn from(value: &'static str) -> Self {
        Self::from(Data::Static(value.as_bytes()))
    }
}
impl From<File> for Source {
    fn from(value: File) -> Self {
        Self::File(value)
    }
}
impl From<BufReader<File>> for Source {
    fn from(value: BufReader<File>) -> Self {
        Self::BufferedFile(value)
    }
}
impl From<TcpStream> for Source {
    fn from(value: TcpStream) -> Self {
        Self::TcpStream(value)
    }
}
impl From<BufReader<TcpStream>> for Source {
    fn from(value: BufReader<TcpStream>) -> Self {
        Self::BufferedTcpStream(value)
    }
}
