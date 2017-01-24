// CAF container decoder written in Rust
//
// Copyright (c) 2017 est31 <MTest31@outlook.com>
// and contributors. All rights reserved.
// Licensed under MIT license, or Apache 2 license,
// at your option. Please see the LICENSE file
// attached to this source distribution for details.

/*!
CAF chunk decoding
*/

use ::CafError;
// TODO once we drop compat for pre rust 1.15 replace this with "use ::Read;"
use std::io::Read;
// TODO once we drop compat for pre rust 1.15 replace this with "use ::IoError;"
use std::io::Error as IoError;
use ::ChunkType;
use ::FormatType;

/// A decoded CAF chunk header
pub struct CafChunkHeader {
	pub ch_type :ChunkType,
	/// The size of the chunk's content (without the head) in bytes.
	///
	/// -1 is a special value and means the chunk ends at the EOF.
	/// The spec only allows this case for the Audio Data chunk.
	/// Such a chunk is obviously last in the file.
	pub ch_size :i64,
}

/// An in-memory CAF chunk.
///
/// The list represents the chunk types we can parse.
#[derive(Debug)]
pub enum CafChunk {
	Desc(AudioDescription),
	AudioDataInMemory(u32, Vec<u8>),
	PacketTable(PacketTable),
	ChanLayout(ChannelLayout),
	MagicCookie(Vec<u8>),
	// ...
	Info(Vec<(String, String)>), // TODO use a hash map
	// ...
}

#[derive(Debug)]
pub struct AudioDescription {
	pub sample_rate :f64,
	pub format_id :FormatType,
	pub format_flags :u32,
	pub bytes_per_packet :u32,
	pub frames_per_packet :u32,
	pub channels_per_frame :u32,
	pub bits_per_channel :u32,
}


#[derive(Debug)]
pub struct PacketTable {
	pub num_valid_frames :i64,
	pub num_priming_frames :i32,
	pub num_remainder_frames :i32,
	pub lengths :Vec<u64>,
}

#[derive(Debug)]
pub struct ChannelLayout {
	// TODO enrich this one and the one below with some meaning
	// e.g. we'll maybe need some other representation, like an enum?
	pub channel_layout_tag :u32,
	pub channel_bitmap :u32,
	pub channel_descriptions :Vec<ChannelDescription>,
}

#[derive(Debug)]
pub struct ChannelDescription {
	pub channel_label :u32,
	pub channel_flags :u32,
	pub coordinates :(f32, f32, f32),
}

/// Returns whether `decode_chunk` can decode chunks with the given type
pub fn can_decode_chunk_type(chunk_type :ChunkType) -> bool {
	use ChunkType::*;
	match chunk_type {
		AudioDescription |
		AudioData |
		PacketTable |
		ChannelLayout |
		MagicCookie |
		Info
		=> true,
		_ => false,
	}
}

/// Decodes an in-memory chunk given its type and content
///
/// If the given chunk type is not not supported, the function will
/// return `CafError::UnsupportedChunkType` in this case.
pub fn decode_chunk(chunk_type :ChunkType, mut chunk_content :Vec<u8>)
		-> Result<CafChunk, CafError> {
	use byteorder::BigEndian as Be;
	use byteorder::ReadBytesExt;
	use std::io::{Cursor, BufRead};
	// ReaD with big endian order and Try
	macro_rules! rdt {
		($rdr:ident, $func:ident) => { try!($rdr.$func::<Be>()) }
	}
	match chunk_type {
			ChunkType::AudioDescription => {
				let mut rdr = Cursor::new(&chunk_content);
				let sample_rate = rdr.read_f64::<Be>().unwrap();
				Ok(CafChunk::Desc(AudioDescription {
					sample_rate : sample_rate,
					format_id : FormatType::from(rdr.read_u32::<Be>().unwrap()),
					format_flags : rdr.read_u32::<Be>().unwrap(),
					bytes_per_packet : rdt!(rdr,read_u32),
					frames_per_packet : rdt!(rdr,read_u32),
					channels_per_frame : rdt!(rdr,read_u32),
					bits_per_channel : rdt!(rdr,read_u32),
				}))
			},
			ChunkType::AudioData => {
				let edit_count = {
					let mut rdr = Cursor::new(&chunk_content);
					rdr.read_u32::<Be>().unwrap()
				};
				// Remove the value just read from the vec
				let new_chunk_content_len = chunk_content.len() - 4;
				for i in 0..new_chunk_content_len {
					chunk_content[i] = chunk_content[i + 4];
				}
				chunk_content.truncate(new_chunk_content_len);
				Ok(CafChunk::AudioDataInMemory(
					edit_count,
					chunk_content
				))
			},
			ChunkType::PacketTable => {
				let mut rdr = Cursor::new(&chunk_content);
				let num_packets =  rdt!(rdr, read_i64);
				Ok(CafChunk::PacketTable(PacketTable {
					num_valid_frames : rdt!(rdr, read_i64),
					num_priming_frames : rdt!(rdr, read_i32),
					num_remainder_frames : rdt!(rdr, read_i32),
					lengths : {
						let mut lengths = Vec::with_capacity(num_packets as usize);
						for _ in 0..num_packets {
							let b = try!(read_vlq(&mut rdr));
							lengths.push(b);
						}
						lengths
					},
				}))
			},
			ChunkType::ChannelLayout => {
				let mut rdr = Cursor::new(&chunk_content);
				let channel_layout_tag = rdr.read_u32::<Be>().unwrap();
				let channel_bitmap = rdr.read_u32::<Be>().unwrap();
				let channel_descriptions_count = rdt!(rdr, read_u32);
				let mut descs = Vec::with_capacity(channel_descriptions_count as usize);
				for _ in 0..channel_descriptions_count {
					descs.push(ChannelDescription {
						channel_label : rdt!(rdr, read_u32),
						channel_flags : rdt!(rdr, read_u32),
						coordinates : (rdt!(rdr, read_f32),
							rdt!(rdr, read_f32), rdt!(rdr, read_f32)),
					});
				}
				Ok(CafChunk::ChanLayout(ChannelLayout {
					channel_layout_tag : channel_layout_tag,
					channel_bitmap : channel_bitmap,
					channel_descriptions : descs,
				}))
			},
			ChunkType::MagicCookie => Ok(CafChunk::MagicCookie(
				chunk_content
			)),
			// ...
			ChunkType::Info => {
				let mut rdr = Cursor::new(&chunk_content);
				let num_entries = rdt!(rdr, read_u32);
				let mut res = Vec::with_capacity(num_entries as usize);
				for _ in 0..num_entries {
					let mut key = Vec::new();
					let mut val = Vec::new();
					try!(rdr.read_until(0, &mut key));
					try!(rdr.read_until(0, &mut val));
					// Remove the trailing \0. Somehow neither
					// read_until nor from_utf8 does this for us.
					key.pop();
					val.pop();
					res.push((try!(String::from_utf8(key)), try!(String::from_utf8(val))));
				}
				Ok(CafChunk::Info(res))
			},
			// ...
			_ => try!(Err(CafError::UnsupportedChunkType(chunk_type))),
	}
}

fn read_vlq<T :Read>(rdr :&mut T) -> Result<u64, IoError> {
	let mut res = 0;
	let mut buf = [0; 1];
	// TODO ensure we don't exceed 64 bytes.
	loop {
		try!(rdr.read_exact(&mut buf));
		let byte = buf[0];
		res <<= 7;
		res |= (byte & 127) as u64;
		if byte & 128 == 0 {
			return Ok(res);
		}
	}
}
