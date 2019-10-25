

#![no_implicit_prelude]


use ::std::env;
use ::std::io;

use ::std::eprintln;
use ::std::format_args;

use ::std::io::Write as _;
use ::std::iter::Iterator as _;
use ::std::iter::IntoIterator as _;
use ::std::iter::ExactSizeIterator as _;
use ::std::option::{Option::Some, Option::None};
use ::std::result::{Result, Result::Ok, Result::Err};
use ::std::os::unix::ffi::OsStrExt as _;

use ::cpio::newc as cpio;
use ::libc;
use ::md5;
use ::sha1;
use ::sha2;
use ::sha3;
use ::digest;




enum HashAlgorithmKind {
	MD5,
	SHA1,
	SHA2_224,
	SHA2_256,
	SHA2_384,
	SHA2_512,
	SHA3_224,
	SHA3_256,
	SHA3_384,
	SHA3_512,
}

struct HashAlgorithm {
	kind : HashAlgorithmKind,
	name : &'static str,
	name_lower : &'static str,
	empty : &'static str,
	invalid : &'static str,
}




fn main () -> (Result<(), io::Error>) {
	
	let (_hash, _zero) = {
		
		let mut _hash = &MD5;
		let mut _zero = false;
		
		let _arguments = env::args_os ();
		let mut _arguments = _arguments.into_iter () .peekable ();
		
		loop {
			_arguments.next () .unwrap ();
			match _arguments.peek () {
				Some (_argument) =>
					match _argument.as_bytes () {
						
						b"--" => {
							_arguments.next () .unwrap ();
							break;
						},
						
						b"--md5" =>
							_hash = &MD5,
						b"--sha1" =>
							_hash = &SHA1,
						b"--sha224" | b"--sha2-224" =>
							_hash = &SHA2_224,
						b"--sha256" | b"--sha2-256" =>
							_hash = &SHA2_256,
						b"--sha384" | b"--sha2-384" =>
							_hash = &SHA2_384,
						b"--sha512" | b"--sha2-512" =>
							_hash = &SHA2_512,
						b"--sha3-224" =>
							_hash = &SHA3_224,
						b"--sha3-256" =>
							_hash = &SHA3_256,
						b"--sha3-384" =>
							_hash = &SHA3_384,
						b"--sha3-512" =>
							_hash = &SHA3_512,
						
						b"--zero" =>
							_zero = true,
						
						b"" =>
							return Err (io::Error::new (io::ErrorKind::Other, "[c80572b3]  unexpected empty argument")),
						_argument if _argument[0] == b'-' =>
							return Err (io::Error::new (io::ErrorKind::Other, "[63a73c9c]  unexpected flag")),
						_ =>
							break,
					},
				None =>
					break,
			}
		}
		
		if _arguments.len () != 0 {
			return Err (io::Error::new (io::ErrorKind::Other, "[f084735b]  unexpected arguments"));
		}
		
		(_hash, _zero)
	};
	
	let mut _input = io::stdin ();
	let mut _input = _input.lock ();
	
	let mut _output = io::stdout ();
	let mut _output = _output.lock ();
	
	let mut _output = io::BufWriter::with_capacity (16 * 1024 * 1024, _output);
	
	let _delimiter = if _zero { b"\0" } else { b"\n" };
	
	loop {
		
		let mut _record = cpio::Reader::new (_input) ?;
		
		let _metadata = _record.entry ();
		if _metadata.is_trailer () {
			break;
		}
		
		if (_metadata.mode () & libc::S_IFMT) == libc::S_IFREG {
			
			let _hash = if (_metadata.file_size () > 0) || (_metadata.nlink () <= 1) {
				match _hash.kind {
					HashAlgorithmKind::MD5 =>
						digest::<md5::Md5, _, _> (&mut _record, &mut _output) ?,
					HashAlgorithmKind::SHA1 =>
						digest::<sha1::Sha1, _, _> (&mut _record, &mut _output) ?,
					HashAlgorithmKind::SHA2_224 =>
						digest::<sha2::Sha224, _, _> (&mut _record, &mut _output) ?,
					HashAlgorithmKind::SHA2_256 =>
						digest::<sha2::Sha256, _, _> (&mut _record, &mut _output) ?,
					HashAlgorithmKind::SHA2_384 =>
						digest::<sha2::Sha384, _, _> (&mut _record, &mut _output) ?,
					HashAlgorithmKind::SHA2_512 =>
						digest::<sha2::Sha512, _, _> (&mut _record, &mut _output) ?,
					HashAlgorithmKind::SHA3_224 =>
						digest::<sha3::Sha3_224, _, _> (&mut _record, &mut _output) ?,
					HashAlgorithmKind::SHA3_256 =>
						digest::<sha3::Sha3_256, _, _> (&mut _record, &mut _output) ?,
					HashAlgorithmKind::SHA3_384 =>
						digest::<sha3::Sha3_384, _, _> (&mut _record, &mut _output) ?,
					HashAlgorithmKind::SHA3_512 =>
						digest::<sha3::Sha3_512, _, _> (&mut _record, &mut _output) ?,
				}
			} else {
				eprintln! ("[ww] [7c9f8eb7]  hard-link detected: `{}`;  ignoring!", _metadata.name ());
				_output.write_all (_hash.invalid.as_bytes ()) ?;
			};
			
			let _metadata = _record.entry ();
			let _path = _metadata.name ();
			
			let _path_prefix =
				if _path.starts_with ("/") { "" }
				else if _path.starts_with ("./") { "" }
				else if _path.starts_with ("../") { "" }
				else { "./" };
			
			_output.write_all (b" *") ?;
			_output.write_all (_path.as_bytes ()) ?;
			_output.write_all (_delimiter) ?;
		}
		
		_input = _record.finish () ?;
	}
	
	return Ok (());
}




