#![allow(unused_variables, unused_assignments, unused_mut, unused_imports)]

use colfer::bytes::{Buf, BufMut};

use colfer::{Message, Type, DateTime};

#[derive(Default, Clone, Debug, PartialEq)]
pub struct O {
	pub b: bool,
	pub u32: u32,
	pub u64: u64,
	pub i32: i32,
	pub i64: i64,
	pub f32: f32,
	pub f64: f64,
	pub t: DateTime,
	pub s: String,
	pub a: Vec<u8>,
	pub o: Option<Box<O>>,
	pub os: Vec<O>,
	pub ss: Vec<String>,
	pub r#as: Vec<Vec<u8>>,
	pub u8: u8,
	pub u16: u16,
	pub f32s: Vec<f32>,
	pub f64s: Vec<f64>,
}

impl Message for O {
	fn encode<B: BufMut>(&self, buf: &mut B) {
		self.b.encode(buf, 0);
		self.u32.encode(buf, 1);
		self.u64.encode(buf, 2);
		self.i32.encode(buf, 3);
		self.i64.encode(buf, 4);
		self.f32.encode(buf, 5);
		self.f64.encode(buf, 6);
		self.t.encode(buf, 7);
		self.s.encode(buf, 8);
		self.a.encode(buf, 9);
		colfer::encode_message(buf, 10, self.o.as_deref());
		colfer::encode_messages(buf, 11, &self.os);
		self.ss.encode(buf, 12);
		self.r#as.encode(buf, 13);
		self.u8.encode(buf, 14);
		self.u16.encode(buf, 15);
		self.f32s.encode(buf, 16);
		self.f64s.encode(buf, 17);
		colfer::write_end(buf);
	}

	fn decode<B: Buf>(buf: B) -> Self {
		let mut buf = buf;
		let mut obj = Self::default();
		let (mut id, mut flag) = colfer::read_header(&mut buf);
		if id == 0 {
			obj.b = Type::decode(&mut buf, flag);
			let next = colfer::read_header(&mut buf);
			id = next.0;
			flag = next.1;
		}
		if id == 1 {
			obj.u32 = Type::decode(&mut buf, flag);
			let next = colfer::read_header(&mut buf);
			id = next.0;
			flag = next.1;
		}
		if id == 2 {
			obj.u64 = Type::decode(&mut buf, flag);
			let next = colfer::read_header(&mut buf);
			id = next.0;
			flag = next.1;
		}
		if id == 3 {
			obj.i32 = Type::decode(&mut buf, flag);
			let next = colfer::read_header(&mut buf);
			id = next.0;
			flag = next.1;
		}
		if id == 4 {
			obj.i64 = Type::decode(&mut buf, flag);
			let next = colfer::read_header(&mut buf);
			id = next.0;
			flag = next.1;
		}
		if id == 5 {
			obj.f32 = Type::decode(&mut buf, flag);
			let next = colfer::read_header(&mut buf);
			id = next.0;
			flag = next.1;
		}
		if id == 6 {
			obj.f64 = Type::decode(&mut buf, flag);
			let next = colfer::read_header(&mut buf);
			id = next.0;
			flag = next.1;
		}
		if id == 7 {
			obj.t = Type::decode(&mut buf, flag);
			let next = colfer::read_header(&mut buf);
			id = next.0;
			flag = next.1;
		}
		if id == 8 {
			obj.s = Type::decode(&mut buf, flag);
			let next = colfer::read_header(&mut buf);
			id = next.0;
			flag = next.1;
		}
		if id == 9 {
			obj.a = Type::decode(&mut buf, flag);
			let next = colfer::read_header(&mut buf);
			id = next.0;
			flag = next.1;
		}
		if id == 10 {
			obj.o = colfer::decode_message(&mut buf);
			let next = colfer::read_header(&mut buf);
			id = next.0;
			flag = next.1;
		}
		if id == 11 {
			obj.os = colfer::decode_messages(&mut buf);
			let next = colfer::read_header(&mut buf);
			id = next.0;
			flag = next.1;
		}
		if id == 12 {
			obj.ss = Type::decode(&mut buf, flag);
			let next = colfer::read_header(&mut buf);
			id = next.0;
			flag = next.1;
		}
		if id == 13 {
			obj.r#as = Type::decode(&mut buf, flag);
			let next = colfer::read_header(&mut buf);
			id = next.0;
			flag = next.1;
		}
		if id == 14 {
			obj.u8 = Type::decode(&mut buf, flag);
			let next = colfer::read_header(&mut buf);
			id = next.0;
			flag = next.1;
		}
		if id == 15 {
			obj.u16 = Type::decode(&mut buf, flag);
			let next = colfer::read_header(&mut buf);
			id = next.0;
			flag = next.1;
		}
		if id == 16 {
			obj.f32s = Type::decode(&mut buf, flag);
			let next = colfer::read_header(&mut buf);
			id = next.0;
			flag = next.1;
		}
		if id == 17 {
			obj.f64s = Type::decode(&mut buf, flag);
		}

		obj
	}

	fn size(&self) -> usize {
		let mut size = 0;
		size += self.b.size();
		size += self.u32.size();
		size += self.u64.size();
		size += self.i32.size();
		size += self.i64.size();
		size += self.f32.size();
		size += self.f64.size();
		size += self.t.size();
		size += self.s.size();
		size += self.a.size();
		size += self.o.as_ref().map(|s| s.size()).unwrap_or_default();
		size += self.os.iter().map(|s| s.size()).sum::<usize>();
		size += self.ss.size();
		size += self.r#as.size();
		size += self.u8.size();
		size += self.u16.size();
		size += self.f32s.size();
		size += self.f64s.size();
		size
	}
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct DromedaryCase {
	pub pascal_case: String,
}

impl Message for DromedaryCase {
	fn encode<B: BufMut>(&self, buf: &mut B) {
		self.pascal_case.encode(buf, 0);
		colfer::write_end(buf);
	}

	fn decode<B: Buf>(buf: B) -> Self {
		let mut buf = buf;
		let mut obj = Self::default();
		let (mut id, mut flag) = colfer::read_header(&mut buf);
		if id == 0 {
			obj.pascal_case = Type::decode(&mut buf, flag);
		}

		obj
	}

	fn size(&self) -> usize {
		let mut size = 0;
		size += self.pascal_case.size();
		size
	}
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct EmbedO {
	pub inner: Option<Box<O>>,
}

impl Message for EmbedO {
	fn encode<B: BufMut>(&self, buf: &mut B) {
		colfer::encode_message(buf, 0, self.inner.as_deref());
		colfer::write_end(buf);
	}

	fn decode<B: Buf>(buf: B) -> Self {
		let mut buf = buf;
		let mut obj = Self::default();
		let (mut id, mut flag) = colfer::read_header(&mut buf);
		if id == 0 {
			obj.inner = colfer::decode_message(&mut buf);
		}

		obj
	}

	fn size(&self) -> usize {
		let mut size = 0;
		size += self.inner.as_ref().map(|s| s.size()).unwrap_or_default();
		size
	}
}

