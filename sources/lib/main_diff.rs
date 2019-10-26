

use ::argparse;
use ::regex;

use crate::core::*;
use crate::flags::*;
use crate::hashes::*;
use crate::prelude::*;

#[ cfg (feature = "profile") ]
use ::cpuprofiler::PROFILER as profiler;




struct Source {
	path : path::PathBuf,
	records : Vec<SourceRecord>,
}

struct SourceRecord {
	hash : HashKey,
	path : PathKey,
	#[ allow (dead_code) ]
	line : usize,
}

struct SourceIndex <'a> {
	by_hash : HashMap<HashKey, Vec<&'a SourceRecord>>,
	by_path : HashMap<PathKey, Vec<&'a SourceRecord>>,
}

struct SourceStatistics {
	records : usize,
	distinct_hashes : usize,
	unique_hashes : usize,
	duplicate_hashes : usize,
	unique_files : usize,
	duplicate_files : usize,
	empty_files : usize,
	invalid_files : usize,
	distinct_paths : usize,
	unique_paths : usize,
	duplicate_paths : usize,
}


struct Diff {
	hashes : Vec<HashKey>,
	paths : Vec<PathKey>,
	by_hash : HashMap<HashKey, DiffEntry<PathKey>>,
	by_path : HashMap<PathKey, DiffEntry<HashKey>>,
	by_hash_statistics : DiffStatistics,
	by_path_statistics : DiffStatistics,
}

enum DiffEntry<K> {
	UniqueLeft (Vec<K>),
	UniqueRight (Vec<K>),
	Matching (Vec<K>, Vec<K>),
	Conflicting (Vec<K>, Vec<K>),
}

struct DiffStatistics {
	distinct : usize,
	matching : usize,
	conflicting : usize,
	unique_left : usize,
	unique_right : usize,
}


struct Tokens {
	hashes : Vec<Rc<HashValue>>,
	hashes_index : HashMap<Rc<HashValue>, HashKey>,
	hashes_order : Vec<usize>,
	paths : Vec<Rc<PathValue>>,
	paths_index : HashMap<Rc<PathValue>, PathKey>,
	paths_order : Vec<usize>,
	hash_key_empty : HashKey,
	hash_key_invalid : HashKey,
}

type HashKey = usize;
type PathKey = usize;
type TokenOrder = usize;




pub fn main () -> (Result<(), io::Error>) {
	
	
	let mut _hashes_flags = HashesFlags {
			algorithm : &MD5,
		};
	
	let mut _format_flags = HashesFormatFlags {
			zero : false,
		};
	
	let mut _compression_flags = CompressionFlags {
			algorithm : CompressionAlgorithm::None,
		};
	
	let mut _path_left = path::PathBuf::from ("");
	let mut _path_right = path::PathBuf::from ("");
	
	{
		let mut _parser = argparse::ArgumentParser::new ();
		_hashes_flags.argparse (&mut _parser);
		_format_flags.argparse (&mut _parser);
		_compression_flags.argparse (&mut _parser);
		_parser.refer (&mut _path_left) .add_argument ("dataset-a", argparse::Parse, "source file for dataset A") .required ();
		_parser.refer (&mut _path_right) .add_argument ("dataset-b", argparse::Parse, "source file for dataset B") .required ();
		_parser.parse_args_or_exit ();
	}
	
	
	#[ cfg (feature = "profile") ]
	profiler.lock () .unwrap () .start ("./target/md5-diff.profile") .unwrap ();
	
	
	if verbose { eprintln! ("[ii] [42c3ae70]  loading..."); }
	let mut _tokens = Tokens::new (_hashes_flags.algorithm.empty, _hashes_flags.algorithm.invalid);
	let _record_pattern = regex::bytes::Regex::new (_hashes_flags.algorithm.pattern) .unwrap ();
	let _source_left = load (_path_left.as_ref (), &mut _tokens, &_record_pattern, _format_flags.zero, _compression_flags.algorithm) ?;
	let _source_right = load (_path_right.as_ref (), &mut _tokens, &_record_pattern, _format_flags.zero, _compression_flags.algorithm) ?;
	_tokens.sort ();
	
	if verbose { eprintln! ("[ii] [42c3ae70]  indexing..."); }
	let (_index_left, _statistics_left) = index (&_source_left, &_tokens);
	let (_index_right, _statistics_right) = index (&_source_right, &_tokens);
	
	if verbose { eprintln! ("[ii] [b89979a2]  diffing..."); }
	let _diff = diff (&_source_left, &_index_left, &_source_right, &_index_right, &_tokens);
	
	if verbose { eprintln! ("[ii] [92d696c3]  reporting statistics..."); }
	report_diff_statistics ('A', 'B', &_diff);
	report_source_statistics ('A', &_source_left, &_statistics_left);
	report_source_statistics ('B', &_source_right, &_statistics_right);
	
	if verbose { eprintln! ("[ii] [eedb34f8]  reporting details..."); }
	report_diff_entries ('A', 'B', &_diff, &_tokens);
	
	
	#[ cfg (feature = "profile") ]
	profiler.lock () .unwrap () .stop () .unwrap ();
	
	
	// NOTE:  We explicitly exit, so that destructors are not called...
	process::exit (0);
}




