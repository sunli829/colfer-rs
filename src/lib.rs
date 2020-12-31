use std::io::{Cursor, Error, ErrorKind, Read, Result, Write};

use byteorder::{ReadBytesExt, WriteBytesExt, BE};
use std::ops::Deref;

/// `MAX_SIZE` is the upper limit for serial byte sizes.
pub const MAX_SIZE: usize = 16 * 1024 * 1024;

/// `MAX_LIST_SIZE` is the upper limit for the number of elements in a list.
pub const MAX_LIST_SIZE: usize = 64 * 1024;

#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
pub struct DateTime {
    pub seconds: i64,
    pub nano_seconds: u32,
}

#[inline]
fn write_uint<W: Write>(w: &mut W, mut x: u64) -> Result<()> {
    while x >= 0x80 {
        w.write_u8((x | 0x80) as u8)?;
        x >>= 7;
    }
    w.write_u8(x as u8)?;
    Ok(())
}

#[inline]
fn read_uint<R: Read>(r: &mut R) -> Result<u64> {
    let mut x = r.read_u8()? as u64;
    if x >= 0x80 {
        x &= 0x7f;
        let mut shift = 7;
        loop {
            let b = r.read_u8()? as u64;
            if b < 0x80 || shift == 56 {
                x |= b << shift;
                break;
            }
            x |= (b & 0x7f) << shift;
            shift += 7;
        }
    }
    Ok(x)
}

#[inline]
fn uint_size(mut x: u64) -> usize {
    let mut l = 1;
    while x >= 0x80 {
        x >>= 7;
        l += 1;
    }
    l
}

#[inline]
#[doc(hidden)]
pub fn read_header<R: Read>(r: &mut R) -> Result<(u8, bool)> {
    let d = r.read_u8()?;
    Ok((d & 0x7f, d & 0x80 > 0))
}

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

#[doc(hidden)]
pub trait Type: Sized {
    fn encode<W: Write>(&self, w: &mut W, id: u8) -> Result<()>;

    fn decode<R: Read>(r: &mut R, flag: bool) -> Result<Self>;

    fn size(&self) -> usize;
}

impl Type for bool {
    fn encode<W: Write>(&self, w: &mut W, id: u8) -> Result<()> {
        if *self {
            w.write_u8(id)?;
        }
        Ok(())
    }

    fn decode<R: Read>(_r: &mut R, _flag: bool) -> Result<Self> {
        Ok(true)
    }

    fn size(&self) -> usize {
        if *self {
            1
        } else {
            0
        }
    }
}

impl Type for u32 {
    fn encode<W: Write>(&self, w: &mut W, id: u8) -> Result<()> {
        if *self >= 1 << 21 {
            w.write_u8(id | 0x80)?;
            w.write_u32::<BE>(*self)?;
        } else if *self != 0 {
            w.write_u8(id)?;
            write_uint(w, *self as u64)?;
        }
        Ok(())
    }

    fn decode<R: Read>(r: &mut R, flag: bool) -> Result<Self> {
        if !flag {
            Ok(read_uint(r)? as u32)
        } else {
            r.read_u32::<BE>()
        }
    }

    fn size(&self) -> usize {
        if *self >= 1 << 21 {
            5
        } else if *self != 0 {
            1 + uint_size(*self as u64)
        } else {
            0
        }
    }
}

impl Type for u64 {
    fn encode<W: Write>(&self, w: &mut W, id: u8) -> Result<()> {
        if *self >= 1 << 49 {
            w.write_u8(id | 0x80)?;
            w.write_u64::<BE>(*self)?;
        } else if *self != 0 {
            w.write_u8(id)?;
            write_uint(w, *self as u64)?;
        }
        Ok(())
    }

    fn decode<R: Read>(r: &mut R, flag: bool) -> Result<Self> {
        if !flag {
            read_uint(r)
        } else {
            r.read_u64::<BE>()
        }
    }

    fn size(&self) -> usize {
        if *self >= 1 << 49 {
            9
        } else {
            1 + uint_size(*self)
        }
    }
}

impl Type for i32 {
    fn encode<W: Write>(&self, w: &mut W, id: u8) -> Result<()> {
        (*self as i64).encode(w, id)
    }

