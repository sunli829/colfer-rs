//! `Colfer` is a binary serialization format optimized for speed and size, this crate
//! is a Rust implementation of the [colfer](https://github.com/pascaldekloe/colfer).

#![warn(missing_docs)]
#![forbid(unsafe_code)]

mod datetime;
mod types;

use std::io::{Cursor, Read, Result, Write};

use byteorder::ReadBytesExt;
pub use datetime::DateTime;
pub use types::{decode_message, decode_messages, encode_message, encode_messages, Type};

/// `MAX_SIZE` is the upper limit for serial byte sizes.
pub const MAX_SIZE: usize = 16 * 1024 * 1024;

/// `MAX_LIST_SIZE` is the upper limit for the number of elements in a list.
pub const MAX_LIST_SIZE: usize = 64 * 1024;

/// A colfer message.
pub trait Message: Sized {
    /// Encodes the message to writer `W`.
    fn encode<W: Write>(&self, w: &mut W) -> Result<()>;

    /// Decodes an instance of the message from reader `R`.
    fn decode<R: Read>(r: &mut R) -> Result<Self>;

    /// Returns the encoded length of the message.
    fn size(&self) -> usize;

    /// Encodes the message to `Vec<u8>`.
    fn to_vec(&self) -> Result<Vec<u8>> {
        let mut data = Vec::new();
        self.encode(&mut data)?;
        Ok(data)
    }

    /// Decodes an instance of the message from `Vec<u8>`.
    fn from_bytes(data: &[u8]) -> Result<Self> {
        Self::decode(&mut Cursor::new(data))
    }
}

#[inline]
#[doc(hidden)]
pub fn read_header<R: Read>(r: &mut R) -> Result<(u8, bool)> {
    let d = r.read_u8()?;
    Ok((d & 0x7f, d & 0x80 > 0))
}
