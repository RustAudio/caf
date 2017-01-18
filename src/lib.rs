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
extern crate ieee754;

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
}
