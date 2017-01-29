// CAF container decoder written in Rust
//
// Copyright (c) 2017 est31 <MTest31@outlook.com>
// and contributors. All rights reserved.
// Licensed under MIT license, or Apache 2 license,
// at your option. Please see the LICENSE file
// attached to this source distribution for details.

use std::string::FromUtf8Error;
use std::io::{Error as IoError};
use std::error::Error;
use std::fmt::Display;
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

impl Error for CafError {
	fn description(&self) -> &str {
		use CafError::*;
		match self {
			&Io(_) => "IO error",
			&FromUtf8(_) => "Can't decode UTF-8",
			&NotCaf => "The given stream doesn't start with a CAF header",
			&UnsupportedChunkType(_) => "Encountered a chunk with an unsupported type",
		}
	}

	fn cause(&self) -> Option<&Error> {
		use CafError::*;
		match self {
			&Io(ref err) => Some(err as &Error),
			&FromUtf8(ref err) => Some(err as &Error),
			_ => None
		}
	}
}

impl Display for CafError {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
		use CafError::*;
		match *self {
			Io(ref err) => err.fmt(f),
			FromUtf8(ref err) => err.fmt(f),
			UnsupportedChunkType(_) |
			NotCaf => write!(f, "{}", self.description()),
		}
	}
}
