

use crate::hashes::*;




pub struct HashesFlags {
	pub algorithm : &'static HashAlgorithm,
}

pub struct HashesFormatFlags {
	pub zero : bool,
}




pub struct CompressionFlags {
	pub algorithm : CompressionAlgorithm,
}

#[ derive (Copy, Clone, PartialEq) ]
pub enum CompressionAlgorithm {
	None,
	Gzip,  // https://www.gzip.org/
	Bzip2, // http://sourceware.org/bzip2/
	Lzip,  // https://www.nongnu.org/lzip/
	Xz,    // https://tukaani.org/xz/
	Lzma,  // https://www.7-zip.org/sdk.html
	Lz4,   // https://lz4.github.io/lz4/
	Lzo,   // http://www.lzop.org/
	Zstd,  // https://github.com/facebook/zstd
}

