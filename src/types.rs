use std::ops::Deref;

use bytes::{Buf, BufMut};

use crate::{DateTime, Message};

#[inline]
fn write_uint<B: BufMut>(buf: &mut B, mut x: u64) {
    while x >= 0x80 {
        buf.put_u8((x | 0x80) as u8);
        x >>= 7;
    }
    buf.put_u8(x as u8);
}

#[inline]
fn read_uint<B: Buf>(mut buf: B) -> u64 {
    let mut x = buf.get_u8() as u64;
    if x >= 0x80 {
        x &= 0x7f;
        let mut shift = 7;
        loop {
            let b = buf.get_u8() as u64;
            if b < 0x80 || shift == 56 {
                x |= b << shift;
                break;
            }
            x |= (b & 0x7f) << shift;
            shift += 7;
        }
    }
    x
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

#[doc(hidden)]
pub trait Type: Sized {
    fn encode<B: BufMut>(&self, buf: &mut B, id: u8);

    fn decode<B: Buf>(buf: B, flag: bool) -> Self;

    fn size(&self) -> usize;
}

impl Type for bool {
    fn encode<B: BufMut>(&self, buf: &mut B, id: u8) {
        if *self {
            buf.put_u8(id);
        }
    }

    fn decode<B: Buf>(_buf: B, _flag: bool) -> Self {
        true
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
    fn encode<B: BufMut>(&self, buf: &mut B, id: u8) {
        if *self >= 1 << 21 {
            buf.put_u8(id | 0x80);
            buf.put_u32(*self);
        } else if *self != 0 {
            buf.put_u8(id);
            write_uint(buf, *self as u64);
        }
    }

    fn decode<B: Buf>(mut buf: B, flag: bool) -> Self {
        if !flag {
            read_uint(buf) as u32
        } else {
            buf.get_u32()
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
    fn encode<B: BufMut>(&self, buf: &mut B, id: u8) {
        if *self >= 1 << 49 {
            buf.put_u8(id | 0x80);
            buf.put_u64(*self);
        } else if *self != 0 {
            buf.put_u8(id);
            write_uint(buf, *self as u64);
        }
    }

    fn decode<B: Buf>(mut buf: B, flag: bool) -> Self {
        if !flag {
            read_uint(buf)
        } else {
            buf.get_u64()
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
    fn encode<B: BufMut>(&self, buf: &mut B, id: u8) {
        (*self as i64).encode(buf, id)
    }

    fn decode<B: Buf>(buf: B, flag: bool) -> Self {
        i64::decode(buf, flag) as i32
    }

    fn size(&self) -> usize {
        (*self as i64).size()
    }
}

impl Type for i64 {
    fn encode<B: BufMut>(&self, buf: &mut B, id: u8) {
        if *self != 0 {
            let mut x = *self as u64;
            if *self >= 0 {
                buf.put_u8(id);
            } else {
                x = !x + 1;
                buf.put_u8(id | 0x80);
            }
            write_uint(buf, x as u64);
        }
    }

    fn decode<B: Buf>(buf: B, flag: bool) -> Self {
        if !flag {
            read_uint(buf) as i64
        } else {
            (!read_uint(buf) + 1) as i64
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
    fn encode<B: BufMut>(&self, buf: &mut B, id: u8) {
        if *self != 0.0 {
            buf.put_u8(id);
            buf.put_u32(self.to_bits());
        }
    }

    fn decode<B: Buf>(mut buf: B, _flag: bool) -> Self {
        f32::from_bits(buf.get_u32())
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
    fn encode<B: BufMut>(&self, buf: &mut B, id: u8) {
        if *self != 0.0 {
            buf.put_u8(id);
            buf.put_u64(self.to_bits());
        }
    }

    fn decode<B: Buf>(mut buf: B, _flag: bool) -> Self {
        f64::from_bits(buf.get_u64())
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
    fn encode<B: BufMut>(&self, buf: &mut B, id: u8) {
        let DateTime {
            seconds: s,
            nano_seconds: ns,
        } = *self;
        if s != 0 || ns != 0 {
            if s < 1 << 32 {
                buf.put_u8(id);
                buf.put_u32(s as u32);
            } else {
                buf.put_u8(id | 0x80);
                buf.put_u64(s as u64);
            }
            buf.put_u32(ns);
        }
    }

    fn decode<B: Buf>(mut buf: B, flag: bool) -> Self {
        if !flag {
            let s = buf.get_u32();
            let ns = buf.get_u32();
            DateTime {
                seconds: s as i64,
                nano_seconds: ns,
            }
        } else {
            let s = buf.get_u64();
            let ns = buf.get_u32();
            DateTime {
                seconds: s as i64,
                nano_seconds: ns,
            }
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
    fn encode<B: BufMut>(&self, buf: &mut B, id: u8) {
        if !self.is_empty() {
            buf.put_u8(id);
            write_uint(buf, self.len() as u64);
            buf.put(self.as_bytes());
        }
    }

    fn decode<B: Buf>(mut buf: B, _flag: bool) -> Self {
        let l = read_uint(&mut buf);
        let mut s = vec![0; l as usize];
        buf.copy_to_slice(&mut s);
        unsafe { String::from_utf8_unchecked(s) }
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
    fn encode<B: BufMut>(&self, buf: &mut B, id: u8) {
        if !self.is_empty() {
            buf.put_u8(id);
            write_uint(buf, self.len() as u64);
            buf.put(self.as_slice());
        }
    }

    fn decode<B: Buf>(mut buf: B, _flag: bool) -> Self {
        let l = read_uint(&mut buf);
        let mut s = vec![0; l as usize];
        buf.copy_to_slice(&mut s);
        s
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
pub fn encode_message<B: BufMut, T: Message>(buf: &mut B, id: u8, message: Option<&T>) {
    if let Some(message) = message {
        buf.put_u8(id);
        message.encode(buf);
    }
}

#[doc(hidden)]
pub fn decode_message<B: Buf, M: Message, T: Deref<Target = M> + From<M>>(buf: B) -> Option<T> {
    Some(T::from(M::decode(buf)))
}

#[doc(hidden)]
pub fn encode_messages<B: BufMut, T: Message>(buf: &mut B, id: u8, messages: &[T]) {
    if !messages.is_empty() {
        buf.put_u8(id);
        write_uint(buf, messages.len() as u64);
        for s in messages {
            s.encode(buf);
        }
    }
}

#[doc(hidden)]
pub fn decode_messages<B: Buf, T: Message>(mut buf: B) -> Vec<T> {
    let l = read_uint(&mut buf);
    let mut s = Vec::with_capacity(l as usize);
    for _ in 0..l {
        s.push(T::decode(&mut buf));
    }
    s
}

impl Type for Vec<String> {
    fn encode<B: BufMut>(&self, buf: &mut B, id: u8) {
        if !self.is_empty() {
            buf.put_u8(id);
            write_uint(buf, self.len() as u64);
            for s in self {
                write_uint(buf, self.len() as u64);
                buf.put(s.as_bytes());
            }
        }
    }

    fn decode<B: Buf>(mut buf: B, _flag: bool) -> Self {
        let l = read_uint(&mut buf);
        let mut s = Vec::with_capacity(l as usize);
        for _ in 0..l {
            let sz = read_uint(&mut buf);
            let mut d = vec![0; sz as usize];
            buf.copy_to_slice(&mut d);
            s.push(unsafe { String::from_utf8_unchecked(d) });
        }
        s
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
    fn encode<B: BufMut>(&self, buf: &mut B, id: u8) {
        if !self.is_empty() {
            buf.put_u8(id);
            write_uint(buf, self.len() as u64);
            for s in self {
                write_uint(buf, self.len() as u64);
                buf.put(s.as_slice());
            }
        }
    }

    fn decode<B: Buf>(mut buf: B, _flag: bool) -> Self {
        let l = read_uint(&mut buf);
        let mut s = Vec::with_capacity(l as usize);
        for _ in 0..l {
            let sz = read_uint(&mut buf);
            let mut d = vec![0; sz as usize];
            buf.copy_to_slice(&mut d);
            s.push(d);
        }
        s
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
    fn encode<B: BufMut>(&self, buf: &mut B, id: u8) {
        if *self != 0 {
            buf.put_u8(id);
            buf.put_u8(*self);
        }
    }

    fn decode<B: Buf>(mut buf: B, _flag: bool) -> Self {
        buf.get_u8()
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
    fn encode<B: BufMut>(&self, buf: &mut B, id: u8) {
        if *self >= 1 << 8 {
            buf.put_u8(id);
            buf.put_u16(*self);
        } else if *self != 0 {
            buf.put_u8(id | 0x80);
            buf.put_u8(*self as u8);
        }
    }

    fn decode<B: Buf>(mut buf: B, flag: bool) -> Self {
        if !flag {
            buf.get_u16()
        } else {
            buf.get_u8() as u16
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
    fn encode<B: BufMut>(&self, buf: &mut B, id: u8) {
        if !self.is_empty() {
            buf.put_u8(id);
            write_uint(buf, self.len() as u64);
            for s in self {
                buf.put_u32(s.to_bits());
            }
        }
    }

    fn decode<B: Buf>(mut buf: B, _flag: bool) -> Self {
        let l = read_uint(&mut buf);
        let mut s = Vec::with_capacity(l as usize);
        for _ in 0..l {
            s.push(f32::from_bits(buf.get_u32()));
        }
        s
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
    fn encode<B: BufMut>(&self, buf: &mut B, id: u8) {
        if !self.is_empty() {
            buf.put_u8(id);
            write_uint(buf, self.len() as u64);
            for s in self {
                buf.put_u64(s.to_bits());
            }
        }
    }

    fn decode<B: Buf>(mut buf: B, _flag: bool) -> Self {
        let l = read_uint(&mut buf);
        let mut s = Vec::with_capacity(l as usize);
        for _ in 0..l {
            s.push(f64::from_bits(buf.get_u64()));
        }
        s
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
    use std::fmt::Debug;
    use std::io::Cursor;

    use crate::*;

    fn do_test<T: Type + PartialEq + Debug + Default>(value: T) {
        let mut data = Vec::new();
        value.encode(&mut data, 10);

        let mut r = Cursor::new(&data);
        if data.is_empty() {
            assert_eq!(T::default(), value);
        } else {
            let (id, flag) = read_header(&mut r);
            assert_eq!(id, 10);
            assert_eq!(T::decode(&mut r, flag), value);
        }
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