fn report_source_statistics (_tag : char, _source : & Source, _statistics : & SourceStatistics) -> () {
	
	println! ();
	println! ("##  Dataset ({}) statistics", _tag);
	println! ("##    * records                 : {:8}", _statistics.records);
	if _statistics.duplicate_paths != 0 {
	println! ("##    * paths !!!!!!!!");
	println! ("##      * distinct paths        : {:8}", _statistics.distinct_paths);
	println! ("##      * unique paths          : {:8}", _statistics.unique_paths);
	println! ("##      * duplicate paths       : {:8}", _statistics.unique_paths);
	}
	println! ("##    * hashes");
	println! ("##      * distinct hashes       : {:8}", _statistics.distinct_hashes);
	println! ("##      * unique hashes         : {:8}", _statistics.unique_hashes);
	println! ("##      * duplicate hashes      : {:8}", _statistics.duplicate_hashes);
	println! ("##    * files");
	println! ("##      * unique files          : {:8}", _statistics.unique_files);
	println! ("##      * duplicate files       : {:8}", _statistics.duplicate_files);
	println! ("##      * empty files           : {:8}", _statistics.empty_files);
	println! ("##      * invalid files         : {:8}", _statistics.invalid_files);
	println! ("##    * source: `{}`", _source.path.display ());
}


fn report_diff_statistics (_tag_left : char, _tag_right : char, _diff : & Diff) -> () {
	
	println! ();
	println! ("##  Diff statistics ({}) vs ({})", _tag_left, _tag_right);
	println! ("##    * hashes");
	println! ("##      * distinct hashes       : {:8}", _diff.by_hash_statistics.distinct);
	println! ("##      * unique hashes in ({})  : {:8}", _tag_left, _diff.by_hash_statistics.unique_left);
	println! ("##      * unique hashes in ({})  : {:8}", _tag_right, _diff.by_hash_statistics.unique_right);
	println! ("##      * common hashes         : {:8}", _diff.by_hash_statistics.matching + _diff.by_hash_statistics.conflicting);
	println! ("##        * matching paths      : {:8}", _diff.by_hash_statistics.matching);
	println! ("##        * conflicting paths   : {:8}", _diff.by_hash_statistics.conflicting);
	println! ("##    * paths");
	println! ("##      * distinct paths        : {:8}", _diff.by_path_statistics.distinct);
	println! ("##      * unique paths in ({})   : {:8}", _tag_left, _diff.by_path_statistics.unique_left);
	println! ("##      * unique paths in ({})   : {:8}", _tag_right, _diff.by_path_statistics.unique_right);
	println! ("##      * common paths          : {:8}", _diff.by_path_statistics.matching + _diff.by_path_statistics.conflicting);
	println! ("##        * matching hashes     : {:8}", _diff.by_path_statistics.matching);
	println! ("##        * conflicting hashes  : {:8}", _diff.by_path_statistics.conflicting);
}


