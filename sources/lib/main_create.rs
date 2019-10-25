

use ::walkdir;

use crate::digests::*;
use crate::hashes::*;
use crate::prelude::*;
use crate::sinks::*;




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
	
	let mut _sink = StandardHashesSink::new (&mut _output, _zero);
	let mut _hash_buffer = Vec::with_capacity (128);
	
	
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
			
			_hash_buffer.clear ();
			digest (_hash, &mut _file, &mut _hash_buffer) ?;
			
			_sink.handle (_entry.path () .as_os_str (), &_hash_buffer) ?;
		}
	}
	
	return Ok (());
}

