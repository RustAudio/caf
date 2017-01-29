// CAF container decoder written in Rust
//
// Copyright (c) 2017 est31 <MTest31@outlook.com>
// and contributors. All rights reserved.
// Licensed under MIT license, or Apache 2 license,
// at your option. Please see the LICENSE file
// attached to this source distribution for details.


/*!
An Apple Core Audio Format (CAF) container decoder

For more information on CAF, see its [wiki page](https://en.wikipedia.org/wiki/Core_Audio_Format), and the [official specification](https://developer.apple.com/documentation/MusicAudio/Reference/CAFSpec/).
*/

#![forbid(unsafe_code)]

extern crate byteorder;

pub mod chunks;
mod enums;

pub use enums::ChunkType;
pub use enums::FormatType;

use chunks::CafChunk;
use chunks::CafChunkHeader;

use std::io::{Read, Seek, SeekFrom, Error as IoError};
use std::string::FromUtf8Error;
use byteorder::{BigEndian as Be, ReadBytesExt};

/// The CAF file header
const CAF_HEADER_MAGIC :[u8; 8] = [0x63, 0x61, 0x66, 0x66, 0x00, 0x01, 0x00, 0x00];

#[derive(Debug)]
pub enum CafError {
	Io(IoError),
	FromUtf8(FromUtf8Error),
	/// If the given stream doesn't start with a CAF header.
	NotCaf,
	/// If the chunk can't be decoded because its type is not supported
	UnsupportedChunkType(ChunkType),
}

impl From<IoError> for CafError {
	fn from(io_err :IoError) -> Self {
		CafError::Io(io_err)
	}
}

impl From<FromUtf8Error> for CafError {
	fn from(utf8_err :FromUtf8Error) -> Self {
		CafError::FromUtf8(utf8_err)
	}
}

pub struct CafChunkReader<T> where T :Read {
	rdr :T,
}

impl<T> CafChunkReader<T> where T :Read {
	pub fn new(mut rdr :T) -> Result<Self, CafError> {
		let mut hdr_buf = [0;8];
		try!(rdr.read_exact(&mut hdr_buf));
		if hdr_buf != CAF_HEADER_MAGIC {
			return Err(CafError::NotCaf);
		}
		Ok(CafChunkReader { rdr : rdr })
	}
	/// Returns the reader that this Reader wraps
	pub fn into_inner(self) -> T {
		self.rdr
	}
	// TODO find a better API.
	// First, we don't want to pass the audio chunk via memory always.
	// Sometimes a file can be very big, so we better leave the choice
	// with the users of this library.
	// Second, we don't want to decode everything ad-hoc, maybe copying
	// around data that the user doesn't really want.
	// A good way to fix this would be to add three API functionalities: first one to
	// iterate the Read stream on a chunk granularity. The only info we read and
	// return here will be the type and size of the chunk.
	// The second API functionality should involve a function "give me some sort of view over the raw data of the chunk we last encountered in the first function".
	// this will be useful for the audio stuff.
	// The third API functionality would involve a function to read+decode the chunk
	// directly into memory, very similar to the read_chunk function now.
	// On top of this three layer API we can provide a simplified API that contains
	// even more logic, e.g. an iterator over audio packets, or similar. Maybe, it
	// can be integrated though, not sure...
	// The first priority though should be to get the alac crate working with our code.
	pub fn read_chunk(&mut self) -> Result<CafChunk, CafError> {
		let hdr = try!(self.read_chunk_header());
		self.read_chunk_body(&hdr)
	}
	/// Reads a chunk body into memory and decodes it
	pub fn read_chunk_body(&mut self, hdr :&CafChunkHeader)
			-> Result<CafChunk, CafError> {
		if hdr.ch_size == -1 {
			// Unspecified chunk size: this means the chunk is extends up to the EOF.
			// TODO handle this case
			panic!("unspecified chunk size is not yet implemented");
		}
		let mut chunk_content = vec![0; hdr.ch_size as usize];
		try!(self.rdr.read_exact(&mut chunk_content));
		chunks::decode_chunk(hdr.ch_type, chunk_content)
	}
	/// Reads a chunk header
	pub fn read_chunk_header(&mut self) -> Result<CafChunkHeader, CafError> {
		let chunk_type_u32 = try!(self.rdr.read_u32::<Be>());
		let chunk_type = ChunkType::from(chunk_type_u32);
		// TODO return some kind of error if chunk_size < 0 and != -1
		let chunk_size = try!(self.rdr.read_i64::<Be>());
		Ok(CafChunkHeader {
			ch_type : chunk_type,
			ch_size : chunk_size,
		})
	}
}

impl<T> CafChunkReader<T> where T :Read + Seek {