fn digest <Hash : digest::Digest + io::Write, Input : io::Read, Output : io::Write> (_input : &mut Input, _output : &mut io::BufWriter<Output>) -> (io::Result<()>)
{
	
	let mut _hasher = Hash::new ();
	io::copy (_input, &mut _hasher) ?;
	let _hash = _hasher.result ();
	
	for _byte in _hash.as_slice () {
		_output.write_fmt (format_args! ("{:02x}", _byte)) ?;
	}
	
	return Ok (());
}




static MD5 : HashAlgorithm = HashAlgorithm {
		kind : HashAlgorithmKind::MD5,
		name : "MD5", name_lower : "md5",
		empty : "d41d8cd98f00b204e9800998ecf8427e",
		invalid : "00000000000000000000000000000000",
	};


static SHA1 : HashAlgorithm = HashAlgorithm {
		kind : HashAlgorithmKind::SHA1,
		name : "SHA1", name_lower : "sha1",
		empty : "da39a3ee5e6b4b0d3255bfef95601890afd80709",
		invalid : "0000000000000000000000000000000000000000",
	};


static SHA2_224 : HashAlgorithm = HashAlgorithm {
		kind : HashAlgorithmKind::SHA2_224,
		name : "SHA224", name_lower : "sha224",
		empty : "d14a028c2a3a2bc9476102bb288234c415a2b01f828ea62ac5b3e42f",
		invalid : "0000000000000000000000000000000000000000000000000000000",
	};

static SHA2_256 : HashAlgorithm = HashAlgorithm {
		kind : HashAlgorithmKind::SHA2_256,
		name : "SHA256", name_lower : "sha256",
		empty : "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
		invalid : "0000000000000000000000000000000000000000000000000000000000000000",
	};

static SHA2_384 : HashAlgorithm = HashAlgorithm {
		kind : HashAlgorithmKind::SHA2_384,
		name : "SHA384", name_lower : "sha384",
		empty : "38b060a751ac96384cd9327eb1b1e36a21fdb71114be07434c0cc7bf63f6e1da274edebfe76f65fbd51ad2f14898b95b",
		invalid : "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
	};

static SHA2_512 : HashAlgorithm = HashAlgorithm {
		kind : HashAlgorithmKind::SHA2_512,
		name : "SHA512", name_lower : "sha512",
		empty : "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e",
		invalid : "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
	};


static SHA3_224 : HashAlgorithm = HashAlgorithm {
		kind : HashAlgorithmKind::SHA3_224,
		name : "SHA3-224", name_lower : "sha3-224",
		empty : "6b4e03423667dbb73b6e15454f0eb1abd4597f9a1b078e3f5b5a6bc7",
		invalid : "0000000000000000000000000000000000000000000000000000000",
	};

static SHA3_256 : HashAlgorithm = HashAlgorithm {
		kind : HashAlgorithmKind::SHA3_256,
		name : "SHA3-256", name_lower : "sha3-256",
		empty : "a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a",
		invalid : "0000000000000000000000000000000000000000000000000000000000000000",
	};

static SHA3_384 : HashAlgorithm = HashAlgorithm {
		kind : HashAlgorithmKind::SHA3_384,
		name : "SHA3-384", name_lower : "sha3-384",
		empty : "0c63a75b845e4f7d01107d852e4c2485c51a50aaaa94fc61995e71bbee983a2ac3713831264adb47fb6bd1e058d5f004",
		invalid : "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
	};

static SHA3_512 : HashAlgorithm = HashAlgorithm {
		kind : HashAlgorithmKind::SHA3_512,
		name : "SHA3-512", name_lower : "sha3-512",
		empty : "a69f73cca23a9ac5c8b567dc185a756e97c982164fe25859e0d1dcc1475c80a615b2123af1f5f94c11e3e9402c3ac558f500199d95b6d3e301758586281dcd26",
		invalid : "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
	};

