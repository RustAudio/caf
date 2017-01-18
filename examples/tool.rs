// CAF container decoder written in Rust
//
// This example file is licensed
// under the CC-0 license:
// https://creativecommons.org/publicdomain/zero/1.0/

extern crate caf;
use std::fs::File;
use caf::chunks::{CafChunk};
use caf::{CafChunkReader};
use std::env;

fn main() {
	let file_path = env::args().nth(1).expect("No arg found. Please specify a file to open.");
	println!("Opening file: {}", file_path);
	let f_rdr = File::open(file_path).unwrap();
	let mut rdr = CafChunkReader::new(f_rdr).unwrap();
	// Dump the decoded packets.
	loop {
		let chunk = rdr.read_chunk().unwrap();
		match chunk {
			CafChunk::AudioDataInMemory(..) => println!("Audio data in memory"),
			_ => println!("{:?}", chunk),
		}
	}
}
