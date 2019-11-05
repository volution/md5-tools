

use ::argparse;

use crate::hashes::*;
use crate::prelude::*;




#[ derive (Clone, Eq, PartialEq) ]
#[ derive (Default) ]
pub struct CreateFlags {
	
	pub source_path : path::PathBuf,
	pub output_path : path::PathBuf,
	
	pub relative : bool,
	
	pub walk_xdev : bool,
	pub walk_follow : bool,
	
	pub threads_count : usize,
	pub threads_nice : i8,
	
	pub queue_size : usize,
	pub batch_size : usize,
	pub batch_order : CreateBatchOrder,
	
	pub read_fadvise : bool,
	
	pub ignore_all_errors : bool,
	pub ignore_walk_errors : bool,
	pub ignore_open_errors : bool,
	pub ignore_read_errors : bool,
	
	pub report_progress : bool,
	pub report_errors_to_sink : bool,
	pub report_errors_to_stderr : bool,
	
	pub hashes_flags : HashesFlags,
	pub format_flags : HashesFormatFlags,
	
}


impl <'a> CreateFlags {
	
	pub fn argparse (&'a mut self, _parser : &mut argparse::parser::ArgumentParser<'a>) -> () {
		
		_parser.refer (&mut self.source_path) .add_argument ("source", argparse::Parse, "source file or folder") .required ();
		_parser.refer (&mut self.output_path) .add_option (&["--output", "-o"], argparse::Parse, "output file or folder (use `-` for stdout, `.` for auto-detection, or a destination file or folder) (`.` by default)");
		
		self.hashes_flags.argparse (_parser);
		
		self.format_flags.argparse (_parser);
		
		self.relative = true;
		
		_parser.refer (&mut self.relative)
				.add_option (&["--relative"], argparse::StoreTrue, "output paths relative to source (enabled by default)")
				.add_option (&["--no-relative"], argparse::StoreFalse, "");
		
		_parser.refer (&mut self.walk_xdev)
				.add_option (&["--xdev", "-x"], argparse::StoreTrue, "do not cross mount points (disabled by default)")
				.add_option (&["--no-xdev"], argparse::StoreFalse, "");
		
		_parser.refer (&mut self.walk_follow)
				.add_option (&["--follow", "-L"], argparse::StoreTrue, "follow symlinks (disabley by default) (n.b. source is always followed)")
				.add_option (&["--no-follow"], argparse::StoreFalse, "");
		
		_parser.refer (&mut self.threads_count) .add_option (&["--workers-count", "-w"], argparse::Parse, "hashing workers count (16 by default)");
		_parser.refer (&mut self.threads_nice) .add_option (&["--workers-nice"], argparse::Parse, "set OS process scheduling priority (i.e. `nice`) (19 by default)");
		
		self.batch_order = CreateBatchOrder::Inode;
		
		_parser.refer (&mut self.queue_size) .add_option (&["--workers-queue"], argparse::Parse, "hashing workers queue size (4096 times the workers count by default)");
		_parser.refer (&mut self.batch_size) .add_option (&["--workers-batch"], argparse::Parse, "hashing workers batch size (half the workers queue size by default)");
		_parser.refer (&mut self.batch_order) .add_option (&["--workers-sort"], argparse::Parse, "hashing workers batch sorting (use `walk`, `inode`, `inode-and-size`, or `extent`) (`inode` by default)");
		
		self.read_fadvise = true;
		
		_parser.refer (&mut self.read_fadvise)
				.add_option (&["--fadvise"], argparse::StoreTrue, "use OS `fadvise` with sequential and no-reuse (enabled by default)")
				.add_option (&["--no-fadvise"], argparse::StoreFalse, "");
		
		self.report_errors_to_sink = true;
		self.report_errors_to_stderr = true;
		
		_parser.refer (&mut self.report_errors_to_sink)
				.add_option (&["--errors-to-stdout"], argparse::StoreTrue, "on errors output an invalid hash (i.e. `00... */path/...`) (enabled by default)")
				.add_option (&["--no-errors-to-stdout"], argparse::StoreFalse, "");
		_parser.refer (&mut self.report_errors_to_stderr)
				.add_option (&["--errors-to-stderr"], argparse::StoreTrue, "on errors report a message (enabled by default)")
				.add_option (&["--no-errors-to-stderr"], argparse::StoreFalse, "");
		
		_parser.refer (&mut self.ignore_all_errors)
				.add_option (&["--ignore-all-errors"], argparse::StoreTrue, "ignore all errors (disabled by default)");
		_parser.refer (&mut self.ignore_walk_errors)
				.add_option (&["--ignore-walk-errors"], argparse::StoreTrue, "ignore walk errors (i.e. folder reading, perhaps due to permissions) (disabled by default)");
		_parser.refer (&mut self.ignore_open_errors)
				.add_option (&["--ignore-open-errors"], argparse::StoreTrue, "ignore open errors (i.e. file opening, perhaps due to permissions) (disabled by default)");
		_parser.refer (&mut self.ignore_read_errors)
				.add_option (&["--ignore-read-errors"], argparse::StoreTrue, "ignore open errors (i.e. file reading, perhaps due to I/O) (disabled by default)");
		
		self.report_progress = true;
		
		_parser.refer (&mut self.report_progress)
				.add_option (&["--progress"], argparse::StoreTrue, "monitor the progress (enabled by default)")
				.add_option (&["--no-progress"], argparse::StoreFalse, "");
	}
}




