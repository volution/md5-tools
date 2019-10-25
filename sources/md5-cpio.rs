

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

use ::md5_tools::hashes::*;




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