fn report_diff_entries (_tag_left : char, _tag_right : char, _diff : & Diff, _tokens : & Tokens) -> () {
	
	let mut _unique_hashes_left : Vec<(char, char, PathKey, HashKey)> = Vec::new ();
	let mut _unique_hashes_right : Vec<(char, char, PathKey, HashKey)> = Vec::new ();
	let mut _conflicting_paths : Vec<(char, char, PathKey, HashKey)> = Vec::new ();
	let mut _renamed_hashes : Vec<(char, char, PathKey, HashKey)> = Vec::new ();
	
	for &_hash in _diff.hashes.iter () {
		if (_hash == _tokens.hash_key_empty) || (_hash == _tokens.hash_key_invalid) {
			continue;
		}
		match _diff.by_hash.get (&_hash) .unwrap () {
			DiffEntry::UniqueLeft (_paths) =>
				for &_path in _paths.iter () {
					_unique_hashes_left.push (('+', _tag_left, _path, _hash))
				},
			DiffEntry::UniqueRight (_paths) =>
				for &_path in _paths.iter () {
					_unique_hashes_right.push (('+', _tag_right, _path, _hash))
				},
			DiffEntry::Conflicting (_paths_left, _paths_right) => {
				for &_path in _paths_left.iter () {
					_renamed_hashes.push (('~', _tag_left, _path, _hash))
				}
				for &_path in _paths_right.iter () {
					_renamed_hashes.push (('~', _tag_right, _path, _hash))
				}
			},
			_ => (),
		}
	}
	
	for &_path in _diff.paths.iter () {
		match _diff.by_path.get (&_path) .unwrap () {
			DiffEntry::Conflicting (_hashes_left, _hashes_right) => {
				for &_hash in _hashes_left.iter () {
					_conflicting_paths.push (('!', _tag_left, _path, _hash))
				}
				for &_hash in _hashes_right.iter () {
					_conflicting_paths.push (('!', _tag_right, _path, _hash))
				}
			},
			_ => (),
		}
	}
	
	fn print_pairs (_pairs : &mut Vec<(char, char, PathKey, HashKey)>, _tokens : & Tokens, _sort_by_path : bool) -> () {
		println! ();
		if _sort_by_path {
			_pairs.sort_unstable_by_key (|_x| (_tokens.order_of_path (_x.2), _x.1, _tokens.order_of_hash (_x.3), _x.0));
		} else {
			_pairs.sort_unstable_by_key (|_x| (_tokens.order_of_hash (_x.3), _tokens.order_of_path (_x.2), _x.1, _x.0));
		}
		for &(_slug, _tag, _path, _hash) in _pairs.iter () {
			println! ("{}{}  {}  {}", _slug, _tag, _tokens.select_hash (_hash), _tokens.select_path (_path).to_string_lossy ());
		}
		println! ();
	}
	
	if ! _unique_hashes_left.is_empty () {
		println! ();
		println! ("####  Hashes unique in ({}) :: {}", _tag_left, _diff.by_hash_statistics.unique_left);
		print_pairs (&mut _unique_hashes_left, _tokens, true);
	}
	
	if ! _unique_hashes_right.is_empty () {
		println! ();
		println! ("####  Hashes unique in ({}) :: {}", _tag_right, _diff.by_hash_statistics.unique_right);
		print_pairs (&mut _unique_hashes_right, _tokens, true);
	}
	
	if ! _conflicting_paths.is_empty () {
		println! ();
		println! ("####  Paths conflicting in ({}) and ({}) :: {}", _tag_left, _tag_right, _diff.by_path_statistics.conflicting);
		print_pairs (&mut _conflicting_paths, _tokens, true);
	}
	
	if ! _renamed_hashes.is_empty () {
		println! ();
		println! ("####  Files re-organized in ({}) and ({}) :: {} (hashes)", _tag_left, _tag_right, _diff.by_hash_statistics.conflicting);
		print_pairs (&mut _renamed_hashes, _tokens, false);
	}
}