#[ derive (Copy, Clone, Eq, PartialEq) ]
pub enum CreateBatchOrder {
	Index,
	Inode,
	InodeAndSizeBuckets,
	Extent,
	Random,
}


impl Default for CreateBatchOrder {
	fn default () -> (Self) {
		CreateBatchOrder::Index
	}
}


impl argparse::FromCommandLine for CreateBatchOrder {
	
	fn from_argument (_value : &str) -> (Result<CreateBatchOrder, String>) {
		match _value {
			"index" | "walk" =>
				Ok (CreateBatchOrder::Index),
			"inode" | "" =>
				Ok (CreateBatchOrder::Inode),
			"inode-and-size" =>
				Ok (CreateBatchOrder::InodeAndSizeBuckets),
			"extent" =>
				Ok (CreateBatchOrder::Extent),
			"random" =>
				Ok (CreateBatchOrder::Random),
			_ =>
				Err (format! ("[3046e5fa]  invalid batch order `{}`", _value)),
		}
	}
}




#[ derive (Copy, Clone, Eq, PartialEq) ]
pub struct HashesFlags {
	pub algorithm : &'static HashAlgorithm,
}


impl Default for HashesFlags {
	fn default () -> (Self) {
		HashesFlags {
				algorithm : &MD5,
			}
	}
}


impl <'a> HashesFlags {
	
	pub fn argparse (&'a mut self, _parser : &mut argparse::parser::ArgumentParser<'a>) -> () {
		_parser.refer (&mut self.algorithm)
				.add_option (&["--md5"], argparse::StoreConst (&MD5), "create/expect MD5 hashes (enabled by default)")
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




#[ derive (Copy, Clone, Eq, PartialEq) ]
#[ derive (Default) ]
pub struct HashesFormatFlags {
	pub zero : bool,
}


impl <'a> HashesFormatFlags {
	
	pub fn argparse (&'a mut self, _parser : &mut argparse::parser::ArgumentParser<'a>) -> () {
		_parser.refer (&mut self.zero)
				.add_option (&["--zero", "-z"], argparse::StoreTrue, "delimit records by `\\0` (as opposed by `\\n`) (disabled by default)")
				.add_option (&["--no-zero"], argparse::StoreFalse, "");
	}
}




#[ derive (Copy, Clone, Eq, PartialEq) ]
#[ derive (Default) ]
pub struct CompressionFlags {
	pub algorithm : CompressionAlgorithm,
}


impl <'a> CompressionFlags {
	
	pub fn argparse (&'a mut self, _parser : &mut argparse::parser::ArgumentParser<'a>) -> () {
		_parser.refer (&mut self.algorithm)
				.add_option (&["--gzip"], argparse::StoreConst (CompressionAlgorithm::Gzip), "create/expect `gzip` compressed")
				.add_option (&["--bzip2"], argparse::StoreConst (CompressionAlgorithm::Bzip2), "create/expect `bzip2` compressed")
				.add_option (&["--lzip"], argparse::StoreConst (CompressionAlgorithm::Lzip), "create/expect `lzip` compressed")
				.add_option (&["--xz"], argparse::StoreConst (CompressionAlgorithm::Xz), "create/expect `xz` compressed")
				.add_option (&["--lzma"], argparse::StoreConst (CompressionAlgorithm::Lzma), "create/expect `lzma` compressed")
				.add_option (&["--lz4"], argparse::StoreConst (CompressionAlgorithm::Lz4), "create/expect `lz4` compressed")
				.add_option (&["--lzo"], argparse::StoreConst (CompressionAlgorithm::Lzo), "create/expect `lzo` compressed")
				.add_option (&["--zstd"], argparse::StoreConst (CompressionAlgorithm::Zstd), "create/expect `zstd` compressed")
				.add_option (&["--no-compression"], argparse::StoreConst (CompressionAlgorithm::None), "create/expect uncompressed (enabled by default)")
			;
	}
}




#[ derive (Copy, Clone, Eq, PartialEq) ]
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


impl Default for CompressionAlgorithm {
	fn default () -> (Self) {
		CompressionAlgorithm::None
	}
}