	/**
	Seeks to the next chunk header in the file

	It is meant to be called directly after a chunk header
	has been read, with the internal reader's position
	at the start of a chunk's body. It then seeks to the
	next chunk header.

	With this function you can ignore chunks, not reading them,
	if they have uninteresting content, or if further knowledge
	on the file is needed before their content becomes interesting.

	Panics if the header's chunk size is unspecified per spec (==-1).
	"Skipping" would make no sense here, as it will put you to the end of the file.
	*/
	pub fn to_next_chunk(&mut self, hdr :&CafChunkHeader) -> Result<(), CafError> {
		if hdr.ch_size == -1 {
			// This would be EOF, makes no sense...
			panic!("can't seek to end of chunk with unspecified chunk size.");
		}
		try!(self.rdr.seek(SeekFrom::Current(hdr.ch_size)));
		Ok(())
	}
	/**
	Seeks to the previous chunk header in the file

	It is meant to be called with the internal reader's position
	at the end of a chunk's body. It then seeks to the start of
	that chunk body.

	Panics if the header's chunk size is unspecified per spec (==-1).
	"Skipping" would make no sense here, as it will put you to the end of the file.
	*/
	pub fn to_previous_chunk(&mut self, hdr :&CafChunkHeader) -> Result<(), CafError> {
		if hdr.ch_size == -1 {
			// This would be EOF, makes no sense...
			panic!("can't seek to end of chunk with unspecified chunk size.");
		}
		try!(self.rdr.seek(SeekFrom::Current(-hdr.ch_size)));
		Ok(())
	}

	/**
	Read chunks from a whitelist to memory

	Uses the given `CafChunkReader` to read all chunks to memory
	whose types are inside the `content_read` slice.
	Stops as soon as all chunk were encountered with types in the
	`required` argument list.

	As we don't have support for reading chunks with unspecified length,
	you shouldn't use this function to read audio data to memory.
	Generally, reading the audio data chunk to memory is a bad idea
	as it may possibly be very big. Instead, use the nice high level
	`CafPacketReader` struct.
	*/
	pub fn read_chunks_to_mem(&mut self,
			mut required :Vec<ChunkType>, content_read :&[ChunkType])
			-> Result<(Vec<CafChunk>, Vec<CafChunkHeader>), CafError> {
		let mut res = Vec::with_capacity(content_read.len());
		let mut read_headers = Vec::new();
		loop {
			let hdr = try!(self.read_chunk_header());
			let mut required_idx = None;
			let mut content_read_found = false;
			for (i, &searched_type) in required.iter().enumerate() {
				if searched_type == hdr.ch_type {
					required_idx = Some(i);
					break;
				}
			}
			for &searched_type in content_read.iter() {
				if searched_type == hdr.ch_type {
					content_read_found = true;
					break;
				}
			}
			if hdr.ch_size == -1 {
				// TODO: return an error.
				/*
				We don't support chunks with unspecified (=-1) length.
				Reading such a chunk to memory would be a bad idea as they
				can possibly be gigantic, and are only used for the audio chunk,
				which is a very uninteresting target to be read to memory anyways.
				Also, such chunks are only found at the end of the file, and if we
				encounter them it means we didn't find the chunks we searched for.
				*/
			}

			match required_idx { None => (), Some(i) => { required.remove(i); } }
			if content_read_found {
				res.push(try!(self.read_chunk_body(&hdr)));
			} else {
				try!(self.to_next_chunk(&hdr));
			}
			read_headers.push(hdr.clone());
			if required.len() == 0 {
				break;
			}
		}
		Ok((res, read_headers))
	}
}

/**
High level Packet reading

Provides a very convenient iterator over the packets of the audio chunk.
*/
pub struct CafPacketReader<T> where T :Read + Seek {
	ch_rdr :CafChunkReader<T>,
	pub audio_desc :chunks::AudioDescription,
	pub packet_table :Option<chunks::PacketTable>,
	pub chunks :Vec<CafChunk>,
	/// The edit count value stored in the audio chunk.
	pub edit_count :u32,
	audio_chunk_len :i64,
	audio_chunk_offs :i64,
	packet_idx :usize,
}

impl<T> CafPacketReader<T> where T :Read + Seek {
	/// Creates a new CAF packet reader struct from a given reader.
	///
	/// With the `filter_by` argument you can pass a list of chunk types
	/// that are important for you. You shouldn't specify three chunk
	/// types though: `AudioData`, `AudioDescription` and `PacketTable`.
	/// These are implicitly retrieved, and you can extract the content
	/// through iterating over the packets (which are all small parts of
	/// the `AudioData` chunk), and through the `audio_desc` and `packet_table`
	/// members.
	///
	/// Equal to calling `CafChunkReader::new` and passing its result to
	/// `from_chunk_reader`.
	pub fn new(rdr :T, filter_by :Vec<ChunkType>) -> Result<Self, CafError> {
		let ch_rdr = try!(CafChunkReader::new(rdr));
		return CafPacketReader::from_chunk_reader(ch_rdr, filter_by);
	}