fn load (_path : & path::Path, _tokens : &mut Tokens, _pattern : & regex::bytes::Regex, _zero : bool, _decompressor : CompressionAlgorithm) -> (Result<Source, io::Error>) {
	
	let mut _file = fs::File::open (_path) ?;
	
	if _decompressor != CompressionAlgorithm::None {
		
		let mut _filter = match _decompressor {
			CompressionAlgorithm::Gzip => {
				let mut _filter = process::Command::new ("gzip");
				_filter.arg ("-d");
				_filter
			},
			CompressionAlgorithm::Bzip2 => {
				let mut _filter = process::Command::new ("bzip2");
				_filter.arg ("-d");
				_filter
			},
			CompressionAlgorithm::Lzip => {
				let mut _filter = process::Command::new ("lzip");
				_filter.arg ("-d");
				_filter
			},
			CompressionAlgorithm::Xz => {
				let mut _filter = process::Command::new ("xz");
				_filter.arg ("-d");
				_filter
			},
			CompressionAlgorithm::Lzma => {
				let mut _filter = process::Command::new ("lzma");
				_filter.arg ("-d");
				_filter
			},
			CompressionAlgorithm::Lz4 => {
				let mut _filter = process::Command::new ("lz4");
				_filter.arg ("-d");
				_filter
			},
			CompressionAlgorithm::Lzo => {
				let mut _filter = process::Command::new ("lzop");
				_filter.arg ("-d");
				_filter
			},
			CompressionAlgorithm::Zstd => {
				let mut _filter = process::Command::new ("zstd");
				_filter.arg ("-d");
				_filter
			},
			CompressionAlgorithm::None =>
				unreachable! ("[9c7ca4b5]"),
		};
		_filter.stdin (process::Stdio::from (_file));
		_filter.stdout (process::Stdio::piped ());
		_filter.stderr (process::Stdio::inherit ());
		
		let mut _filter = _filter.spawn () ?;
		let mut _stream = _filter.stdout.as_mut () .unwrap ();
		
		let _outcome = load_from_stream (_stream, _path, _tokens, _pattern, _zero);
		
		if _outcome.is_err () {
			_filter.kill () ?;
		}
		let _exit = _filter.wait () ?;
		if _outcome.is_ok () && ! _exit.success () {
			return Err (io::Error::new (io::ErrorKind::Other, "[7fadf032]  filter failed"));
		}
		
		return _outcome;
		
	} else {
		
		return load_from_stream (&mut _file, _path, _tokens, _pattern, _zero);
	}
}


fn load_from_stream <Stream : io::Read> (_stream : &mut Stream, _path : & path::Path, _tokens : &mut Tokens, _pattern : & regex::bytes::Regex, _zero : bool) -> (Result<Source, io::Error>) {
	
	let mut _stream = io::BufReader::with_capacity (16 * 1024 * 1024, _stream);
	
	let mut _records = Vec::with_capacity (128 * 1024);
	
	{
		let _delimiter = if _zero { b'\0' } else { b'\n' };
		let mut _buffer = Vec::with_capacity (8 * 1024);
		let mut _line : usize = 0;
		
		loop {
			
			_line += 1;
			_buffer.clear ();
			_stream.read_until (_delimiter, &mut _buffer) ?;
			
			match _buffer.pop () {
				Some (_byte) if _byte == _delimiter => (),
				Some (_byte) => _buffer.push (_byte),
				None => break,
			}
			
			if _buffer.is_empty () {
				continue;
			}
			
			if _pattern.is_match (&_buffer) {
				
				let _split = _buffer.iter () .position (|&_byte| _byte == b' ') .unwrap ();
				
				let _hash = &_buffer[.. _split];
				let _path = &_buffer[_split + 1 ..];
				
				let _hash = str::from_utf8 (_hash) .unwrap ();
				let _path = ffi::OsStr::from_bytes (_path);
				
				let _hash = _tokens.include_hash (_hash);
				let _path = _tokens.include_path (_path);
				
				let _record = SourceRecord {
						hash : _hash,
						path : _path,
						line : _line,
					};
				
				_records.push (_record);
				
			} else {
				
				if verbose { eprintln! ("[ee] [d8bd4da9] @{} {:?}", _line, ffi::OsStr::from_bytes (&_buffer)); }
				return Err (io::Error::new (io::ErrorKind::Other, "[1bd51464]  invalid record line syntax"));
			}
		}
	}
	
	let _source = Source {
			path : _path.into (),
			records : _records,
		};
	
	return Ok (_source);
}




