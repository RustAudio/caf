// CAF container decoder written in Rust
//
// Copyright (c) 2017 est31 <MTest31@outlook.com>
// and contributors. All rights reserved.
// Licensed under MIT license, or Apache 2 license,
// at your option. Please see the LICENSE file
// attached to this source distribution for details.

/*!
Module with some (sparse) enums

In muliple places, the spec provides lists of IDs, saying
that the list is non exhaustive.
*/

/// Module containing the different specified chunk types
///
/// Beware, the spec explicitly says that its list is non exhaustive.
mod chunk_types {
	// The order is not random, its how it appears in the spec, bear this in mind.
	// The spec says that this list is not exhaustive, so we can't use an enum here.
	// Especially, users may add their own custom chunk types provided those are
	// outside of the reserved space of identifiers.

	pub const AUDIO_DESCRIPTION :u32 = 0x64_65_73_63; // "desc"
	pub const AUDIO_DATA :u32 = 0x64_61_74_61; // "data"
	pub const PACKET_TABLE :u32 = 0x70_61_6b_74; // "pakt"
	pub const CHANNEL_LAYOUT :u32 = 0x63_68_61_6e; // "chan"
	pub const MAGIC_COOKIE :u32 = 0x6b_75_6b_69; // "kuki"
	pub const STRINGS :u32 = 0x73_74_42_67; // "strg"
	pub const MARKER :u32 = 0x6d_61_72_6b; // "mark"
	pub const REGION :u32 = 0x72_65_67_6e; // "regn"
	pub const INSTRUMENT :u32 = 0x69_6e_73_74; // "inst"
	pub const MIDI :u32 = 0x6d_69_64_69; // "midi"
	pub const OVERVIEW :u32 = 0x6f_76_76_77; // "ovvw"
	pub const PEAK :u32 = 0x70_65_61_6b; // "peak"
	pub const EDIT_COMMENTS :u32 = 0x65_64_63_74; // "edct"
	pub const INFO :u32 = 0x69_6e_66_6f; // "info"
	pub const UNIQUE_MATERIAL_IDENTIFIER :u32 = 0x75_6d_69_64; // "umid"
	pub const USER_DEFINED :u32 = 0x75_75_69_64; // "uuid"
	pub const FREE :u32 = 0x66_72_65_65; // "free"
}

/// Possible chunk types defined by the spec
///
/// The chunks in a CAF file after the CAF File Header form the
/// uppermost layer of granularity.
///
/// The spec explicitly says that the list is not exhaustive
/// and that users may add their own unofficial chunk types
/// from outside of the reserved range of chunks.
/// Those chunk types are represented by the `Other` variant.
#[derive(Debug, Clone, Copy)]
pub enum ChunkType {
	/// mChunkType for the "Audio Description" chunk
	AudioDescription,
	/// mChunkType for the "Audio Data" chunk
	AudioData,
	/// mChunkType for the "Packet Table" chunk
	PacketTable,
	/// mChunkType for the "Channel Layout" chunk
	ChannelLayout,
	/// mChunkType for the "Magic Cookie" chunk
	MagicCookie,
	/// mChunkType for the "Strings" chunk
	Strings,
	/// mChunkType for the "Marker" chunk
	Marker,
	/// mChunkType for the "Region" chunk
	Region,
	/// mChunkType for the "Instrument" chunk
	Instrument,
	/// mChunkType for the "MIDI" chunk
	Midi,
	/// mChunkType for the "Overview" chunk
	Overview,
	/// mChunkType for the "Peak" chunk
	Peak,
	/// mChunkType for the "Edit Comments" chunk
	EditComments,
	/// mChunkType for the "Information" chunk
	Info,
	/// mChunkType for the "Unique Material Identifier" chunk
	UniqueMaterialIdentifier,
	/// mChunkType for the "User-Defined" chunk
	UserDefined,
	/// mChunkType for the "Free" chunk
	Free,
	/// Variant for all chunks that were not mentioned in this list.
	///
	/// This includes both chunk types from the range of reserved
	/// chunk types that weren't mentioned, and those from outside
	/// the range of reserved ones.
	Other(u32),
}

