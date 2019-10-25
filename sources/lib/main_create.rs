

use ::std::env;
use ::std::fs;
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

use ::walkdir;
use ::libc;
use ::md5;
use ::sha1;
use ::sha2;
use ::sha3;
use ::digest;

use crate::hashes::*;




pub fn main () -> (Result<(), io::Error>) {
	
	let (_path, _hash, _zero) = {
		
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
							return Err (io::Error::new (io::ErrorKind::Other, "[82eb61b2]  unexpected empty argument")),
						_argument if _argument[0] == b'-' =>
							return Err (io::Error::new (io::ErrorKind::Other, "[272d8b13]  unexpected flag")),
						_ =>
							break,
					},
				None =>
					break,
			}
		}
		
		if _arguments.len () != 1 {
			return Err (io::Error::new (io::ErrorKind::Other, "[92cf6a8d]  unexpected arguments"));
		}
		
		let _path = _arguments.next () .unwrap ();
		
		(_path, _hash, _zero)
	};
	
	let mut _output = io::stdout ();
	let mut _output = _output.lock ();
	
	let mut _output = io::BufWriter::with_capacity (16 * 1024 * 1024, _output);
	
	let _delimiter = if _zero { b"\0" } else { b"\n" };
	
	
	let mut _walker = walkdir::WalkDir::new (&_path)
			.follow_links (false)
			.same_file_system (false)
			.contents_first (true)
			.into_iter ();
	
	
	loop {
		
		let _entry = if let Some (_entry) = _walker.next () {
			_entry ?
		} else {
			break;
		};
		
		let _metadata = _entry.metadata () ?;
		
		if _metadata.is_file () {
			
			let mut _file = fs::File::open (_entry.path ()) ?;
			
			match _hash.kind {
				HashAlgorithmKind::MD5 =>
					digest::<md5::Md5, _, _> (&mut _file, &mut _output) ?,
				HashAlgorithmKind::SHA1 =>
					digest::<sha1::Sha1, _, _> (&mut _file, &mut _output) ?,
				HashAlgorithmKind::SHA2_224 =>
					digest::<sha2::Sha224, _, _> (&mut _file, &mut _output) ?,
				HashAlgorithmKind::SHA2_256 =>
					digest::<sha2::Sha256, _, _> (&mut _file, &mut _output) ?,
				HashAlgorithmKind::SHA2_384 =>
					digest::<sha2::Sha384, _, _> (&mut _file, &mut _output) ?,
				HashAlgorithmKind::SHA2_512 =>
					digest::<sha2::Sha512, _, _> (&mut _file, &mut _output) ?,
				HashAlgorithmKind::SHA3_224 =>
					digest::<sha3::Sha3_224, _, _> (&mut _file, &mut _output) ?,
				HashAlgorithmKind::SHA3_256 =>
					digest::<sha3::Sha3_256, _, _> (&mut _file, &mut _output) ?,
				HashAlgorithmKind::SHA3_384 =>
					digest::<sha3::Sha3_384, _, _> (&mut _file, &mut _output) ?,
				HashAlgorithmKind::SHA3_512 =>
					digest::<sha3::Sha3_512, _, _> (&mut _file, &mut _output) ?,
			}
			
			let _path = _entry.path ();
			
			let _path_prefix =
				if _path.starts_with ("/") { "" }
				else if _path.starts_with ("./") { "" }
				else if _path.starts_with ("../") { "" }
				else { "./" };
			
			_output.write_all (b" *") ?;
			_output.write_all (_path.as_os_str () .as_bytes ()) ?;
			_output.write_all (_delimiter) ?;
		}
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

