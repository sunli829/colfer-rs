//! `Colfer` is a binary serialization format optimized for speed and size, this crate
//! is a Rust implementation of the [colfer](https://github.com/pascaldekloe/colfer).

#![warn(missing_docs)]

mod datetime;
mod types;

#[doc(hidden)]
pub use bytes;
use bytes::{Buf, BufMut};
pub use datetime::DateTime;
pub use types::{decode_message, decode_messages, encode_message, encode_messages, Type};

/// `MAX_SIZE` is the upper limit for serial byte sizes.
pub const MAX_SIZE: usize = 16 * 1024 * 1024;

/// `MAX_LIST_SIZE` is the upper limit for the number of elements in a list.
pub const MAX_LIST_SIZE: usize = 64 * 1024;

/// A colfer message.
pub trait Message: Sized {
    /// Encodes the message to writer `W`.
    fn encode<B: BufMut>(&self, buf: &mut B);

    /// Decodes an instance of the message from reader `R`.
    fn decode<B: Buf>(buf: B) -> Self;

    /// Returns the encoded length of the message.
    fn size(&self) -> usize;

    /// Encodes the message to `Vec<u8>`.
    fn to_vec(&self) -> Vec<u8> {
        let mut data = Vec::new();
        self.encode(&mut data);
        data
    }

    /// Decodes an instance of the message from `Vec<u8>`.
    fn from_bytes(data: &[u8]) -> Self {
        Self::decode(data)
    }
}

#[inline]
#[doc(hidden)]
pub fn read_header<B: Buf>(mut buf: B) -> (u8, bool) {
    let d = buf.get_u8();
    (d & 0x7f, d & 0x80 > 0)
}

#[inline]
#[doc(hidden)]
pub fn write_end<B: BufMut>(buf: &mut B) {
    buf.put_u8(0x7f)
}