    fn decode<R: Read>(r: &mut R, flag: bool) -> Result<Self> {
        Ok(i64::decode(r, flag)? as i32)
    }

    fn size(&self) -> usize {
        (*self as i64).size()
    }
}

impl Type for i64 {
    fn encode<W: Write>(&self, w: &mut W, id: u8) -> Result<()> {
        if *self != 0 {
            let mut x = *self as u32;
            if *self >= 0 {
                w.write_u8(id)?;
            } else {
                x = !x + 1;
                w.write_u8(id | 0x80)?;
            }
            write_uint(w, x as u64)?;
        }
        Ok(())
    }

    fn decode<R: Read>(r: &mut R, flag: bool) -> Result<Self> {
        if !flag {
            Ok(read_uint(r)? as i64)
        } else {
            Ok((!read_uint(r)? + 1) as i64)
        }
    }

    fn size(&self) -> usize {
        if *self >= 0 {
            1 + uint_size(*self as u64)
        } else {
            1 + uint_size((!*self + 1) as u64)
        }
    }
}

impl Type for f32 {
    fn encode<W: Write>(&self, w: &mut W, id: u8) -> Result<()> {
        if *self != 0.0 {
            w.write_u8(id)?;
            w.write_u32::<BE>(self.to_bits())?;
        }
        Ok(())
    }

    fn decode<R: Read>(r: &mut R, _flag: bool) -> Result<Self> {
        Ok(f32::from_bits(r.read_u32::<BE>()?))
    }

    fn size(&self) -> usize {
        if *self != 0.0 {
            1 + 4
        } else {
            0
        }
    }
}

impl Type for f64 {
    fn encode<W: Write>(&self, w: &mut W, id: u8) -> Result<()> {
        if *self != 0.0 {
            w.write_u8(id)?;
            w.write_u64::<BE>(self.to_bits())?;
        }
        Ok(())
    }

    fn decode<R: Read>(r: &mut R, _flag: bool) -> Result<Self> {
        Ok(f64::from_bits(r.read_u64::<BE>()?))
    }

    fn size(&self) -> usize {
        if *self != 0.0 {
            1 + 8
        } else {
            0
        }
    }
}

impl Type for DateTime {
    fn encode<W: Write>(&self, w: &mut W, id: u8) -> Result<()> {
        let DateTime {
            seconds: s,
            nano_seconds: ns,
        } = *self;
        if s != 0 || ns != 0 {
            if s < 1 << 32 {
                w.write_u8(id)?;
                w.write_u32::<BE>(s as u32)?;
            } else {
                w.write_u8(id | 0x80)?;
                w.write_u64::<BE>(s as u64)?;
            }
            w.write_u32::<BE>(ns)?;
        }
        Ok(())
    }

    fn decode<R: Read>(r: &mut R, flag: bool) -> Result<Self> {
        if !flag {
            let s = r.read_u32::<BE>()?;
            let ns = r.read_u32::<BE>()?;
            Ok(DateTime {
                seconds: s as i64,
                nano_seconds: ns,
            })
        } else {
            let s = r.read_u64::<BE>()?;
            let ns = r.read_u32::<BE>()?;
            Ok(DateTime {
                seconds: s as i64,
                nano_seconds: ns,
            })
        }
    }

    fn size(&self) -> usize {
        let DateTime {
            seconds: s,
            nano_seconds: ns,
        } = *self;
        if s != 0 || ns != 0 {
            if s < 1 << 32 {
                1 + 8
            } else {
                1 + 12
            }
        } else {
            0
        }
    }
}

impl Type for String {
    fn encode<W: Write>(&self, w: &mut W, id: u8) -> Result<()> {
        if !self.is_empty() {
            w.write_u8(id)?;
            write_uint(w, self.len() as u64)?;
            w.write_all(self.as_bytes())?;
        }
        Ok(())
    }

    fn decode<R: Read>(r: &mut R, _flag: bool) -> Result<Self> {
        let l = read_uint(r)?;
        let mut s = vec![0; l as usize];
        r.read_exact(&mut s)?;
        Ok(String::from_utf8(s).map_err(|err| Error::new(ErrorKind::Other, err))?)
    }

