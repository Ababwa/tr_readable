use std::io::{Read, Cursor, Result};
use byteorder::{ReadBytesExt, LE};
use compress::zlib::Decoder;

pub use tr_derive::Readable;

pub trait Readable {
	fn read<R: Read>(reader: &mut R) -> Result<Self> where Self: Sized;
}

macro_rules! impl_readable_prim {
	($type:ty, $func:ident $(, $($endian:tt)*)?) => {
		impl Readable for $type {
			fn read<R: Read>(reader: &mut R) -> Result<Self> {
				reader.$func$($($endian)*)?()
			}
		}
	};
}

macro_rules! impl_readable_prim_le {
	($type:ty, $func:ident) => {
		impl_readable_prim!($type, $func, ::<LE>);
	};
}

impl_readable_prim!(u8, read_u8);
impl_readable_prim!(i8, read_i8);
impl_readable_prim_le!(u16, read_u16);
impl_readable_prim_le!(i16, read_i16);
impl_readable_prim_le!(u32, read_u32);
impl_readable_prim_le!(i32, read_i32);
impl_readable_prim_le!(u64, read_u64);
impl_readable_prim_le!(i64, read_i64);
impl_readable_prim_le!(f32, read_f32);
impl_readable_prim_le!(f64, read_f64);

pub fn read_vec<R: Read, T: Readable>(reader: &mut R, len: usize) -> Result<Vec<T>> {
	let mut vec = Vec::with_capacity(len);
	for _ in 0..len {
		vec.push(T::read(reader)?);
	}
	Ok(vec)
}

impl<T: Readable, const N: usize> Readable for [T; N] {
	fn read<R: Read>(reader: &mut R) -> Result<Self> {
		Ok(read_vec(reader, N)?.try_into().ok().unwrap())//reads exactly N items
	}
}

impl<T: Readable, const N: usize> Readable for Box<[T; N]> {
	fn read<R: Read>(reader: &mut R) -> Result<Self> {
		Ok(read_vec(reader, N)?.into_boxed_slice().try_into().ok().unwrap())//reads exactly N items
	}
}

pub trait Len {
	fn read_len<R: Read>(reader: &mut R) -> Result<usize>;
}

macro_rules! impl_len {
	($type:ty, $func:ident) => {
		impl Len for $type {
			fn read_len<R: Read>(reader: &mut R) -> Result<usize> {
				Ok(reader.$func::<LE>()? as usize)
			}
		}
	};
}

impl_len!(u16, read_u16);
impl_len!(u32, read_u32);

pub fn read_list<R: Read, T: Readable, L: Len>(reader: &mut R) -> Result<Vec<T>> {
	let len = L::read_len(reader)?;
	read_vec(reader, len)
}

pub fn read_list_2d<R: Read, T: Readable>(reader: &mut R) -> Result<Vec<Vec<T>>> {
	let len1 = u16::read_len(reader)?;
	let len2 = u16::read_len(reader)?;
	let mut vec = Vec::with_capacity(len1);
	for _ in 0..len1 {
		vec.push(read_vec(reader, len2)?);
	}
	Ok(vec)
}

pub fn read_meshes<R: Read, T: Readable>(reader: &mut R) -> Result<Vec<T>> {
	let num_bytes = u32::read_len(reader)? * 2;
	let bytes = read_vec::<_, u8>(reader, num_bytes)?;
	let mut cursor = Cursor::new(bytes);
	let mut vec = Vec::new();
	let num_bytes = num_bytes as u64;
	loop {
		let pos1 = cursor.position();
		if num_bytes - pos1 == 0 {
			break;
		}
		vec.push(T::read(&mut cursor)?);
		let pos2 = cursor.position();
		if (pos2 - pos1) % 4 != 0 {
			cursor.set_position(pos2 + 2);
		}
	}
	Ok(vec)
}

pub fn get_zlib<R: Read>(reader: &mut R) -> Result<Decoder<Cursor<Vec<u8>>>> {
	u32::read_len(reader)?;//uncompressed_len
	let compressed_len = u32::read_len(reader)?;
	let bytes = read_vec::<_, u8>(reader, compressed_len)?;
	Ok(Decoder::new(Cursor::new(bytes)))
}

pub fn skip<R: Read, const N: usize>(reader: &mut R) -> Result<()> {
	let mut buf = [0];
	for _ in 0..N {
		reader.read_exact(&mut buf)?;
	}
	Ok(())
}
