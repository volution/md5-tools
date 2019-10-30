

use ::argparse;
use ::chrono;
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
	
	let mut _output_path = path::PathBuf::from ("");
	let mut _source_path = path::PathBuf::from ("");
	
	let mut _relative = true;
	let mut _walk_xdev = false;
	let mut _walk_follow = false;
	let mut _threads_count = 0 as usize;
	let mut _queue_size = 0 as usize;
	let mut _batch_size = 0 as usize;
	let mut _nice_level = 19 as i8;
	
	let mut _ignore_all_errors = false;
	let mut _ignore_walk_errors = false;
	let mut _ignore_open_errors = false;
	let mut _ignore_read_errors = false;
	let mut _report_errors_to_sink = true;
	let mut _report_errors_to_stderr = true;
	let mut _io_fadvise = false;
	
	{
		let mut _parser = argparse::ArgumentParser::new ();
		_hashes_flags.argparse (&mut _parser);
		_format_flags.argparse (&mut _parser);
		_parser.refer (&mut _relative)
				.add_option (&["--relative"], argparse::StoreTrue, "output paths relative to source (true by default)")
				.add_option (&["--no-relative"], argparse::StoreFalse, "do not output paths relative to source");
		_parser.refer (&mut _walk_xdev) .add_option (&["-x", "--xdev"], argparse::StoreTrue, "do not cross mount points");
		_parser.refer (&mut _walk_follow) .add_option (&["-L", "--follow"], argparse::StoreTrue, "follow symlinks (n.b. arguments are followed)");
		_parser.refer (&mut _threads_count) .add_option (&["-w", "--workers-count"], argparse::Parse, "hashing workers count (16 by default)");
		_parser.refer (&mut _queue_size) .add_option (&["--workers-queue"], argparse::Parse, "hashing workers queue size (1024 times workers count by default)");
		_parser.refer (&mut _batch_size) .add_option (&["--workers-batch"], argparse::Parse, "hashing workers batch size (16 times workers queue size by default)");
		_parser.refer (&mut _nice_level) .add_option (&["--nice"], argparse::Parse, "set OS process scheduling priority (i.e. `nice`) (19 by default)");
		_parser.refer (&mut _io_fadvise) .add_option (&["--fadvise"], argparse::StoreTrue, "use OS `fadvise` with sequential and no-reuse (false by default)");
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
		_parser.refer (&mut _output_path) .add_option (&["-o", "--output"], argparse::Parse, "output file (use `-` for `stdout`, and `.` for auto-detection) (`.` by default)");
		_parser.refer (&mut _source_path) .add_argument ("source", argparse::Parse, "source file or folder") .required ();
		_parser.parse_args_or_exit ();
	}
	
	
	if _output_path == path::Path::new ("") {
		_output_path = path::PathBuf::from (".");
	}
	if _source_path == path::Path::new ("") {
		_source_path = path::PathBuf::from (".");
	}
	
	if _threads_count == 0 {
		_threads_count = 16;
	}
	if _queue_size == 0 {
		_queue_size = _threads_count * 1024;
	}
	if _batch_size == 0 {
		_batch_size = _queue_size * 16;
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
	
	
	
	
	let _relative_path = match fs::metadata (&_source_path) {
		Ok (ref _stat) if _stat.is_dir () =>
			if _relative {
				Some (_source_path.clone ())
			} else {
				None
			},
		Ok (ref _stat) if _stat.is_file () =>
			if _relative {
				if let Some (_relative_path) = _source_path.parent () {
					Some (_relative_path.into ())
				} else {
					None
				}
			} else {
				None
			},
		Ok (_) =>
			return Err (io::Error::new (io::ErrorKind::Other, "[a12f1634]  invalid source path (non file or folder)")),
		Err (ref _error) if _error.kind () == io::ErrorKind::NotFound =>
			return Err (io::Error::new (io::ErrorKind::Other, "[9ee46264]  invalid source path (non exists)")),
		Err (_error) =>
			return Err (_error),
	};
	
	
	
	
	let _output_path = if _output_path == path::Path::new ("-") {
		None
		
	} else {
		
		let _output_path_with_transformer = if _output_path != path::Path::new (".") {
			match fs::metadata (&_output_path) {
				Ok (ref _stat) if _stat.is_dir () =>
					Some ((_output_path, Some (true))),
				Ok (ref _stat) if _stat.is_file () =>
					return Err (io::Error::new (io::ErrorKind::Other, "[b4ab81b9]  invalid output path (already exists)")),
				Ok (_) =>
					return Err (io::Error::new (io::ErrorKind::Other, "[8366e424]  invalid output path (non file or folder)")),
				Err (ref _error) if _error.kind () == io::ErrorKind::NotFound =>
					Some ((_output_path, None)),
				Err (_error) =>
					return Err (_error),
			}
			
		} else {
			match fs::metadata (&_source_path) {
				Ok (ref _stat) if _stat.is_dir () => {
					let mut _outcome = None;
					for _suffix in &[_hashes_flags.algorithm.suffix, ".hashes", ".md5"] {
						let _output_path_base = _source_path.join (_suffix);
						match fs::metadata (&_output_path_base) {
							Ok (ref _stat) if _stat.is_dir () => {
								_outcome = Some (Some ((_output_path_base, Some (true))));
								break;
							},
							Ok (ref _stat) if _stat.is_file () => {
								_outcome = Some (Some ((_output_path_base, Some (false))));
								break;
							},
							Ok (_) =>
								return Err (io::Error::new (io::ErrorKind::Other, "[2cb4982d]  invalid hashes path (non file or folder)")),
							Err (ref _error) if _error.kind () == io::ErrorKind::NotFound =>
								(),
							Err (_error) =>
								return Err (_error),
						}
					}
					if let Some (_outcome) = _outcome {
						_outcome
					} else {
						let mut _output_path = ffi::OsString::from (&_source_path);
						_output_path.push (path::MAIN_SEPARATOR.to_string ());
						_output_path.push (".");
						Some ((_output_path.into (), Some (false)))
					}
				},
				Ok (ref _stat) if _stat.is_file () =>
					Some ((_source_path.clone (), Some (false))),
				Ok (_) =>
					return Err (io::Error::new (io::ErrorKind::Other, "[cce14438]  invalid source path (non file or folder)")),
				Err (ref _error) if _error.kind () == io::ErrorKind::NotFound =>
					return Err (io::Error::new (io::ErrorKind::Other, "[5f86a63d]  invalid source path (non exists)")),
				Err (_error) =>
					return Err (_error),
			}
		};
		
		match _output_path_with_transformer {
			None =>
				None,
			Some ((_output_path, None)) =>
				Some (_output_path),
			Some ((_output_path_base, Some (_transformer))) => {
				
				let _output_path_suffix = _hashes_flags.algorithm.suffix;
				
				let _output_timestamp = {
					
					use chrono::Datelike as _;
					use chrono::Timelike as _;
					let _output_timestamp = chrono::Local::now ();
					let _output_timestamp_date = _output_timestamp.date ();
					let _output_timestamp_time = _output_timestamp.time ();
					
					format! (
							"{:04}-{:02}-{:02}-{:02}-{:02}-{:02}",
							_output_timestamp_date.year (),
							_output_timestamp_date.month (),
							_output_timestamp_date.day (),
							_output_timestamp_time.hour (),
							_output_timestamp_time.minute (),
							_output_timestamp_time.second (),
						)
				};
				
				if _transformer {
					let _output_path = _output_path_base.join (_output_timestamp + _output_path_suffix);
					Some (_output_path)
				} else {
					let mut _output_path = ffi::OsString::from (_output_path_base);
					_output_path.push ("--");
					_output_path.push (_output_timestamp);
					_output_path.push (_output_path_suffix);
					Some (_output_path.into ())
				}
			}
		}
	};
	
	let _output_path_and_tmp = if let Some (_output_path) = _output_path {
		let mut _output_path_tmp = ffi::OsString::from (&_output_path);
		_output_path_tmp.push (".tmp");
		let _output_path_tmp = path::PathBuf::from (_output_path_tmp);
		Some ((_output_path, _output_path_tmp))
	} else {
		None
	};
	
	
	
	
	if let Some ((ref _output_path, _)) = _output_path_and_tmp {
		eprintln! ("[ii] [8cc8542c]  creating `{}`...", _output_path.to_string_lossy ());
	}
	let (_output_file, _output_stat) = if let Some ((_, ref _output_path_tmp)) = _output_path_and_tmp {
		let mut _output_file = fs::OpenOptions::new () .create_new (true) .write (true) .open (_output_path_tmp) ?;
		_output_file.set_permissions (fs::Permissions::from_mode (0o600)) ?;
		let _output_stat = _output_file.metadata () ?;
		(_output_file, Some (_output_stat))
	} else {
		let _output_file = fs::OpenOptions::new () .write (true) .open ("/dev/stdout") ?;
		(_output_file, None)
	};
	
	
	let _sink = StandardHashesSink::new (_output_file, _format_flags.zero);
	let _sink = sync::Arc::new (sync::Mutex::new (_sink));
	
	
	let (_enqueue, _dequeue) = crossbeam::channel::bounded::<walkdir::DirEntry> (_queue_size);
	let mut _completions = Vec::with_capacity (_threads_count);
	let _threads_errors = sync::Arc::new (sync::Mutex::new (Vec::new ()));
	let _done = crossbeam::sync::WaitGroup::new ();
	
	
	for _ in 0 .. _threads_count {
		
		let _relative_path = _relative_path.clone ();
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
					let _path_for_sink = if let Some (ref _relative_path) = _relative_path {
						_path.strip_prefix (_relative_path) .unwrap () .as_os_str ()
					} else {
						_path.as_os_str ()
					};
					let _path_for_sink = if _path_for_sink != "" { _path_for_sink } else { ffi::OsStr::new (".") };
					
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
								_sink.handle (_path_for_sink, _hashes_algorithm.invalid_raw) ?;
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
					
					if _io_fadvise {
						let mut _failed = false;
						unsafe {
							if libc::posix_fadvise (_file.as_raw_fd (), 0, 0, libc::POSIX_FADV_SEQUENTIAL) != 0 {
								_failed = true;
							}
							if libc::posix_fadvise (_file.as_raw_fd (), 0, 0, libc::POSIX_FADV_NOREUSE) != 0 {
								_failed = true;
							}
							if libc::posix_fadvise (_file.as_raw_fd (), 0, 0, libc::POSIX_FADV_WILLNEED) != 0 {
								_failed = true;
							}
						}
						if _failed {
							eprintln! ("[ww] [76280772]  `fadvise` failed!")
						}
					}
					
					_hash_buffer.clear ();
					match digest (_hashes_algorithm, &mut _file, &mut _hash_buffer) {
						Ok (()) => {
							let mut _sink = _sink.lock () .unwrap ();
							_sink.handle (_path_for_sink, &_hash_buffer) ?;
						},
						Err (_error) => {
							let mut _sink = _sink.lock () .unwrap ();
							if _report_errors_to_stderr {
								eprintln! ("[ee] [1aeb2750]  failed reading file `{}`: `{}`!", _path.to_string_lossy (), _error);
							}
							if _report_errors_to_sink {
								_sink.handle (_path_for_sink, _hashes_algorithm.invalid_raw) ?;
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
					
					if _io_fadvise {
						let mut _failed = false;
						unsafe {
							if libc::posix_fadvise (_file.as_raw_fd (), 0, 0, libc::POSIX_FADV_DONTNEED) != 0 {
								_failed = true;
							}
						}
						if _failed {
							eprintln! ("[ww] [def753c5]  `fadvise` failed!")
						}
					}
				}
				
				drop (_sink);
				drop (_done);
				
				return Ok (());
			});
		
		_completions.push (_completion);
	}
	
	
	let mut _walker = walkdir::WalkDir::new (&_source_path)
			.same_file_system (_walk_xdev)
			.follow_links (_walk_follow)
			.contents_first (true)
			.into_iter ();
	
	let mut _batch = if _batch_size > 1 {
		Some (Vec::<(walkdir::DirEntry, (u64, u64))>::with_capacity (_batch_size))
	} else {
		None
	};
	
	
	let mut _errors = Vec::<io::Error>::new ();
	let _unknown_error = io::Error::new (io::ErrorKind::Other, "[31b7b284]  unexpected error");
	
	
	loop {
		
		if let Some (ref mut _batch) = _batch {
			if _batch.capacity () == _batch.len () {
				_batch.sort_by_key (|&(_, _order)| _order);
				for (_entry, _) in _batch.drain (..) {
					_enqueue.send (_entry) .unwrap ();
				}
			}
		}
		
		let _entry = match _walker.next () {
			Some (Ok (_entry)) =>
				_entry,
			Some (Err (_error)) => {
				let mut _sink = _sink.lock () .unwrap ();
				let _path = _error.path () .unwrap_or (&_source_path);
				if let Some (_ancestor) = _error.loop_ancestor () {
					eprintln! ("[ww] [55021f5c]  detected walking loop for `{}` pointing at `{}`;  ignoring!", _path.to_string_lossy (), _ancestor.to_string_lossy ());
					continue;
				}
				if _report_errors_to_stderr {
					eprintln! ("[ee] [a5e88e25]  failed walking path `{}`: `{}`!", _path.to_string_lossy (), _error.io_error () .unwrap_or (&_unknown_error));
				}
				if _report_errors_to_sink {
					let _path_for_sink = if let Some (ref _relative_path) = _relative_path {
						_path.strip_prefix (_relative_path) .unwrap () .as_os_str ()
					} else {
						_path.as_os_str ()
					};
					let _path_for_sink = if _path_for_sink != "" { _path_for_sink } else { ffi::OsStr::new (".") };
					_sink.handle (_path_for_sink, _hashes_flags.algorithm.invalid_raw) ?;
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
				let _path = _error.path () .unwrap_or (&_source_path);
				if _report_errors_to_stderr {
					eprintln! ("[ee] [96d2838a]  failed walking path `{}`: `{}`!", _entry.path () .to_string_lossy (), _error.io_error () .unwrap_or (&_unknown_error));
				}
				if _report_errors_to_sink {
					let _path = _entry.path ();
					let _path_for_sink = if let Some (ref _relative_path) = _relative_path {
						_path.strip_prefix (_relative_path) .unwrap () .as_os_str ()
					} else {
						_path.as_os_str ()
					};
					let _path_for_sink = if _path_for_sink != "" { _path_for_sink } else { ffi::OsStr::new (".") };
					_sink.handle (_path_for_sink, _hashes_flags.algorithm.invalid_raw) ?;
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
		
		if let Some (ref _output_stat) = _output_stat {
			if (_metadata.dev () == _output_stat.dev ()) && (_metadata.ino () == _output_stat.ino ()) {
				continue;
			}
		}
		
		if _metadata.is_file () {
			if let Some (ref mut _batch) = _batch {
				let _order = (_metadata.dev (), _metadata.ino ());
				_batch.push ((_entry, _order));
			} else {
				_enqueue.send (_entry) .unwrap ();
			}
		}
	}
	
	if let Some (ref mut _batch) = _batch {
		for (_entry, _) in _batch.drain (..) {
			_enqueue.send (_entry) .unwrap ();
		}
	}
	
	drop (_enqueue);
	drop (_dequeue);
	
	
	_done.wait ();
	
	
	let _sink = sync::Arc::try_unwrap (_sink) .ok () .expect ("[3d3636b0]");
	let _sink = _sink.into_inner () .expect ("[1a198ea3]");
	let mut _output_file = _sink.done () ?;
	
	if let Some ((ref _output_path, ref _output_path_tmp)) = _output_path_and_tmp {
		_output_file.set_permissions (fs::Permissions::from_mode (0o400)) ?;
		_output_file.sync_all () ?;
		fs::rename (_output_path_tmp, _output_path) ?;
	}
	drop (_output_file);
	
	
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

