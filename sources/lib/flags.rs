

use ::argparse;

use crate::hashes::*;




pub struct HashesFlags {
	pub algorithm : &'static HashAlgorithm,
}


impl <'a> HashesFlags {
	
	pub fn argparse (&'a mut self, _parser : &mut argparse::parser::ArgumentParser<'a>) -> () {
		_parser.refer (&mut self.algorithm)
				.add_option (&["--md5"], argparse::StoreConst (&MD5), "create/expect MD5 hashes")
				.add_option (&["--sha1"], argparse::StoreConst (&SHA1), "create/expect SHA1 hashes")
				.add_option (&["--sha224", "--sha2-224"], argparse::StoreConst (&SHA2_224), "create/expect SHA2-224 hashes")
				.add_option (&["--sha256", "--sha2-256"], argparse::StoreConst (&SHA2_256), "create/expect SHA2-256 hashes")
				.add_option (&["--sha384", "--sha2-384"], argparse::StoreConst (&SHA2_384), "create/expect SHA2-384 hashes")
				.add_option (&["--sha512", "--sha2-512"], argparse::StoreConst (&SHA2_512), "create/expect SHA2-512 hashes")
				.add_option (&["--sha3-224"], argparse::StoreConst (&SHA3_224), "create/expect SHA3-224 hashes")
				.add_option (&["--sha3-256"], argparse::StoreConst (&SHA3_256), "create/expect SHA3-256 hashes")
				.add_option (&["--sha3-384"], argparse::StoreConst (&SHA3_384), "create/expect SHA3-384 hashes")
				.add_option (&["--sha3-512"], argparse::StoreConst (&SHA3_512), "create/expect SHA3-512 hashes")
			;
	}
}




pub struct HashesFormatFlags {
	pub zero : bool,
}


impl <'a> HashesFormatFlags {
	
	pub fn argparse (&'a mut self, _parser : &mut argparse::parser::ArgumentParser<'a>) -> () {
		_parser.refer (&mut self.zero) .add_option (&["-z", "--zero"], argparse::StoreTrue, "delimit records by `\\0`");
	}
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


pub struct CompressionFlags {
	pub algorithm : CompressionAlgorithm,
}


impl <'a> CompressionFlags {
	
	pub fn argparse (&'a mut self, _parser : &mut argparse::parser::ArgumentParser<'a>) -> () {
		_parser.refer (&mut self.algorithm)
				.add_option (&["--gzip"], argparse::StoreConst (CompressionAlgorithm::Gzip), "create/expect `gzip` compression")
				.add_option (&["--bzip2"], argparse::StoreConst (CompressionAlgorithm::Bzip2), "create/expect `bzip2` compression")
				.add_option (&["--lzip"], argparse::StoreConst (CompressionAlgorithm::Lzip), "create/expect `lzip` compression")
				.add_option (&["--xz"], argparse::StoreConst (CompressionAlgorithm::Xz), "create/expect `xz` compression")
				.add_option (&["--lzma"], argparse::StoreConst (CompressionAlgorithm::Lzma), "create/expect `lzma` compression")
				.add_option (&["--lz4"], argparse::StoreConst (CompressionAlgorithm::Lz4), "create/expect `lz4` compression")
				.add_option (&["--lzo"], argparse::StoreConst (CompressionAlgorithm::Lzo), "create/expect `lzo` compression")
				.add_option (&["--zstd"], argparse::StoreConst (CompressionAlgorithm::Zstd), "create/expect `zstd` compression")
			;
	}
}