fn index <'a> (_source : &'a Source, _tokens : &'a Tokens) -> (SourceIndex<'a>, SourceStatistics) {
	
	let _records = &_source.records;
	
	let mut _index_by_hash : HashMap<HashKey, Vec<&SourceRecord>> = HashMap::with_capacity (_records.len ());
	let mut _index_by_path : HashMap<PathKey, Vec<&SourceRecord>> = HashMap::with_capacity (_records.len ());
	
	let mut _records_count = 0;
	for (_index, _record) in _records.iter () .enumerate () {
		_index_by_hash.entry (_record.hash) .or_default () .push (_record);
		_index_by_path.entry (_record.path) .or_default () .push (_record);
		_records_count += 1;
	}
	
	let mut _distinct_hashes = 0;
	let mut _unique_hashes = 0;
	let mut _duplicate_hashes = 0;
	let mut _unique_files = 0;
	let mut _duplicate_files = 0;
	let mut _empty_files = 0;
	let mut _invalid_files = 0;
	for (&_hash, _records) in _index_by_hash.iter () {
		_distinct_hashes += 1;
		if _records.len () == 1 {
			_unique_hashes += 1;
		} else {
			_duplicate_hashes += 1;
		}
		if _hash == _tokens.hash_key_empty {
			_empty_files += _records.len ();
		} else if _hash == _tokens.hash_key_invalid {
			_invalid_files += _records.len ();
		} else if _records.len () == 1 {
			_unique_files += 1;
		} else {
			_duplicate_files += _records.len ();
		}
	}
	
	let mut _distinct_paths = 0;
	let mut _unique_paths = 0;
	let mut _duplicate_paths = 0;
	for _records in _index_by_path.values () {
		_distinct_paths += 1;
		if _records.len () == 1 {
			_unique_paths += 1;
		} else {
			_duplicate_paths += 1;
		}
	}
	
	let _index = SourceIndex {
			by_hash : _index_by_hash,
			by_path : _index_by_path,
		};
	
	let _statistics = SourceStatistics {
			records : _records_count,
			distinct_hashes : _distinct_hashes,
			unique_hashes : _unique_hashes,
			duplicate_hashes : _duplicate_hashes,
			unique_files : _unique_files,
			duplicate_files : _duplicate_files,
			empty_files : _empty_files,
			invalid_files : _invalid_files,
			distinct_paths : _distinct_paths,
			unique_paths : _unique_paths,
			duplicate_paths : _duplicate_paths,
		};
	
	return (_index, _statistics);
}