    fn size(&self) -> usize {
        if !self.is_empty() {
            1 + uint_size(self.len() as u64) + self.len()
        } else {
            0
        }
    }
}

impl Type for Vec<u8> {
    fn encode<W: Write>(&self, w: &mut W, id: u8) -> Result<()> {
        if !self.is_empty() {
            w.write_u8(id)?;
            write_uint(w, self.len() as u64)?;
            w.write_all(self)?;
        }
        Ok(())
    }

    fn decode<R: Read>(r: &mut R, _flag: bool) -> Result<Self> {
        let l = read_uint(r)?;
        let mut s = vec![0; l as usize];
        r.read_exact(&mut s)?;
        Ok(s)
    }

    fn size(&self) -> usize {
        if !self.is_empty() {
            1 + uint_size(self.len() as u64) + self.len()
        } else {
            0
        }
    }
}

#[doc(hidden)]
pub fn encode_message<W: Write, T: Message>(w: &mut W, id: u8, message: Option<&T>) -> Result<()> {
    if let Some(message) = message {
        w.write_u8(id)?;
        message.encode(w)?;
    }
    Ok(())
}

#[doc(hidden)]
pub fn decode_message<R: Read, M: Message, T: Deref<Target = M> + From<M>>(
    r: &mut R,
) -> Result<Option<T>> {
    Ok(Some(T::from(M::decode(r)?)))
}

#[doc(hidden)]
pub fn encode_messages<W: Write, T: Message>(w: &mut W, id: u8, messages: &[T]) -> Result<()> {
    if !messages.is_empty() {
        w.write_u8(id)?;
        write_uint(w, messages.len() as u64)?;
        for s in messages {
            s.encode(w)?;
        }
    }
    Ok(())
}

#[doc(hidden)]
pub fn decode_messages<R: Read, T: Message>(r: &mut R) -> Result<Vec<T>> {
    let l = read_uint(r)?;
    let mut s = Vec::with_capacity(l as usize);
    for _ in 0..l {
        s.push(T::decode(r)?);
    }
    Ok(s)
}

impl Type for Vec<String> {
    fn encode<W: Write>(&self, w: &mut W, id: u8) -> Result<()> {
        if !self.is_empty() {
            w.write_u8(id)?;
            write_uint(w, self.len() as u64)?;
            for s in self {
                write_uint(w, self.len() as u64)?;
                w.write_all(s.as_bytes())?;
            }
        }
        Ok(())
    }

    fn decode<R: Read>(r: &mut R, _flag: bool) -> Result<Self> {
        let l = read_uint(r)?;
        let mut s = Vec::with_capacity(l as usize);
        for _ in 0..l {
            let sz = read_uint(r)?;
            let mut d = vec![0; sz as usize];
            r.read_exact(&mut d)?;
            s.push(String::from_utf8(d).map_err(|err| Error::new(ErrorKind::Other, err))?);
        }
        Ok(s)
    }

    fn size(&self) -> usize {
        if !self.is_empty() {
            1 + uint_size(self.len() as u64)
                + self
                    .iter()
                    .map(|s| uint_size(s.len() as u64) + s.len())
                    .sum::<usize>()
        } else {
            0
        }
    }
}

impl Type for Vec<Vec<u8>> {
    fn encode<W: Write>(&self, w: &mut W, id: u8) -> Result<()> {
        if !self.is_empty() {
            w.write_u8(id)?;
            write_uint(w, self.len() as u64)?;
            for s in self {
                write_uint(w, self.len() as u64)?;
                w.write_all(s)?;
            }
        }
        Ok(())
    }

    fn decode<R: Read>(r: &mut R, _flag: bool) -> Result<Self> {
        let l = read_uint(r)?;
        let mut s = Vec::with_capacity(l as usize);
        for _ in 0..l {
            let sz = read_uint(r)?;
            let mut d = vec![0; sz as usize];
            r.read_exact(&mut d)?;
            s.push(d);
        }
        Ok(s)
    }