	/// Creates a new CAF packet reader struct from a given chunk reader.
	///
	/// With the `filter_by` argument you can pass a list of chunk types
	/// that are important for you. You shouldn't specify three chunk
	/// types though: `AudioData`, `AudioDescription` and `PacketTable`.
	/// These are implicitly retrieved, and you can extract the content
	/// through iterating over the packets (which are all small parts of
	/// the `AudioData` chunk), and through the `audio_desc` and `packet_table`
	/// members.
	pub fn from_chunk_reader(mut ch_rdr :CafChunkReader<T>,
			mut filter_by :Vec<ChunkType>) -> Result<Self, CafError> {

		// 1. Read all the chunks we need to memory
		filter_by.push(ChunkType::AudioDescription);
		let mut content_read = filter_by.clone();
		content_read.push(ChunkType::PacketTable);
		let (mut chunks_in_mem, mut read_headers) =
			try!(ch_rdr.read_chunks_to_mem(filter_by, &content_read));

		// 2. Extract the special chunks we will need later on
		let mut audio_desc_idx = None;
		let mut packet_table_idx = None;
		for (idx, ch) in chunks_in_mem.iter().enumerate() {
			use ChunkType::*;
			//println!("{:?}", ch.get_type());
			match ch.get_type() {
				AudioDescription => (audio_desc_idx = Some(idx)),
				PacketTable => (packet_table_idx = Some(idx)),
				_ => (),
			}
		}
		macro_rules! remove_and_unwrap {
			($idx:expr, $id:ident) => {
				match chunks_in_mem.remove($idx) {
					CafChunk::$id(v) => v,
					_ => panic!(),
				}
			}
		}
		let audio_desc = remove_and_unwrap!(audio_desc_idx.unwrap(), Desc);
		let p_table_required = audio_desc.bytes_per_packet == 0 ||
			audio_desc.frames_per_packet == 0;
		let packet_table = match packet_table_idx {
			Some(i) => Some(remove_and_unwrap!(i, PacketTable)),
			None if p_table_required => {
				let (chunks, hdrs) =  try!(ch_rdr.read_chunks_to_mem(
						vec![ChunkType::PacketTable],
						&content_read));
				chunks_in_mem.extend_from_slice(&chunks);
				read_headers.extend_from_slice(&hdrs);
				for (idx, ch) in chunks_in_mem.iter().enumerate() {
					use ChunkType::*;
					match ch.get_type() {
						PacketTable => (packet_table_idx = Some(idx)),
						_ => (),
					}
				}
				Some(remove_and_unwrap!(packet_table_idx.unwrap(), PacketTable))
			},
			// Only reaches this if p_table_required == false
			None => None,
		};

		// 3. Navigate to audio chunk position.
		// Check whether we already read the audio block.
		// If yes, calculate the amount to seek back to get to it.
		let mut audio_chunk_len = 0;
		let mut seek_backwards = 0;
		const HEADER_LEN :i64 = 12;
		for hdr in read_headers.iter() {
			if seek_backwards > 0 || hdr.ch_type == ChunkType::AudioData {
				seek_backwards += HEADER_LEN;
				seek_backwards += hdr.ch_size;
			}
			if hdr.ch_type == ChunkType::AudioData {
				audio_chunk_len = hdr.ch_size;
			}
		}
		if seek_backwards != 0 {
			// We already skipped the audio chunk once.
			// Seek back to it, and we are done.
			seek_backwards -= HEADER_LEN;
			//println!("seek_backwards: {}", seek_backwards);
			try!(ch_rdr.rdr.seek(SeekFrom::Current(-(seek_backwards as i64))));
		} else {
			// The audio chunk is ahead of us. Seek towards it.
			loop {
				let ch_hdr = try!(ch_rdr.read_chunk_header());
				if ch_hdr.ch_type == ChunkType::AudioData {
					audio_chunk_len = ch_hdr.ch_size;
					break;
				} else {
					try!(ch_rdr.to_next_chunk(&ch_hdr));
				}
			}
		}
		// 4. Read the edit count
		let edit_count = {
			use byteorder::{ReadBytesExt, BigEndian};
			try!(ch_rdr.rdr.read_u32::<BigEndian>())
		};
		// 5. Return the result
		Ok(CafPacketReader {
			ch_rdr : ch_rdr,
			audio_desc : audio_desc,
			packet_table : packet_table,
			chunks : chunks_in_mem,
			edit_count : edit_count,
			audio_chunk_len : audio_chunk_len,
			audio_chunk_offs : 4, // 4 bytes for the edit count.
			packet_idx : 0,
		})
	}
	pub fn into_inner(self) -> CafChunkReader<T> {
		self.ch_rdr
	}
	/// Returns whether the size of the packets doesn't change
	///
	/// Some formats have a constant, not changing packet size
	/// (mostly the uncompressed ones).
	pub fn packet_size_is_constant(&self) -> bool {
		return self.audio_desc.bytes_per_packet != 0;
	}
	/// Returns the size of the next packet in bytes.
	///
	/// Returns None if all packets were read,
	/// Some(_) otherwise.
	///
	/// Very useful if you want to allocate the packet
	/// slice yourself.
	pub fn next_packet_size(&self) -> Option<usize> {
		let res = match self.audio_desc.bytes_per_packet {
			0 => match self.packet_table.as_ref()
					.unwrap().lengths.get(self.packet_idx) {
				Some(v) => *v as usize,
				None => return None,
			},
			v => v as usize,
		};
		if self.audio_chunk_len != -1 &&
				self.audio_chunk_offs + res as i64 > self.audio_chunk_len {
			// We would read outside of the chunk.
			// In theory this is a format error as the packet table is not
			// supposed to have such a length combination that the sum is larger
			// than the size of the audio chunk + 4 for edit_count.
			// But we are too lazy to return Result<...> here...
			None
		} else {
			Some(res)
		}
	}
	/// Read one packet from the audio chunk
	///
	/// Returns Ok(Some(v)) if the next packet could be read successfully,
	/// Ok(None) if its the last chunk.
	pub fn next_packet(&mut self) -> Result<Option<Vec<u8>>, CafError> {
		let next_packet_size = match self.next_packet_size() {
			Some(v) => v,
			None => return Ok(None),
		};

		let mut arr = vec![0; next_packet_size];
		try!(self.ch_rdr.rdr.read_exact(&mut arr));
		self.packet_idx += 1;
		self.audio_chunk_offs += next_packet_size as i64;
		return Ok(Some(arr));
	}
	/// Read one packet from the audio chunk into a pre-allocated array
	///
	/// The method doesn't check whether the size of the passed slice matches
	/// the actual next packet length, it uses the length blindly.
	/// For correct operation, only use sizes returned from the
	/// `next_packet_size` function, and only if it didn't return `None`.
	pub fn read_packet_into(&mut self, data :&mut [u8]) -> Result<(), CafError> {
		try!(self.ch_rdr.rdr.read_exact(data));
		self.packet_idx += 1;
		self.audio_chunk_offs += data.len() as i64;
		return Ok(());
	}

