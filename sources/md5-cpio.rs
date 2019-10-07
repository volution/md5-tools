

#![no_implicit_prelude]


use ::std::env;
use ::std::io;

use ::cpio::newc as cpio;
use ::libc;
use ::md5;

use ::std::eprintln;
use ::std::panic;
use ::std::println;

use ::std::iter::ExactSizeIterator as _;
use ::std::result::{Result, Result::Ok};
use ::std::option::{Option::Some, Option::None};




fn main () -> Result<(), io::Error> {
	
	if env::args () .len () != 1 {
		panic! ("[f084735b]  unexpected arguments!");
	}
	
	let mut _input = io::stdin ();
	let mut _input = _input.lock ();
	
	let mut _output = io::stdout ();
	let mut _output = _output.lock ();
	
	loop {
		
		let mut _record = cpio::Reader::new (_input) ?;
		
		let _metadata = _record.entry ();
		if _metadata.is_trailer () {
			break;
		}
		
		if (_metadata.mode () & libc::S_IFMT) == libc::S_IFREG {
			
			let _hash = if _metadata.file_size () > 0 {
				
				let mut _hasher = md5::Context::new ();
				io::copy (&mut _record, &mut _hasher) ?;
				
				let _hash = _hasher.compute ();
				Some (_hash)
			} else {
				None
			};
			
			let _metadata = _record.entry ();
			let _path = _metadata.name ();
			
			let _path_prefix =
				if _path.starts_with ("/") { "" }
				else if _path.starts_with ("./") { "" }
				else if _path.starts_with ("../") { "" }
				else { "./" };
			
			if let Some (_hash) = _hash {
				println! ("{:x} *{}{}", _hash, _path_prefix, _path);
			} else {
				let _hash = "00000000000000000000000000000000";
				println! ("{} *{}{}", _hash, _path_prefix, _path);
				eprintln! ("[ww] [7c9f8eb7]  hard-link detected: `{}`;  ignoring!", _metadata.name ());
			}
		}
		
		_input = _record.finish () ?;
	}
	
	return Ok (());
}

