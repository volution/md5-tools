

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
	hash : Hash,
	path : Path0,
	line : usize,
}

struct SourceIndex <'a> {
	by_hash : HashMap<&'a Hash, Vec<&'a SourceRecord>>,
	by_path : HashMap<&'a Path0, Vec<&'a SourceRecord>>,
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


struct Diff <'a> {
	hashes : Vec<&'a Hash>,
	paths : Vec<&'a Path0>,
	by_hash : HashMap<&'a Hash, DiffEntry<&'a Path0>>,
	by_path : HashMap<&'a Path0, DiffEntry<&'a Hash>>,
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


type Hash = String;
// type Path0 = PathBuf;
type Path0 = ffi::OsString;




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
	let _source_left = load (_path_left.as_ref ()) ?;
	let _source_right = load (_path_right.as_ref ()) ?;
	
	if verbose { eprintln! ("[ii] [42c3ae70]  indexing..."); }
	let (_index_left, _statistics_left) = index (&_source_left);
	let (_index_right, _statistics_right) = index (&_source_right);
	
	if verbose { eprintln! ("[ii] [b89979a2]  analyzing..."); }
	let _diff = diff (&_source_left, &_index_left, &_source_right, &_index_right);
	
	if verbose { eprintln! ("[ii] [92d696c3]  reporting statistics..."); }
	report_source_statistics ('A', &_source_left, &_statistics_left);
	report_source_statistics ('B', &_source_right, &_statistics_right);
	report_diff_statistics ('A', 'B', &_diff);
	
	if verbose { eprintln! ("[ii] [eedb34f8]  reporting details..."); }
	report_diff_entries ('A', 'B', &_diff);
	
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


fn report_diff_entries (_tag_left : char, _tag_right : char, _diff : & Diff) -> () {
	
	let mut _pairs : Vec<(char, char, &Path0, &Hash)> = Vec::new ();
	
	fn print_pairs (_pairs : &mut Vec<(char, char, &Path0, &Hash)>, _sort_by_path : bool) -> () {
		println! ();
		if _sort_by_path {
			_pairs.sort_unstable_by_key (|a| (a.2, a.1, a.3, a.0));
		} else {
			_pairs.sort_unstable_by_key (|a| (a.3, a.2, a.1, a.0));
		}
		for (_slug, _tag, _path, _hash) in _pairs.iter () {
			println! ("{}{}  {}  {}", _slug, _tag, _hash, _path.to_string_lossy ());
		}
		_pairs.clear ();
		println! ();
	}
	
	if true {
		for _hash in _diff.hashes.iter () {
			if (*_hash == hash_for_empty) || (*_hash == hash_for_invalid) {
				continue;
			}
			match _diff.by_hash.get (_hash) .unwrap () {
				DiffEntry::UniqueLeft (_paths) =>
					for _path in _paths.iter () {
						_pairs.push (('+', _tag_left, _path, _hash))
					},
				_ => (),
			}
		}
		if ! _pairs.is_empty () {
			println! ();
			println! ("####  Hashes unique in ({}) :: {}", _tag_left, _diff.by_hash_statistics.unique_left);
			print_pairs (&mut _pairs, true);
		}
	}
	
	if true {
		for _hash in _diff.hashes.iter () {
			if (*_hash == hash_for_empty) || (*_hash == hash_for_invalid) {
				continue;
			}
			match _diff.by_hash.get (_hash) .unwrap () {
				DiffEntry::UniqueRight (_paths) =>
					for _path in _paths.iter () {
						_pairs.push (('+', _tag_right, _path, _hash))
					},
				_ => (),
			}
		}
		if ! _pairs.is_empty () {
			println! ();
			println! ("####  Hashes unique in ({}) :: {}", _tag_right, _diff.by_hash_statistics.unique_right);
			print_pairs (&mut _pairs, true);
		}
	}
	
	if true {
		for _path in _diff.paths.iter () {
			match _diff.by_path.get (_path) .unwrap () {
				DiffEntry::Conflicting (_hashes_left, _hashes_right) => {
					for _hash in _hashes_left.iter () {
						_pairs.push (('!', _tag_left, _path, _hash))
					}
					for _hash in _hashes_right.iter () {
						_pairs.push (('!', _tag_right, _path, _hash))
					}
				},
				_ => (),
			}
		}
		if ! _pairs.is_empty () {
			println! ();
			println! ("####  Paths conflicting in ({}) and ({}) :: {}", _tag_left, _tag_right, _diff.by_path_statistics.conflicting);
			print_pairs (&mut _pairs, true);
		}
	}
	
	if true {
		for _hash in _diff.hashes.iter () {
			if (*_hash == hash_for_empty) || (*_hash == hash_for_invalid) {
				continue;
			}
			match _diff.by_hash.get (_hash) .unwrap () {
				DiffEntry::Conflicting (_paths_left, _paths_right) => {
					for _path in _paths_left.iter () {
						_pairs.push (('~', _tag_left, _path, _hash))
					}
					for _path in _paths_right.iter () {
						_pairs.push (('~', _tag_right, _path, _hash))
					}
				},
				_ => (),
			}
		}
		if ! _pairs.is_empty () {
			println! ();
			println! ("####  Files re-organized in ({}) and ({}) :: {} (hashes)", _tag_left, _tag_right, _diff.by_hash_statistics.conflicting);
			print_pairs (&mut _pairs, false);
		}
	}
}