impl From<u32> for ChunkType {
	fn from(v :u32) -> Self {
		use self::chunk_types::*;
		use self::ChunkType::*;
		match v {
			AUDIO_DESCRIPTION => AudioDescription,
			AUDIO_DATA => AudioData,
			PACKET_TABLE => PacketTable,
			CHANNEL_LAYOUT => ChannelLayout,
			MAGIC_COOKIE => MagicCookie,
			STRINGS => Strings,
			MARKER => Marker,
			REGION => Region,
			INSTRUMENT => Instrument,
			MIDI => Midi,
			OVERVIEW => Overview,
			PEAK => Peak,
			EDIT_COMMENTS => EditComments,
			INFO => Info,
			UNIQUE_MATERIAL_IDENTIFIER => UniqueMaterialIdentifier,
			USER_DEFINED => UserDefined,
			FREE => Free,
			_ => Other(v),
		}
	}
}

/// Module containing the different specified chunk types
///
/// Beware, the spec explicitly says that its list is non exhaustive.
mod format_types {
	// The order is not random, its how it appears in the spec, bear this in mind.
	// The spec says that this list is not exhaustive, so we can't use an enum here.


	pub const LINEAR_PCM :u32 = 0x6c_70_63_6d; // "lpcm"
	pub const APPLE_IMA4 :u32 = 0x69_6d_61_34; // "ima4"
	pub const MPEG4_AAC :u32 = 0x61_61_63_20; // "aac "
	pub const MACE3 :u32 = 0x4d_41_43_33; // "MAC3"
	pub const MACE6 :u32 = 0x4d_41_43_36; // "MAC6"
	pub const U_LAW :u32 = 0x75_6c_61_77; // "ulaw"
	pub const A_LAW :u32 = 0x61_6c_61_77; // "alaw"
	pub const MPEG_LAYER_1 :u32 = 0x2e_6d_70_31; // ".mp1"
	pub const MPEG_LAYER_2 :u32 = 0x2e_6d_70_32; // ".mp2"
	pub const MPEG_LAYER_3 :u32 = 0x2e_6d_70_33; // ".mp3"
	pub const AAPL_LOSSLESS :u32 = 0x61_6c_61_63; // "alac"
}

/// Payload format types defined by the spec
///
/// Enum for all the possible `mFormatID` field contents
/// defined by the spec.
///
/// The spec explicitly says that the list is not exhaustive.
#[derive(Debug)]
pub enum FormatType {
	/// mFormatID for Linear PCM
	LinearPcm,
	/// mFormatID for IMA 4:1 ADPCM
	AppleIma4,
	/// mFormatID for MPEG-4 AAC
	Mpeg4Aac,
	/// mFormatID for MACE 3:1
	Mace3,
	/// mFormatID for MACE 6:1
	Mace6,
	/// mFormatID for uLaw 2:1
	Ulaw,
	/// mFormatID for aLaw 2:1
	Alaw,
	/// mFormatID for MPEG-1
	MpegLayer1,
	/// mFormatID for MPEG-{1,2}
	MpegLayer2,
	/// mFormatID for MPEG-{1,2,3}
	MpegLayer3,
	/// mFormatID for Apple Lossless
	AppleLossless,
	/// Variant for all formats that were not mentioned in this list.
	Other(u32),
}

impl From<u32> for FormatType {
	fn from(v :u32) -> Self {
		use self::format_types::*;
		use self::FormatType::*;
		match v {
			LINEAR_PCM => LinearPcm,
			APPLE_IMA4 => AppleIma4,
			MPEG4_AAC => Mpeg4Aac,
			MACE3 => Mace3,
			MACE6 => Mace6,
			U_LAW => Ulaw,
			A_LAW => Alaw,
			MPEG_LAYER_1 => MpegLayer1,
			MPEG_LAYER_2 => MpegLayer2,
			MPEG_LAYER_3 => MpegLayer3,
			AAPL_LOSSLESS => AppleLossless,
			_ => Other(v),
		}
	}
}
