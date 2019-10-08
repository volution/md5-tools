

#![ no_implicit_prelude ]
#![ allow (unused_imports, dead_code, non_upper_case_globals) ]


use ::std::cmp;
use ::std::env;
use ::std::ffi;
use ::std::fs;
use ::std::io;
use ::std::path;
use ::std::str;

use ::std::collections::HashMap;
use ::std::convert::{AsRef, From, Into};
use ::std::io::BufRead;
use ::std::option::{Option::Some, Option::None};
use ::std::path::{Path, PathBuf};
use ::std::rc::Rc;
use ::std::result::{Result, Result::Ok, Result::Err};
use ::std::string::String;
use ::std::vec::Vec;

use ::std::eprintln;
use ::std::println;
use ::std::panic;
use ::std::unreachable;

use ::std::clone::Clone as _;
use ::std::cmp::Ord as _;
use ::std::ops::Deref as _;
use ::std::iter::Iterator as _;
use ::std::iter::IntoIterator as _;
use ::std::iter::ExactSizeIterator as _;
use ::std::iter::Extend as _;
use ::std::os::unix::ffi::OsStrExt as _;

use ::regex;
use ::lazy_static::lazy_static;

#[ cfg (feature = "profile") ]
use ::cpuprofiler::PROFILER as profiler;




struct Source {
	path : PathBuf,
	records : Vec<SourceRecord>,
}

struct SourceRecord {
	hash : HashKey,
	path : PathKey,
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

type HashValue = String;
type HashValueRef = str;
type PathValue = ffi::OsString;
type PathValueRef = ffi::OsStr;

type HashKey = usize;
type PathKey = usize;
type TokenOrder = usize;




fn main () -> (Result<(), io::Error>) {
	
	#[ cfg (feature = "profile") ]
	profiler.lock () .unwrap () .start ("./target/md5-diff.profile") .unwrap ();
	
	let (_path_left, _path_right) = {
		
		let _arguments = env::args_os ();
		if _arguments.len () != 3 {
			return Err (io::Error::new (io::ErrorKind::Other, "[6f5bd360]  unexpected arguments"));
		}
		
		let mut _arguments = _arguments.into_iter ();
		_arguments.next () .unwrap ();
		let _path_left = _arguments.next () .unwrap ();
		let _path_right = _arguments.next () .unwrap ();
		
		(_path_left, _path_right)
	};
	
	if verbose { eprintln! ("[ii] [42c3ae70]  loading..."); }
	let mut _tokens = Tokens::new ();
	let _source_left = load (_path_left.as_ref (), &mut _tokens) ?;
	let _source_right = load (_path_right.as_ref (), &mut _tokens) ?;
	_tokens.sort ();
	
	if verbose { eprintln! ("[ii] [42c3ae70]  indexing..."); }
	let (_index_left, _statistics_left) = index (&_source_left, &_tokens);
	let (_index_right, _statistics_right) = index (&_source_right, &_tokens);
	
	if verbose { eprintln! ("[ii] [b89979a2]  analyzing..."); }
	let _diff = diff (&_source_left, &_index_left, &_source_right, &_index_right, &_tokens);
	
	if verbose { eprintln! ("[ii] [92d696c3]  reporting statistics..."); }
	report_source_statistics ('A', &_source_left, &_statistics_left);
	report_source_statistics ('B', &_source_right, &_statistics_right);
	report_diff_statistics ('A', 'B', &_diff);
	
	if verbose { eprintln! ("[ii] [eedb34f8]  reporting details..."); }
	report_diff_entries ('A', 'B', &_diff, &_tokens);
	
	#[ cfg (feature = "profile") ]
	profiler.lock () .unwrap () .stop () .unwrap ();
	
	return Ok (());
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
	
	let mut _pairs : Vec<(char, char, PathKey, HashKey)> = Vec::new ();
	
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
		_pairs.clear ();
		println! ();
	}
	
	if true {
		for &_hash in _diff.hashes.iter () {
			if (_hash == _tokens.hash_key_empty) || (_hash == _tokens.hash_key_invalid) {
				continue;
			}
			match _diff.by_hash.get (&_hash) .unwrap () {
				DiffEntry::UniqueLeft (_paths) =>
					for &_path in _paths.iter () {
						_pairs.push (('+', _tag_left, _path, _hash))
					},
				_ => (),
			}
		}
		if ! _pairs.is_empty () {
			println! ();
			println! ("####  Hashes unique in ({}) :: {}", _tag_left, _diff.by_hash_statistics.unique_left);
			print_pairs (&mut _pairs, _tokens, true);
		}
	}
	
