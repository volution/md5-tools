

use ::crossbeam;
use ::walkdir;

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
	
	let mut _path = ffi::OsString::from (".");
	
	let _threads_count = 16;
	let _queue_size = _threads_count * 1024;
	
	
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
		
		_path = _arguments.next () .unwrap ();
	}
	
	
	let _output = fs::OpenOptions::new () .write (true) .open ("/dev/stdout") ?;
	
	let _sink = StandardHashesSink::new (_output, _format_flags.zero);
	let _sink = sync::Arc::new (sync::Mutex::new (_sink));
	
	
	let mut _walker = walkdir::WalkDir::new (&_path)
			.follow_links (false)
			.same_file_system (false)
			.contents_first (true)
			.into_iter ();
	
	
	let (_enqueue, _dequeue) = crossbeam::channel::bounded::<walkdir::DirEntry> (_queue_size);
	let mut _completions = Vec::with_capacity (_threads_count);
	let _done = crossbeam::sync::WaitGroup::new ();
	
	
	for _ in 0 .. _threads_count {
		
		let _sink = sync::Arc::clone (&_sink);
		let _dequeue = _dequeue.clone ();
		let _done = _done.clone ();
		
		let _hashes_algorithm = _hashes_flags.algorithm;
		
		let _completion = thread::spawn (move || -> Result<(), io::Error> {
				
				let mut _hash_buffer = Vec::with_capacity (128);
				
				loop {
					
					let _entry = match _dequeue.recv () {
						Ok (_entry) =>
							_entry,
						Err (crossbeam::channel::RecvError) =>
							break,
					};
					
					let mut _file = fs::File::open (_entry.path ()) ?;
					
					_hash_buffer.clear ();
					digest (_hashes_algorithm, &mut _file, &mut _hash_buffer) ?;
					
					let mut _sink = _sink.lock () .unwrap ();
					_sink.handle (_entry.path () .as_os_str (), &_hash_buffer) ?;
				}
				
				drop (_done);
				
				return Ok (());
			});
		
		_completions.push (_completion);
	}
	
	
	loop {
		
		let _entry = if let Some (_entry) = _walker.next () {
			_entry ?
		} else {
			break;
		};
		
		let _metadata = _entry.metadata () ?;
		
		if _metadata.is_file () {
			_enqueue.send (_entry) .unwrap ();
		}
	}
	
	drop (_enqueue);
	drop (_dequeue);
	
	
	_done.wait ();
	
	for _completion in _completions.into_iter () {
		match _completion.join () {
			Ok (_outcome) =>
				_outcome ?,
			Err (_error) =>
				panic! (_error),
		}
	}
	
	return Ok (());
}

