

use ::cpio::newc as cpio;
use ::libc;

use crate::digests::*;
use crate::flags::*;
use crate::hashes::*;
use crate::prelude::*;
use crate::sinks::*;




pub fn main () -> (Result<(), io::Error>) {
	
	let mut _hashes_flags = HashesFlags {
			algorithm : &MD5,
		};
	
	let mut _format_flags = HashesFormatFlags {
			zero : false,
		};
	
	{
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
							_hashes_flags.algorithm = &MD5,
						b"--sha1" =>
							_hashes_flags.algorithm = &SHA1,
						b"--sha224" | b"--sha2-224" =>
							_hashes_flags.algorithm = &SHA2_224,
						b"--sha256" | b"--sha2-256" =>
							_hashes_flags.algorithm = &SHA2_256,
						b"--sha384" | b"--sha2-384" =>
							_hashes_flags.algorithm = &SHA2_384,
						b"--sha512" | b"--sha2-512" =>
							_hashes_flags.algorithm = &SHA2_512,
						b"--sha3-224" =>
							_hashes_flags.algorithm = &SHA3_224,
						b"--sha3-256" =>
							_hashes_flags.algorithm = &SHA3_256,
						b"--sha3-384" =>
							_hashes_flags.algorithm = &SHA3_384,
						b"--sha3-512" =>
							_hashes_flags.algorithm = &SHA3_512,
						
						b"--zero" =>
							_format_flags.zero = true,
						
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
	}
	
	
	let mut _input = io::stdin ();
	let mut _input = _input.lock ();
	
	let mut _output = io::stdout ();
	let mut _output = _output.lock ();
	
	let mut _sink = StandardHashesSink::new (&mut _output, _format_flags.zero);
	
	let mut _hash_buffer = Vec::with_capacity (128);
	let mut _path_buffer = Vec::with_capacity (4 * 1024);
	
	
	loop {
		
		let mut _record = cpio::Reader::new (_input) ?;
		
		let _metadata = _record.entry ();
		if _metadata.is_trailer () {
			break;
		}
		
		if (_metadata.mode () & (libc::S_IFMT as u32)) == (libc::S_IFREG as u32) {
			
			let _hash = if (_metadata.file_size () > 0) || (_metadata.nlink () <= 1) {
				
				_hash_buffer.clear ();
				digest (_hashes_flags.algorithm, &mut _record, &mut _hash_buffer) ?;
				
			} else {
				
				eprintln! ("[ww] [7c9f8eb7]  hard-link detected: `{}`;  ignoring!", _metadata.name ());
				
				_hash_buffer.clear ();
				_hash_buffer.extend_from_slice (_hashes_flags.algorithm.invalid_raw);
			};
			
			let _metadata = _record.entry ();
			let _path = _metadata.name ();
			
			let _path_prefix =
				if _path.starts_with ("/") { "" }
				else if _path.starts_with ("./") { "" }
				else if _path.starts_with ("../") { "" }
				else { "./" };
			
			_path_buffer.clear ();
			_path_buffer.extend_from_slice (_path_prefix.as_bytes ());
			_path_buffer.extend_from_slice (_path.as_bytes ());
			
			_sink.handle (ffi::OsStr::from_bytes (&_path_buffer), &_hash_buffer) ?;
		}
		
		_input = _record.finish () ?;
	}
	
	return Ok (());
}