fn diff (_source_left : & Source, _index_left : & SourceIndex, _source_right : & Source, _index_right : & SourceIndex, _tokens : & Tokens) -> (Diff) {
	
	let mut _hashes = Vec::with_capacity (cmp::max (_index_left.by_hash.len (), _index_right.by_hash.len ()) * 3 / 2);
	let mut _paths = Vec::with_capacity (cmp::max (_index_left.by_path.len (), _index_right.by_path.len ()) * 3 / 2);
	
	_hashes.extend (_index_left.by_hash.keys () .cloned ());
	_paths.extend (_index_left.by_path.keys () .cloned ());
	
	_hashes.extend (_index_right.by_hash.keys () .cloned ());
	_paths.extend (_index_right.by_path.keys () .cloned ());
	
	_hashes.sort_unstable_by_key (|&_x| _tokens.order_of_hash (_x));
	_paths.sort_unstable_by_key (|&_x| _tokens.order_of_path (_x));
	
	_hashes.dedup ();
	_paths.dedup ();
	
	let mut _diff_by_hash = HashMap::with_capacity (_hashes.len ());
	let mut _diff_by_path = HashMap::with_capacity (_paths.len ());
	
	
	let mut _distinct_hashes = 0;
	let mut _unique_hashes_left = 0;
	let mut _unique_hashes_right = 0;
	let mut _matching_hashes = 0;
	let mut _conflicting_hashes = 0;
	
	for &_hash in _hashes.iter () {
		
		let _records_left = _index_left.by_hash.get (&_hash)
				.map (|_records| _records.iter () .map (|_record| _record.path) .collect::<Vec<PathKey>> ())
				.map (|mut _values| { _values.sort_unstable_by_key (|&_x| _tokens.order_of_path (_x)); _values });
		
		let _records_right = _index_right.by_hash.get (&_hash)
				.map (|_records| _records.iter () .map (|_record| _record.path) .collect::<Vec<PathKey>> ())
				.map (|mut _values| { _values.sort_unstable_by_key (|&_x| _tokens.order_of_path (_x)); _values });
		
		let _entry = match (_records_left, _records_right) {
			(Some (_records_left), Some (_records_right)) =>
				if _records_left == _records_right {
					_matching_hashes += 1;
					DiffEntry::Matching (_records_left, _records_right)
				} else {
					_conflicting_hashes += 1;
					DiffEntry::Conflicting (_records_left, _records_right)
				},
			(Some (_records_left), None) => {
				_unique_hashes_left += 1;
				DiffEntry::UniqueLeft (_records_left)
			},
			(None, Some (_records_right)) => {
				_unique_hashes_right += 1;
				DiffEntry::UniqueRight (_records_right)
			},
			(None, None) =>
				unreachable! ("[6deb2aea]"),
		};
		
		_diff_by_hash.insert (_hash, _entry);
		_distinct_hashes += 1;
	}
	
	
	let mut _distinct_paths = 0;
	let mut _unique_paths_left = 0;
	let mut _unique_paths_right = 0;
	let mut _matching_paths = 0;
	let mut _conflicting_paths = 0;
	
	for &_path in _paths.iter () {
		
		let _records_left = _index_left.by_path.get (&_path)
				.map (|_records| _records.iter () .map (|_record| _record.hash) .collect::<Vec<HashKey>> ())
				.map (|mut _values| { _values.sort_unstable_by_key (|&_x| _tokens.order_of_hash (_x)); _values });
		
		let _records_right = _index_right.by_path.get (&_path)
				.map (|_records| _records.iter () .map (|_record| _record.hash) .collect::<Vec<HashKey>> ())
				.map (|mut _values| { _values.sort_unstable_by_key (|&_x| _tokens.order_of_hash (_x)); _values });
		
		let _entry = match (_records_left, _records_right) {
			(Some (_records_left), Some (_records_right)) =>
				if _records_left == _records_right {
					_matching_paths += 1;
					DiffEntry::Matching (_records_left, _records_right)
				} else {
					_conflicting_paths += 1;
					DiffEntry::Conflicting (_records_left, _records_right)
				},
			(Some (_records_left), None) => {
				_unique_paths_left += 1;
				DiffEntry::UniqueLeft (_records_left)
			},
			(None, Some (_records_right)) => {
				_unique_paths_right += 1;
				DiffEntry::UniqueRight (_records_right)
			},
			(None, None) =>
				unreachable! ("[6deb2aea]"),
		};
		
		_diff_by_path.insert (_path, _entry);
		_distinct_paths += 1;
	}
	
	let _diff = Diff {
			hashes : _hashes,
			paths : _paths,
			by_hash : _diff_by_hash,
			by_path : _diff_by_path,
			by_hash_statistics : DiffStatistics {
					distinct : _distinct_hashes,
					matching : _matching_hashes,
					conflicting : _conflicting_hashes,
					unique_left : _unique_hashes_left,
					unique_right : _unique_hashes_right,
				},
			by_path_statistics : DiffStatistics {
					distinct : _distinct_paths,
					matching : _matching_paths,
					conflicting : _conflicting_paths,
					unique_left : _unique_paths_left,
					unique_right : _unique_paths_right,
				},
		};
	
	return _diff;
}




