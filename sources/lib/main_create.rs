

use ::argparse;
use ::crossbeam;
use ::libc;
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
	
	let mut _path = path::PathBuf::from ("");
	
	let mut _threads_count = 0 as usize;
	let mut _queue_size = 0 as usize;
	let mut _nice_level = 19 as i8;
	let mut _walk_xdev = false;
	let mut _walk_follow = false;
	
	let mut _ignore_all_errors = false;
	let mut _ignore_walk_errors = false;
	let mut _ignore_open_errors = false;
	let mut _ignore_read_errors = false;
	let mut _report_errors_to_sink = true;
	let mut _report_errors_to_stderr = true;
	
	{
		let mut _parser = argparse::ArgumentParser::new ();
		_hashes_flags.argparse (&mut _parser);
		_format_flags.argparse (&mut _parser);
		_parser.refer (&mut _walk_xdev) .add_option (&["-x", "--xdev"], argparse::StoreTrue, "do not cross mount points");
		_parser.refer (&mut _walk_follow) .add_option (&["-L", "--follow"], argparse::StoreTrue, "follow symlinks (n.b. arguments are followed)");
		_parser.refer (&mut _threads_count) .add_option (&["-w", "--workers-count"], argparse::Parse, "hashing workers count (16 by default)");
		_parser.refer (&mut _queue_size) .add_option (&["--workers-queue"], argparse::Parse, "hashing workers queue size (1024 times workers count by default)");
		_parser.refer (&mut _nice_level) .add_option (&["--nice"], argparse::Parse, "OS process scheduling priority (i.e. `nice`) (19 by default)");
		_parser.refer (&mut _ignore_all_errors)
				.add_option (&["--ignore-all-errors"], argparse::StoreTrue, "ignore all errors (false by default)");
		_parser.refer (&mut _ignore_walk_errors)
				.add_option (&["--ignore-walk-errors"], argparse::StoreTrue, "ignore walk errors (i.e. folder reading, perhaps due to permissions) (false by default)");
		_parser.refer (&mut _ignore_open_errors)
				.add_option (&["--ignore-open-errors"], argparse::StoreTrue, "ignore open errors (i.e. file opening, perhaps due to permissions) (false by default)");
		_parser.refer (&mut _ignore_read_errors)
				.add_option (&["--ignore-read-errors"], argparse::StoreTrue, "ignore open errors (i.e. file reading, perhaps due to I/O) (false by default)");
		_parser.refer (&mut _report_errors_to_sink)
				.add_option (&["--errors-to-stdout"], argparse::StoreTrue, "on errors output an invalid hash (i.e. `00... */path/...`) (true by default)")
				.add_option (&["--no-errors-to-stdout"], argparse::StoreFalse, "on errors do not output an invalid hash");
		_parser.refer (&mut _report_errors_to_stderr)
				.add_option (&["--errors-to-stderr"], argparse::StoreTrue, "on errors report a message (true by default)")
				.add_option (&["--no-errors-to-stderr"], argparse::StoreFalse, "on errors report a message");
		_parser.refer (&mut _path) .add_argument ("path", argparse::Parse, "starting file or folder") .required ();
		_parser.parse_args_or_exit ();
	}
	
	
	if _threads_count == 0 {
		_threads_count = 16;
	}
	if _queue_size == 0 {
		_queue_size = _threads_count * 1024;
	}
	if _ignore_all_errors {
		_ignore_walk_errors = true;
		_ignore_open_errors = true;
		_ignore_read_errors = true;
	}
	
	
	if _nice_level != 0 {
		unsafe {
			// FIXME:  Check the return value!
			libc::nice (_nice_level as i32);
		}
	}
	
	
	let _output = fs::OpenOptions::new () .write (true) .open ("/dev/stdout") ?;
	
	let _sink = StandardHashesSink::new (_output, _format_flags.zero);
	let _sink = sync::Arc::new (sync::Mutex::new (_sink));
	
	
	let (_enqueue, _dequeue) = crossbeam::channel::bounded::<walkdir::DirEntry> (_queue_size);
	let mut _completions = Vec::with_capacity (_threads_count);
	let _threads_errors = sync::Arc::new (sync::Mutex::new (Vec::new ()));
	let _done = crossbeam::sync::WaitGroup::new ();
	
	
	for _ in 0 .. _threads_count {
		
		let _sink = sync::Arc::clone (&_sink);
		let _dequeue = _dequeue.clone ();
		let _threads_errors = sync::Arc::clone (&_threads_errors);
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
					
					let _path = _entry.path ();
					
					let mut _open = fs::OpenOptions::new ();
					_open.read (true);
					
					let mut _file = match _open.open (_path) {
						Ok (_file) =>
							_file,
						Err (_error) => {
							let mut _sink = _sink.lock () .unwrap ();
							if _report_errors_to_stderr {
								eprintln! ("[ee] [42f1352f]  failed opening file `{}`: `{}`!", _path.to_string_lossy (), _error);
							}
							if _report_errors_to_sink {
								_sink.handle (_path.as_os_str (), _hashes_algorithm.invalid_raw) ?;
								_sink.flush () ?;
							}
							_threads_errors.lock () .unwrap () .push (_error);
							if _ignore_open_errors {
								continue;
							} else {
								return Ok (());
							}
						},
					};
					
					_hash_buffer.clear ();
					match digest (_hashes_algorithm, &mut _file, &mut _hash_buffer) {
						Ok (()) => {
							let mut _sink = _sink.lock () .unwrap ();
							_sink.handle (_path.as_os_str (), &_hash_buffer) ?;
						},
						Err (_error) => {
							let mut _sink = _sink.lock () .unwrap ();
							if _report_errors_to_stderr {
								eprintln! ("[ee] [1aeb2750]  failed reading file `{}`: `{}`!", _path.to_string_lossy (), _error);
							}
							if _report_errors_to_sink {
								_sink.handle (_path.as_os_str (), _hashes_algorithm.invalid_raw) ?;
								_sink.flush () ?;
							}
							_threads_errors.lock () .unwrap () .push (_error);
							if _ignore_read_errors {
								continue;
							} else {
								return Ok (());
							}
						},
					}
					
				}
				
				drop (_done);
				
				return Ok (());
			});
		
		_completions.push (_completion);
	}
	
	
	let mut _walker = walkdir::WalkDir::new (&_path)
			.same_file_system (_walk_xdev)
			.follow_links (_walk_follow)
			.contents_first (true)
			.into_iter ();
	
	let mut _errors = Vec::<io::Error>::new ();
	let _unknown_error = io::Error::new (io::ErrorKind::Other, "[31b7b284]  unexpected error");
	
	
	loop {
		
		let _entry = match _walker.next () {
			Some (Ok (_entry)) =>
				_entry,
			Some (Err (_error)) => {
				let mut _sink = _sink.lock () .unwrap ();
				let _path = _error.path () .unwrap_or (&_path);
				if let Some (_ancestor) = _error.loop_ancestor () {
					eprintln! ("[ww] [55021f5c]  detected walking loop for `{}` pointing at `{}`;  ignoring!", _path.to_string_lossy (), _ancestor.to_string_lossy ());
					continue;
				}
				if _report_errors_to_stderr {
					eprintln! ("[ee] [a5e88e25]  failed walking path `{}`: `{}`!", _path.to_string_lossy (), _error.io_error () .unwrap_or (&_unknown_error));
				}
				if _report_errors_to_sink {
					_sink.handle (_path.as_os_str (), _hashes_flags.algorithm.invalid_raw) ?;
					_sink.flush () ?;
				}
				if _ignore_walk_errors {
					continue;
				} else {
					let _error = _error.into_io_error () .unwrap_or_else (|| io::Error::new (io::ErrorKind::Other, "[7961fa68]  unexpected error"));
					_errors.push (_error);
					break;
				}
			},
			None =>
				break,
		};
		
		let _metadata = match _entry.metadata () {
			Ok (_metadata) =>
				_metadata,
			Err (_error) => {
				let mut _sink = _sink.lock () .unwrap ();
				let _path = _error.path () .unwrap_or (&_path);
				if _report_errors_to_stderr {
					eprintln! ("[ee] [96d2838a]  failed walking path `{}`: `{}`!", _entry.path () .to_string_lossy (), _error.io_error () .unwrap_or (&_unknown_error));
				}
				if _report_errors_to_sink {
					_sink.handle (_entry.path () .as_os_str (), _hashes_flags.algorithm.invalid_raw) ?;
					_sink.flush () ?;
				}
				if _ignore_walk_errors {
					continue;
				} else {
					let _error = _error.into_io_error () .unwrap_or_else (|| io::Error::new (io::ErrorKind::Other, "[7961fa68]  unexpected error"));
					_errors.push (_error);
					break;
				}
			},
		};
		
		if _metadata.is_file () {
			_enqueue.send (_entry) .unwrap ();
		}
	}
	
	drop (_enqueue);
	drop (_dequeue);
	
	
	_done.wait ();
	
	for _completion in _completions.into_iter () {
		match _completion.join () {
			Ok (Ok (())) =>
				(),
			Ok (Err (_error)) =>
				_errors.push (_error),
			Err (_error) =>
				_errors.push (io::Error::new (io::ErrorKind::Other, "[ee3e2b02]  unexpected error")),
		}
	}
	
	{
		let mut _threads_errors = _threads_errors.lock () .unwrap ();
		while let Some (_error) = _threads_errors.pop () {
			_errors.push (_error);
		}
	}
	
	if _errors.is_empty () {
		return Ok (());
	} else {
		return Err (io::Error::new (io::ErrorKind::Other, format! ("[32f6fc78]  encountered {} errors", _errors.len ())));
	}
}


pub fn main_0 () -> ! {
	if let Err (_error) = main () {
		eprintln! ("[!!] {}", _error);
		process::exit (1);
	} else {
		process::exit (0);
	}
}