	if true {
		for &_hash in _diff.hashes.iter () {
			if (_hash == _tokens.hash_key_empty) || (_hash == _tokens.hash_key_invalid) {
				continue;
			}
			match _diff.by_hash.get (&_hash) .unwrap () {
				DiffEntry::UniqueRight (_paths) =>
					for &_path in _paths.iter () {
						_pairs.push (('+', _tag_right, _path, _hash))
					},
				_ => (),
			}
		}
		if ! _pairs.is_empty () {
			println! ();
			println! ("####  Hashes unique in ({}) :: {}", _tag_right, _diff.by_hash_statistics.unique_right);
			print_pairs (&mut _pairs, _tokens, true);
		}
	}
	
	if true {
		for &_path in _diff.paths.iter () {
			match _diff.by_path.get (&_path) .unwrap () {
				DiffEntry::Conflicting (_hashes_left, _hashes_right) => {
					for &_hash in _hashes_left.iter () {
						_pairs.push (('!', _tag_left, _path, _hash))
					}
					for &_hash in _hashes_right.iter () {
						_pairs.push (('!', _tag_right, _path, _hash))
					}
				},
				_ => (),
			}
		}
		if ! _pairs.is_empty () {
			println! ();
			println! ("####  Paths conflicting in ({}) and ({}) :: {}", _tag_left, _tag_right, _diff.by_path_statistics.conflicting);
			print_pairs (&mut _pairs, _tokens, true);
		}
	}
	
	if true {
		for &_hash in _diff.hashes.iter () {
			if (_hash == _tokens.hash_key_empty) || (_hash == _tokens.hash_key_invalid) {
				continue;
			}
			match _diff.by_hash.get (&_hash) .unwrap () {
				DiffEntry::Conflicting (_paths_left, _paths_right) => {
					for &_path in _paths_left.iter () {
						_pairs.push (('~', _tag_left, _path, _hash))
					}
					for &_path in _paths_right.iter () {
						_pairs.push (('~', _tag_right, _path, _hash))
					}
				},
				_ => (),
			}
		}
		if ! _pairs.is_empty () {
			println! ();
			println! ("####  Files re-organized in ({}) and ({}) :: {} (hashes)", _tag_left, _tag_right, _diff.by_hash_statistics.conflicting);
			print_pairs (&mut _pairs, _tokens, false);
		}
	}
}




fn load (_path : & Path, _tokens : &mut Tokens) -> (Result<Source, io::Error>) {
	
	let _file = fs::File::open (_path) ?;
	let mut _stream = io::BufReader::with_capacity (16 * 1024 * 1024, _file);
	
	let mut _records = Vec::with_capacity (128 * 1024);
	
	{
		let mut _buffer = Vec::with_capacity (8 * 1024);
		let mut _line : usize = 0;
		
		let _record_line_pattern = record_line_pattern.deref ();
		
		loop {
			
			_line += 1;
			_buffer.clear ();
			_stream.read_until (b'\n', &mut _buffer) ?;
			
			match _buffer.pop () {
				Some (b'\n') => (),
				Some (_byte) => _buffer.push (_byte),
				None => break,
			}
			
			if _buffer.is_empty () {
				continue;
			}
			
			if _record_line_pattern.is_match (&_buffer) {
				
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
	
	fn new () -> (Self) {
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
		_tokens.hash_key_empty = _tokens.include_hash (hash_for_empty);
		_tokens.hash_key_invalid = _tokens.include_hash (hash_for_invalid);
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




lazy_static! {
	static ref record_line_pattern : regex::bytes::Regex = regex::bytes::Regex::new (r"^(?-u)([0-9a-f]{32}) ([ *])(.+)$") .unwrap ();
}

static hash_for_empty : & HashValueRef = "d41d8cd98f00b204e9800998ecf8427e";
static hash_for_invalid : & HashValueRef = "00000000000000000000000000000000";

static verbose : bool = false;

