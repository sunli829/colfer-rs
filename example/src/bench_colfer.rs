#![allow(unused_variables, unused_assignments, unused_mut, unused_imports)]

use std::io::{Write, Read, Result};

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
	#[inline]
	fn encode<W: Write>(&self, w: &mut W) -> Result<()> {
		self.key.encode(w, 0)?;
		self.host.encode(w, 1)?;
		self.port.encode(w, 2)?;
		self.size.encode(w, 3)?;
		self.hash.encode(w, 4)?;
		self.ratio.encode(w, 5)?;
		self.route.encode(w, 6)?;
		colfer::write_end(w)?;

		Ok(())
	}

	#[inline]
	fn decode<R: Read>(r: &mut R) -> Result<Self> {
		let mut obj = Self::default();
		let (mut id, mut flag) = colfer::read_header(r)?;
		if id == 0 {
			obj.key = Type::decode(r, flag)?;
			let next = colfer::read_header(r)?;
			id = next.0;
			flag = next.1;
		}
		if id == 1 {
			obj.host = Type::decode(r, flag)?;
			let next = colfer::read_header(r)?;
			id = next.0;
			flag = next.1;
		}
		if id == 2 {
			obj.port = Type::decode(r, flag)?;
			let next = colfer::read_header(r)?;
			id = next.0;
			flag = next.1;
		}
		if id == 3 {
			obj.size = Type::decode(r, flag)?;
			let next = colfer::read_header(r)?;
			id = next.0;
			flag = next.1;
		}
		if id == 4 {
			obj.hash = Type::decode(r, flag)?;
			let next = colfer::read_header(r)?;
			id = next.0;
			flag = next.1;
		}
		if id == 5 {
			obj.ratio = Type::decode(r, flag)?;
			let next = colfer::read_header(r)?;
			id = next.0;
			flag = next.1;
		}
		if id == 6 {
			obj.route = Type::decode(r, flag)?;
		}

		Ok(obj)
	}

	#[inline]
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

