

use ::atty;
use ::argparse;
use ::chrono;
use ::crossbeam;
use ::indicatif;
use ::libc;
use ::walkdir;

use crate::digests::*;
use crate::flags::*;
use crate::prelude::*;
use crate::sinks::*;




#[ derive (Clone) ]
struct HasherContext <Sink : HashesSink>  {
	
	flags : CreateFlags,
	queue : crossbeam::channel::Receiver<HasherTask>,
	sink : sync::Arc<sync::Mutex<Sink>>,
	errors : sync::Arc<sync::Mutex<Vec<io::Error>>>,
	progress : Option<Progress>,
	done : crossbeam::sync::WaitGroup,
}


#[ derive (Clone) ]
struct HasherTask {
	
	path : path::PathBuf,
	metadata : fs::Metadata,
}


#[ derive (Clone) ]
struct Progress {
	
	folder : indicatif::ProgressBar,
	files : indicatif::ProgressBar,
	data : indicatif::ProgressBar,
}




pub fn main () -> (Result<(), io::Error>) {
	
	
	let mut _flags = CreateFlags::default ();
	
	{
		let mut _parser = argparse::ArgumentParser::new ();
		_flags.argparse (&mut _parser);
		_parser.parse_args_or_exit ();
	}
	
	
	if _flags.output_path == path::Path::new ("") {
		_flags.output_path = path::PathBuf::from (".");
	}
	if _flags.source_path == path::Path::new ("") {
		_flags.source_path = path::PathBuf::from (".");
	}
	
	if _flags.threads_count == 0 {
		_flags.threads_count = 16;
	}
	if _flags.queue_size == 0 {
		_flags.queue_size = _flags.threads_count * 1024 * 4;
	}
	if _flags.batch_size == 0 {
		_flags.batch_size = _flags.queue_size / 2;
	}
	if _flags.ignore_all_errors {
		_flags.ignore_walk_errors = true;
		_flags.ignore_open_errors = true;
		_flags.ignore_read_errors = true;
	}
	
	
	if _flags.threads_nice != 0 {
		unsafe {
			// FIXME:  Check the return value!
			libc::nice (_flags.threads_nice as i32);
		}
	}
	
	
	
	
	let _source_path = _flags.source_path.clone ();
	
	let _relative_path = match fs::metadata (&_source_path) {
		Ok (ref _stat) if _stat.is_dir () =>
			if _flags.relative {
				Some (_source_path.clone ())
			} else {
				None
			},
		Ok (ref _stat) if _stat.is_file () =>
			if _flags.relative {
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
	
	
	
	
	let mut _output_path = _flags.output_path.clone ();
	
	let _output_descriptor = if
			if _output_path == path::Path::new ("-") {
				_output_path = path::PathBuf::from ("/dev/stdout");
				true
			} else if _output_path == path::Path::new ("/dev/stdout") {
				true
			} else if _output_path == path::Path::new ("/dev/stderr") {
				true
			} else if _output_path == path::Path::new ("/dev/null") {
				true
			} else if
					_output_path.starts_with (path::Path::new ("/dev/fd")) ||
					_output_path.starts_with (path::Path::new ("/proc/self/fd")) {
				true
			} else if
					_output_path.starts_with (path::Path::new ("/dev")) ||
					_output_path.starts_with (path::Path::new ("/proc")) ||
					_output_path.starts_with (path::Path::new ("/sys")) {
				return Err (io::Error::new (io::ErrorKind::Other, "[49b2e473]  invalid output path"));
			} else {
				false
			}
	{
		None
		
	} else {
		
		let _output_path_with_transformer = if _output_path != path::Path::new (".") {
			match fs::metadata (&_output_path) {
				Ok (ref _stat) if _stat.is_dir () =>
					Some ((_output_path.clone (), Some (true))),
				Ok (ref _stat) if _stat.is_file () =>
					return Err (io::Error::new (io::ErrorKind::Other, "[b4ab81b9]  invalid output path (already exists)")),
				Ok (_) =>
					return Err (io::Error::new (io::ErrorKind::Other, "[8366e424]  invalid output path (non file or folder)")),
				Err (ref _error) if _error.kind () == io::ErrorKind::NotFound =>
					Some ((_output_path.clone (), None)),
				Err (_error) =>
					return Err (_error),
			}
			
		} else {
			match fs::metadata (&_source_path) {
				Ok (ref _stat) if _stat.is_dir () => {
					let mut _outcome = None;
					for _suffix in &[_flags.hashes_flags.algorithm.suffix, ".hashes", ".md5"] {
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
				
				let _output_path_suffix = _flags.hashes_flags.algorithm.suffix;
				
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
	
	let _output_path_and_tmp = if let Some (_output_path) = _output_descriptor {
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
		if _output_path == path::Path::new ("/dev/stdout") && atty::is (atty::Stream::Stdout) {
			_flags.report_progress = false;
		}
		let _output_file = fs::OpenOptions::new () .write (true) .open (_output_path) ?;
		(_output_file, None)
	};
	
	
	let _sink = StandardHashesSink::new (_output_file, _flags.format_flags.zero);
	let _sink = sync::Arc::new (sync::Mutex::new (_sink));
	
	
	let (_enqueue, _dequeue) = crossbeam::channel::bounded::<HasherTask> (_flags.queue_size);
	let mut _completions = Vec::with_capacity (_flags.threads_count);
	let _threads_errors = sync::Arc::new (sync::Mutex::new (Vec::new ()));
	let _done = crossbeam::sync::WaitGroup::new ();
	
	
	
	
	if ! atty::is (atty::Stream::Stderr) {
		_flags.report_progress = false;
	}
	
	let _progress = if _flags.report_progress {
		
		let _folder = indicatif::ProgressBar::new (!0);
		_folder.set_style (
				indicatif::ProgressStyle::default_bar ()
					.template ("[..] [{elapsed:>8}]  | {wide_msg} |")
					.progress_chars ("=>-")
					.tick_chars (".|/-\\")
			);
		
		let _files = indicatif::ProgressBar::new (0);
		_files.set_style (
				indicatif::ProgressStyle::default_bar ()
					.template ("[..] [{elapsed_precise}]  | {wide_bar} | {percent:>3}% | {per_sec:>10} | {pos:>10} | {len:>10} |")
					.progress_chars ("=>-")
					.tick_chars (".|/-\\")
			);
		_files.set_draw_delta (10);
		
		let _data = indicatif::ProgressBar::new (0);
		_data.set_style (
				indicatif::ProgressStyle::default_bar ()
					.template ("[..] [{eta_precise}]  | {wide_bar} | {percent:>3}% | {bytes_per_sec:>10} | {bytes:>10} | {total_bytes:>10} |")
					.progress_chars ("=>-")
					.tick_chars (".|/-\\")
			);
		_data.set_draw_delta (128 * 1024);
		
		{
			let _dashboard = indicatif::MultiProgress::new ();
			_dashboard.set_draw_target (indicatif::ProgressDrawTarget::stderr_with_hz (4));
			_dashboard.add (_folder.clone ());
			_dashboard.add (_files.clone ());
			_dashboard.add (_data.clone ());
			thread::spawn (move || -> () {
					_dashboard.join () .unwrap ();
				});
		}
		
		Some (Progress {
				folder : _folder,
				files : _files,
				data : _data,
			})
		
	} else {
		None
	};
	
	macro_rules! message {
		( $progress : expr, $( $token : tt )+ ) => (
			if let Some (ref _progress) = $progress {
				_progress.files.println (format! ( $( $token )+ ));
			} else {
				eprintln! ( $( $token )+ );
			}
		)
	}
	
	
	
	
	for _ in 0 .. _flags.threads_count {
		
		let _context = HasherContext {
				flags : _flags.clone (),
				queue : _dequeue.clone (),
				sink : sync::Arc::clone (&_sink),
				errors : sync::Arc::clone (&_threads_errors),
				progress : _progress.clone (),
				done : _done.clone (),
			};
		
		let _relative_path = _relative_path.clone ();
		
		let _completion = thread::spawn (move || execute_hasher (_context, _relative_path));
		
		_completions.push (_completion);
	}
	
	
	
	
	let mut _walker = walkdir::WalkDir::new (&_source_path)
			.same_file_system (_flags.walk_xdev)
			.follow_links (_flags.walk_follow)
			.contents_first (true)
			.into_iter ();
	
	let mut _walk_index = 0 as u64;
	
	
	let mut _batch = if _flags.batch_size > 1 {
		Some (Vec::<(walkdir::DirEntry, fs::Metadata, DirEntryOrder)>::with_capacity (_flags.batch_size))
	} else {
		None
	};
	
	
	let mut _errors = Vec::<io::Error>::new ();
	let _unknown_error = io::Error::new (io::ErrorKind::Other, "[31b7b284]  unexpected error");
	
	
	loop {
		
		_walk_index += 1;
		
		if let Some (ref mut _batch) = _batch {
			if _batch.capacity () == _batch.len () {
				if let Some (ref _progress) = _progress {
					_progress.folder.set_message ("(enqueueing...)");
					_progress.folder.tick ();
				}
				_batch.sort_by_key (|&(_, _, _order)| _order);
				for (_entry, _metadata, _) in _batch.drain (..) {
					let _task = HasherTask {
							path : _entry.into_path (),
							metadata : _metadata,
						};
					_enqueue.send (_task) .unwrap ();
				}
				if let Some (ref _progress) = _progress {
					_progress.folder.set_message ("(walking...)");
					_progress.folder.tick ();
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
					message! (_progress, "[ww] [55021f5c]  detected walking loop for `{}` pointing at `{}`;  ignoring!", _path.to_string_lossy (), _ancestor.to_string_lossy ());
					continue;
				}
				if _flags.report_errors_to_stderr {
					message! (_progress, "[ee] [a5e88e25]  failed walking path `{}`: `{}`!", _path.to_string_lossy (), _error.io_error () .unwrap_or (&_unknown_error));
				}
				if _flags.report_errors_to_sink {
					let _path_for_sink = if let Some (ref _relative_path) = _relative_path {
						_path.strip_prefix (_relative_path) .unwrap () .as_os_str ()
					} else {
						_path.as_os_str ()
					};
					let _path_for_sink = if _path_for_sink != "" { _path_for_sink } else { ffi::OsStr::new (".") };
					_sink.handle (_path_for_sink, _flags.hashes_flags.algorithm.invalid_raw) ?;
					_sink.flush () ?;
				}
				if _flags.ignore_walk_errors {
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
				if _flags.report_errors_to_stderr {
					message! (_progress, "[ee] [96d2838a]  failed walking path `{}`: `{}`!", _entry.path () .to_string_lossy (), _error.io_error () .unwrap_or (&_unknown_error));
				}
				if _flags.report_errors_to_sink {
					let _path = _entry.path ();
					let _path_for_sink = if let Some (ref _relative_path) = _relative_path {
						_path.strip_prefix (_relative_path) .unwrap () .as_os_str ()
					} else {
						_path.as_os_str ()
					};
					let _path_for_sink = if _path_for_sink != "" { _path_for_sink } else { ffi::OsStr::new (".") };
					_sink.handle (_path_for_sink, _flags.hashes_flags.algorithm.invalid_raw) ?;
					_sink.flush () ?;
				}
				if _flags.ignore_walk_errors {
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
		
		if _metadata.is_dir () {
			if let Some (ref _progress) = _progress {
				_progress.folder.set_message (& _entry.path () .to_string_lossy ());
				_progress.folder.tick ();
			}
		}
		
		if _metadata.is_file () {
			
			if let Some (ref _progress) = _progress {
				_progress.files.inc_length (1);
				_progress.files.tick ();
				_progress.data.inc_length (_metadata.size ());
				_progress.data.tick ();
			}
			
			if let Some (ref mut _batch) = _batch {
				let _order = entry_order (&_entry, &_metadata, _walk_index, _flags.batch_order);
				_batch.push ((_entry, _metadata, _order));
			} else {
				let _task = HasherTask {
						path : _entry.into_path (),
						metadata : _metadata,
					};
				_enqueue.send (_task) .unwrap ();
			}
		}
	}
	
	if let Some (ref mut _batch) = _batch {
		if let Some (ref _progress) = _progress {
			_progress.folder.set_message ("(enqueueing...)");
			_progress.folder.tick ();
		}
		_batch.sort_by_key (|&(_, _, _order)| _order);
		for (_entry, _metadata, _) in _batch.drain (..) {
			let _task = HasherTask {
					path : _entry.into_path (),
					metadata : _metadata,
				};
			_enqueue.send (_task) .unwrap ();
		}
	}
	
	if let Some (ref _progress) = _progress {
		_progress.folder.set_message ("(waiting...)");
		_progress.folder.tick ();
	}
	
	drop (_enqueue);
	drop (_dequeue);
	
	
	_done.wait ();
	
	
	if let Some (ref _progress) = _progress {
		_progress.folder.set_message ("(completed!)");
		_progress.folder.tick ();
		_progress.folder.finish ();
		_progress.files.finish ();
		_progress.data.finish ();
	}
	
	
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




fn execute_hasher <Sink : HashesSink> (_context : HasherContext<Sink>, _relative_path : Option<path::PathBuf>) -> (Result<(), io::Error>) {
	
	macro_rules! message {
		( $( $token : tt )+ ) => (
			if let Some (ref _progress) = _context.progress {
				_progress.files.println (format! ( $( $token )+ ));
			} else {
				eprintln! ( $( $token )+ );
			}
		)
	}
	
	let mut _hash_buffer = Vec::with_capacity (128);
	
	loop {
		
		let _task = match _context.queue.recv () {
			Ok (_task) =>
				_task,
			Err (crossbeam::channel::RecvError) =>
				break,
		};
		
		let _path = &_task.path;
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
				let mut _sink = _context.sink.lock () .unwrap ();
				if _context.flags.report_errors_to_stderr {
					message! ("[ee] [42f1352f]  failed opening file `{}`: `{}`!", _path.to_string_lossy (), _error);
				}
				if _context.flags.report_errors_to_sink {
					_sink.handle (_path_for_sink, _context.flags.hashes_flags.algorithm.invalid_raw) ?;
					_sink.flush () ?;
				}
				_context.errors.lock () .unwrap () .push (_error);
				if _context.flags.ignore_open_errors {
					continue;
				} else {
					return Ok (());
				}
			},
		};
		
		if _context.flags.read_fadvise {
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
				message! ("[ww] [76280772]  `fadvise` failed!")
			}
		}
		
		_hash_buffer.clear ();
		match digest (_context.flags.hashes_flags.algorithm, &mut _file, &mut _hash_buffer) {
			Ok (()) => {
				let mut _sink = _context.sink.lock () .unwrap ();
				_sink.handle (_path_for_sink, &_hash_buffer) ?;
			},
			Err (_error) => {
				let mut _sink = _context.sink.lock () .unwrap ();
				if _context.flags.report_errors_to_stderr {
					message! ("[ee] [1aeb2750]  failed reading file `{}`: `{}`!", _path.to_string_lossy (), _error);
				}
				if _context.flags.report_errors_to_sink {
					_sink.handle (_path_for_sink, _context.flags.hashes_flags.algorithm.invalid_raw) ?;
					_sink.flush () ?;
				}
				_context.errors.lock () .unwrap () .push (_error);
				if _context.flags.ignore_read_errors {
					continue;
				} else {
					return Ok (());
				}
			},
		}
		
		if _context.flags.read_fadvise {
			let mut _failed = false;
			unsafe {
				if libc::posix_fadvise (_file.as_raw_fd (), 0, 0, libc::POSIX_FADV_DONTNEED) != 0 {
					_failed = true;
				}
			}
			if _failed {
				message! ("[ww] [def753c5]  `fadvise` failed!")
			}
		}
		
		if let Some (ref _progress) = _context.progress {
			_progress.files.inc (1);
			_progress.data.inc (_task.metadata.size ());
		}
	}
	
	return Ok (());
}




#[ derive (Copy, Clone, Eq, Ord, PartialEq, PartialOrd) ]
struct DirEntryOrder (u64, u64, u64);


fn entry_order (_entry : & walkdir::DirEntry, _metadata : & fs::Metadata, _index : u64, _kind : CreateBatchOrder) -> (DirEntryOrder) {
	match _kind {
		CreateBatchOrder::Index =>
			DirEntryOrder (_index, 0, 0),
		CreateBatchOrder::Inode =>
			DirEntryOrder (_metadata.ino (), 0, 0),
		CreateBatchOrder::InodeAndSizeBuckets =>
			return entry_order_by_inode (_entry, _metadata, _index),
		CreateBatchOrder::Extent =>
			return entry_order_by_extent (_entry, _metadata, _index),
		CreateBatchOrder::Random =>
			return entry_order_by_hash (_entry, _metadata, _index),
	}
}


fn entry_order_by_inode (_entry : & walkdir::DirEntry, _metadata : & fs::Metadata, _index : u64) -> (DirEntryOrder) {
	
	let _dev = _metadata.dev ();
	let _inode = _metadata.ino ();
	let _blocks = _metadata.blocks () * 512 / _metadata.blksize ();
	
	// NOTE:  First group files based on inode (regardless of device).
	let _order_1 = _inode / (1024 * 128);
	
	// NOTE:  Then group files based on log2 actual used file-system blocks.
	let _order_2 = (64 - _blocks.leading_zeros ()) as u64;
	
	// NOTE:  Then order files by inode and then based on device.
	//   (This doesn't perfectly distributes files from different devices, but we try...)
	let _order_3 = (_inode % (1024 * 128) << 32) | ((_dev >> 32) ^ (_dev & 0xffffffff));
	
	DirEntryOrder (_order_1, _order_2, _order_3)
}


fn entry_order_by_hash (_entry : & walkdir::DirEntry, _metadata : & fs::Metadata, _index : u64) -> (DirEntryOrder) {
	#[ allow (deprecated) ]
	let mut _hasher = hash::SipHasher::new ();
	_hasher.write_u64 (_metadata.dev ());
	_hasher.write_u64 (_metadata.ino ());
	_hasher.write_u64 (_metadata.size ());
	let _order = _hasher.finish ();
	DirEntryOrder (_order, 0, 0)
}




#[ allow (dead_code) ]
fn entry_order_by_extent (_entry : & walkdir::DirEntry, _metadata : & fs::Metadata, _index : u64) -> (DirEntryOrder) {
	
	
	// NOTE:  See also:  https://www.kernel.org/doc/Documentation/filesystems/fiemap.txt
	// NOTE:  Inspired by: https://github.com/lilydjwg/fiemap-rs/blob/master/fiemap/src/lib.rs
	
	
	#[ repr (C) ]
	#[ derive (Default) ]
	struct fiemap {
		fm_start: u64,
		fm_length: u64,
		fm_flags: u32,
		fm_mapped_extents: u32,
		fm_extent_count: u32,
		fm_reserved: u32,
		fm_extents: [fiemap_extent; 1],
	}
	
	#[repr (C) ]
	#[ derive (Default) ]
	struct fiemap_extent {
		fe_logical: u64,
		fe_physical: u64,
		fe_length: u64,
		fe_reserved64: [u64; 2],
		fe_flags: u32,
		fe_reserved: [u32; 3],
	}
	
	const FS_IOC_FIEMAP : libc::c_ulong = 0xC020660B;
	
	const FIEMAP_FLAG_SYNC  : u32 = 0x00000001;
	const FIEMAP_FLAG_XATTR : u32 = 0x00000002;
	const FIEMAP_FLAG_CACHE : u32 = 0x00000004;
	
	const FIEMAP_EXTENT_LAST           : u32 = 0x00000001;
	const FIEMAP_EXTENT_UNKNOWN        : u32 = 0x00000002;
	const FIEMAP_EXTENT_DELALLOC       : u32 = 0x00000004;
	const FIEMAP_EXTENT_ENCODED        : u32 = 0x00000008;
	const FIEMAP_EXTENT_DATA_ENCRYPTED : u32 = 0x00000080;
	const FIEMAP_EXTENT_NOT_ALIGNED    : u32 = 0x00000100;
	const FIEMAP_EXTENT_DATA_INLINE    : u32 = 0x00000200;
	const FIEMAP_EXTENT_DATA_TAIL      : u32 = 0x00000400;
	const FIEMAP_EXTENT_UNWRITTEN      : u32 = 0x00000800;
	const FIEMAP_EXTENT_MERGED         : u32 = 0x00001000;
	const FIEMAP_EXTENT_SHARED         : u32 = 0x00002000;
	
	
	let mut _fiemap : fiemap = Default::default ();
	_fiemap.fm_length = 1;
	_fiemap.fm_extent_count = 1;
	
	let _path = ffi::CString::new (_entry.path () .as_os_str () .as_bytes ()) .unwrap ();
	
	let _succeeded = unsafe {
		let mut _succeeded = true;
		let _file = libc::open (_path.as_ptr (), libc::O_RDONLY | libc::O_NOFOLLOW);
		if _file < 0 {
			_succeeded = false;
		}
		if _succeeded {
			_succeeded = libc::ioctl (_file, FS_IOC_FIEMAP, &mut _fiemap as *mut _) == 0;
		}
		if _file >= 0 {
			_succeeded = libc::close (_file) == 0;
		}
		_succeeded
	};
	
	if !_succeeded {
		DirEntryOrder (0, _metadata.ino (), 0)
	} else if _fiemap.fm_mapped_extents == 1 {
		if (_fiemap.fm_extents[0].fe_flags & FIEMAP_EXTENT_UNKNOWN) == 0 {
			let _block = _fiemap.fm_extents[0].fe_physical;
			DirEntryOrder (3 + _block, 0, 0)
		} else {
			DirEntryOrder (2, _metadata.ino (), 0)
		}
	} else {
		DirEntryOrder (1, _metadata.ino (), 0)
	}
}