    fn size(&self) -> usize {
        if !self.is_empty() {
            1 + uint_size(self.len() as u64)
                + self
                    .iter()
                    .map(|s| uint_size(s.len() as u64) + s.len())
                    .sum::<usize>()
        } else {
            0
        }
    }
}

impl Type for u8 {
    fn encode<W: Write>(&self, w: &mut W, id: u8) -> Result<()> {
        if *self != 0 {
            w.write_u8(id)?;
            w.write_u8(*self)?;
        }
        Ok(())
    }

    fn decode<R: Read>(r: &mut R, _flag: bool) -> Result<Self> {
        r.read_u8()
    }

    fn size(&self) -> usize {
        if *self != 0 {
            1 + 1
        } else {
            0
        }
    }
}

impl Type for u16 {
    fn encode<W: Write>(&self, w: &mut W, id: u8) -> Result<()> {
        if *self >= 1 << 8 {
            w.write_u8(id)?;
            w.write_u16::<BE>(*self)?;
        } else if *self != 0 {
            w.write_u8(id | 0x80)?;
            w.write_u8(*self as u8)?;
        }
        Ok(())
    }

    fn decode<R: Read>(r: &mut R, flag: bool) -> Result<Self> {
        if !flag {
            r.read_u16::<BE>()
        } else {
            Ok(r.read_u8()? as u16)
        }
    }

    fn size(&self) -> usize {
        if *self >= 1 << 8 {
            3
        } else if *self != 0 {
            2
        } else {
            0
        }
    }
}

impl Type for Vec<f32> {
    fn encode<W: Write>(&self, w: &mut W, id: u8) -> Result<()> {
        if !self.is_empty() {
            w.write_u8(id)?;
            write_uint(w, self.len() as u64)?;
            for s in self {
                w.write_u32::<BE>(s.to_bits())?;
            }
        }
        Ok(())
    }

    fn decode<R: Read>(r: &mut R, _flag: bool) -> Result<Self> {
        let l = read_uint(r)?;
        let mut s = Vec::with_capacity(l as usize);
        for _ in 0..l {
            s.push(f32::from_bits(r.read_u32::<BE>()?));
        }
        Ok(s)
    }

    fn size(&self) -> usize {
        if !self.is_empty() {
            1 + uint_size(self.len() as u64)
                + self
                    .iter()
                    .map(|n| uint_size(n.to_bits() as u64))
                    .sum::<usize>()
        } else {
            0
        }
    }
}

impl Type for Vec<f64> {
    fn encode<W: Write>(&self, w: &mut W, id: u8) -> Result<()> {
        if !self.is_empty() {
            w.write_u8(id)?;
            write_uint(w, self.len() as u64)?;
            for s in self {
                w.write_u64::<BE>(s.to_bits())?;
            }
        }
        Ok(())
    }

    fn decode<R: Read>(r: &mut R, _flag: bool) -> Result<Self> {
        let l = read_uint(r)?;
        let mut s = Vec::with_capacity(l as usize);
        for _ in 0..l {
            s.push(f64::from_bits(r.read_u64::<BE>()?));
        }
        Ok(s)
    }

    fn size(&self) -> usize {
        if !self.is_empty() {
            1 + uint_size(self.len() as u64)
                + self.iter().map(|n| uint_size(n.to_bits())).sum::<usize>()
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;
    use std::io::Cursor;

    fn do_test<T: Type + PartialEq + Debug>(value: T) {
        let mut data = Vec::new();
        value.encode(&mut data, 10).unwrap();

        let mut r = Cursor::new(&data);
        let (id, flag) = read_header(&mut r).unwrap();
        assert_eq!(id, 10);
        assert_eq!(T::decode(&mut r, flag).unwrap(), value);
    }

    #[test]
    fn test_i32() {
        do_test(i32::MAX);
        do_test(i32::MIN);
        do_test(0i32);
    }

    #[test]
    fn test_i64() {
        do_test(i64::MAX);
        do_test(i64::MIN);
    }

    #[test]
    fn test_f32() {
        do_test(f32::MAX);
        do_test(f32::MIN);
    }

    #[test]
    fn test_f64() {
        do_test(f64::MAX);
        do_test(f64::MIN);
    }
}
