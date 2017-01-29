// CAF container decoder written in Rust
//
// Copyright (c) 2017 est31 <MTest31@outlook.com>
// and contributors. All rights reserved.
// Licensed under MIT license, or Apache 2 license,
// at your option. Please see the LICENSE file
// attached to this source distribution for details.

use std::string::FromUtf8Error;
use std::io::{Error as IoError};
use ::ChunkType;

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
