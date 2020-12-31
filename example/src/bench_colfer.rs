#![allow(unused_variables, unused_assignments, unused_mut, unused_imports)]

use colfer::bytes::{Buf, BufMut};

use colfer::{Message, Type, DateTime};

#[derive(Default, Clone, Debug, PartialEq)]
pub struct Colfer {
	pub key: i64,
	pub host: String,
	pub port: u16,
	pub size: i64,
	pub hash: u64,
	pub ratio: f64,
	pub route: bool,
}

impl Message for Colfer {
	fn encode<B: BufMut>(&self, buf: &mut B) {
		self.key.encode(buf, 0);
		self.host.encode(buf, 1);
		self.port.encode(buf, 2);
		self.size.encode(buf, 3);
		self.hash.encode(buf, 4);
		self.ratio.encode(buf, 5);
		self.route.encode(buf, 6);
		colfer::write_end(buf);
	}

	fn decode<B: Buf>(buf: B) -> Self {
		let mut buf = buf;
		let mut obj = Self::default();
		let (mut id, mut flag) = colfer::read_header(&mut buf);
		if id == 0 {
			obj.key = Type::decode(&mut buf, flag);
			let next = colfer::read_header(&mut buf);
			id = next.0;
			flag = next.1;
		}
		if id == 1 {
			obj.host = Type::decode(&mut buf, flag);
			let next = colfer::read_header(&mut buf);
			id = next.0;
			flag = next.1;
		}
		if id == 2 {
			obj.port = Type::decode(&mut buf, flag);
			let next = colfer::read_header(&mut buf);
			id = next.0;
			flag = next.1;
		}
		if id == 3 {
			obj.size = Type::decode(&mut buf, flag);
			let next = colfer::read_header(&mut buf);
			id = next.0;
			flag = next.1;
		}
		if id == 4 {
			obj.hash = Type::decode(&mut buf, flag);
			let next = colfer::read_header(&mut buf);
			id = next.0;
			flag = next.1;
		}
		if id == 5 {
			obj.ratio = Type::decode(&mut buf, flag);
			let next = colfer::read_header(&mut buf);
			id = next.0;
			flag = next.1;
		}
		if id == 6 {
			obj.route = Type::decode(&mut buf, flag);
		}

		obj
	}

	fn size(&self) -> usize {
		let mut size = 0;
		size += self.key.size();
		size += self.host.size();
		size += self.port.size();
		size += self.size.size();
		size += self.hash.size();
		size += self.ratio.size();
		size += self.route.size();
		size
	}
}