impl Tokens {
	
	fn new (_hash_for_empty : & HashValueRef, _hash_for_invalid : & HashValueRef) -> (Self) {
		let _size = 512 * 1024;
		let mut _tokens = Tokens {
				hashes : Vec::with_capacity (_size),
				hashes_index : HashMap::with_capacity (_size),
				hashes_order : Vec::with_capacity (_size),
				paths : Vec::with_capacity (_size),
				paths_index : HashMap::with_capacity (_size),
				paths_order : Vec::with_capacity (_size),
				hash_key_empty : 0,
				hash_key_invalid : 0,
			};
		_tokens.hash_key_empty = _tokens.include_hash (_hash_for_empty);
		_tokens.hash_key_invalid = _tokens.include_hash (_hash_for_invalid);
		return _tokens;
	}
	
	fn include_hash (&mut self, _token : &HashValueRef) -> (HashKey) {
		let _token = HashValue::from (_token);
		if let Some (&_key) = self.hashes_index.get (&_token) {
			return _key;
		} else {
			let _token = Rc::new (_token);
			let _key = self.hashes.len ();
			self.hashes.push (Rc::clone (&_token));
			self.hashes_index.insert (Rc::clone (&_token), _key);
			return _key;
		}
	}
	
	fn include_path (&mut self, _token : &PathValueRef) -> (HashKey) {
		let _token = PathValue::from (_token);
		if let Some (&_key) = self.paths_index.get (&_token) {
			return _key;
		} else {
			let _token = Rc::new (_token);
			let _key = self.paths.len ();
			self.paths.push (Rc::clone (&_token));
			self.paths_index.insert (Rc::clone (&_token), _key);
			return _key;
		}
	}
	
	fn select_hash (& self, _key : HashKey) -> (&HashValueRef) {
		return self.hashes.get (_key) .unwrap () .as_ref ();
	}
	
	fn select_path (& self, _key : PathKey) -> (&PathValueRef) {
		return self.paths.get (_key) .unwrap () .as_ref ();
	}
	
	fn order_of_hash (& self, _key : HashKey) -> (TokenOrder) {
		return self.hashes_order[_key];
	}
	
	fn order_of_path (& self, _key : PathKey) -> (TokenOrder) {
		return self.paths_order[_key];
	}
	
	fn sort (&mut self) -> () {
		
		let mut _hashes = self.hashes.iter () .map (|_token| Rc::as_ref (_token)) .collect::<Vec<&HashValue>> ();
		let mut _paths = self.paths.iter () .map (|_token| Rc::as_ref (_token)) .collect::<Vec<&PathValue>> ();
		
		let mut _hashes_order = Vec::new ();
		let mut _paths_order = Vec::new ();
		
		_hashes_order.resize (_hashes.len (), 0);
		_paths_order.resize (_paths.len (), 0);
		
		_hashes.sort_unstable ();
		_paths.sort_unstable ();
		
		for (_order, &_token) in _hashes.iter () .enumerate () {
			let &_key = self.hashes_index.get (_token) .unwrap ();
			_hashes_order[_key] = _order;
		}
		
		for (_order, &_token) in _paths.iter () .enumerate () {
			let &_key = self.paths_index.get (_token) .unwrap ();
			_paths_order[_key] = _order;
		}
		
		self.hashes_order = _hashes_order;
		self.paths_order = _paths_order;
	}
}


#[ allow (non_upper_case_globals) ]
static verbose : bool = false;