	/// Gets the number of packets if its known.
	pub fn get_packet_count(&self) -> Option<usize> {
		match &self.packet_table {
			&Some(ref t) => Some(t.lengths.len()),
			&None => match self.audio_desc.bytes_per_packet {
				// We are supposed to never reach this as the constructor
				// should enforce a packet table to be present if the
				// number of bytes per packet is unspecified.
				0 => panic!("No packet table was stored by the constructor"),
				// If the length of the audio chunk is unspecified,
				// and there is no packet table,
				// we won't know the count of packets.
				_ if self.audio_chunk_len == -1 => None,
				v => Some((self.audio_chunk_len as usize - 4) / v as usize),
			},
		}
	}

	/// Returns the index of the currently read packet
	pub fn get_packet_idx(&self) -> usize {
		self.packet_idx
	}

	/// Seeks to the packet with the given index
	///
	/// This function never has been tested.
	/// If there are bugs please report them.
	pub fn seek_to_packet(&mut self, packet_idx :usize) -> Result<(), CafError> {

		let min_idx = ::std::cmp::min(self.packet_idx, packet_idx);
		let max_idx = ::std::cmp::min(self.packet_idx, packet_idx);

		// The amount we need to seek by.
		let offs :i64 = match self.audio_desc.bytes_per_packet {
			0 => self.packet_table.as_ref()
				.unwrap().lengths[min_idx..max_idx].iter().map(|v| *v as i64).sum(),
			v => (max_idx - min_idx) as i64 * v as i64,
		};
		if self.packet_idx < packet_idx {
			try!(self.ch_rdr.rdr.seek(SeekFrom::Current(offs)));
		} else if self.packet_idx > packet_idx {
			try!(self.ch_rdr.rdr.seek(SeekFrom::Current(-offs)));
		} else {
			// No seek needed
		}
		Ok(())
	}
}
