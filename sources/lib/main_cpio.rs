

use ::argparse;
use ::cpio;
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
	
	let mut _nice_level = 19 as i8;
	
	
	{
		let mut _parser = argparse::ArgumentParser::new ();
		_hashes_flags.argparse (&mut _parser);
		_format_flags.argparse (&mut _parser);
		_parser.refer (&mut _nice_level) .add_option (&["--nice"], argparse::Parse, "OS process scheduling priority (i.e. `nice`) (19 by default)");
		_parser.parse_args_or_exit ();
	}
	
	
	if _nice_level != 0 {
		unsafe {
			// FIXME:  Check the return value!
			libc::nice (_nice_level as i32);
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
		
		let mut _record = cpio::newc::Reader::new (_input) ?;
		
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


pub fn main_0 () -> ! {
	if let Err (_error) = main () {
		eprintln! ("[!!] {}", _error);
		process::exit (1);
	} else {
		process::exit (0);
	}
}