fn load (_path : & Path) -> (Result<Source, io::Error>) {
	
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
			
			if let Some (_captures) = _record_line_pattern.captures (&_buffer) {
				
				let _hash = _captures.get (1) .unwrap () .as_bytes ();
				let _path = _captures.get (3) .unwrap () .as_bytes ();
				
				let _hash = Hash::from (str::from_utf8 (_hash) .unwrap ());
				let _path = Path0::from (ffi::OsStr::from_bytes (_path));
				
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
	
	_records.sort_unstable_by (|_left, _right| Path0::cmp (&_left.path, &_right.path));
	
	let _source = Source {
			path : _path.into (),
			records : _records,
		};
	
	return Ok (_source);
}




fn index (_source : & Source) -> (SourceIndex, SourceStatistics) {
	
	let _records = &_source.records;
	
	let mut _index_by_hash : HashMap<&Hash, Vec<&SourceRecord>> = HashMap::with_capacity (_records.len ());
	let mut _index_by_path : HashMap<&Path0, Vec<&SourceRecord>> = HashMap::with_capacity (_records.len ());
	
	let mut _records_count = 0;
	for (_index, _record) in _records.iter () .enumerate () {
		_index_by_hash.entry (&_record.hash) .or_default () .push (_record);
		_index_by_path.entry (&_record.path) .or_default () .push (_record);
		_records_count += 1;
	}
	
	let mut _distinct_hashes = 0;
	let mut _unique_hashes = 0;
	let mut _duplicate_hashes = 0;
	let mut _unique_files = 0;
	let mut _duplicate_files = 0;
	let mut _empty_files = 0;
	let mut _invalid_files = 0;
	for (_hash, _records) in _index_by_hash.iter () {
		_distinct_hashes += 1;
		if _records.len () == 1 {
			_unique_hashes += 1;
		} else {
			_duplicate_hashes += 1;
		}
		if *_hash == hash_for_empty {
			_empty_files += _records.len ();
		} else if *_hash == hash_for_invalid {
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




fn diff <'a> (_source_left : &'a Source, _index_left : &'a SourceIndex, _source_right : &'a Source, _index_right : &'a SourceIndex) -> (Diff<'a>) {
	
	let mut _hashes = Vec::with_capacity (cmp::max (_index_left.by_hash.len (), _index_right.by_hash.len ()) * 3 / 2);
	let mut _paths = Vec::with_capacity (cmp::max (_index_left.by_path.len (), _index_right.by_path.len ()) * 3 / 2);
	
	_hashes.extend (_index_left.by_hash.keys ());
	_paths.extend (_index_left.by_path.keys ());
	
	_hashes.extend (_index_right.by_hash.keys ());
	_paths.extend (_index_right.by_path.keys ());
	
	_hashes.sort_unstable ();
	_paths.sort_unstable ();
	
	_hashes.dedup ();
	_paths.dedup ();
	
	let mut _diff_by_hash = HashMap::with_capacity (_hashes.len ());
	let mut _diff_by_path = HashMap::with_capacity (_paths.len ());
	
	
	let mut _distinct_hashes = 0;
	let mut _unique_hashes_left = 0;
	let mut _unique_hashes_right = 0;
	let mut _matching_hashes = 0;
	let mut _conflicting_hashes = 0;
	
	for _hash in _hashes.iter () {
		let _hash = *_hash;
		
		let _records_left = _index_left.by_hash.get (_hash)
				.map (|_records| _records.iter () .map (|_record| &_record.path) .collect::<Vec<&Path0>> ())
				.map (|mut _values| { _values.sort_unstable (); _values });
		
		let _records_right = _index_right.by_hash.get (_hash)
				.map (|_records| _records.iter () .map (|_record| &_record.path) .collect::<Vec<&Path0>> ())
				.map (|mut _values| { _values.sort_unstable (); _values });
		
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
	
	for _path in _paths.iter () {
		let _path = *_path;
		
		let _records_left = _index_left.by_path.get (_path)
				.map (|_records| _records.iter () .map (|_record| &_record.hash) .collect::<Vec<&Hash>> ())
				.map (|mut _values| { _values.sort_unstable (); _values });
		
		let _records_right = _index_right.by_path.get (_path)
				.map (|_records| _records.iter () .map (|_record| &_record.hash) .collect::<Vec<&Hash>> ())
				.map (|mut _values| { _values.sort_unstable (); _values });
		
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




lazy_static! {
	static ref record_line_pattern : regex::bytes::Regex = regex::bytes::Regex::new (r"^(?-u)([0-9a-f]{32}) ([ *])(.+)$") .unwrap ();
}

static hash_for_empty : & str = "d41d8cd98f00b204e9800998ecf8427e";
static hash_for_invalid : & str = "00000000000000000000000000000000";

static verbose : bool = false;

