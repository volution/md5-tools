

package main


import "bufio"
import "fmt"
import "os"
import "regexp"
import "sort"


type records struct {
	name string
	hashIndex map[string][]string
	pathIndex map[string]string
	hashes []string
	paths []string
	statistics recordStatistics
}

type recordStatistics struct {
	uniqueHashes uint
	duplicateHashes uint
	paths uint
}

type diff struct {
	recordsA *records
	recordsB *records
	entries []diffEntry
	statistics diffStatistics
}

type diffEntry struct {
	source *records
	path string
	hash string
	status uint
	duplicate bool
	pathsInOther []string
}

type diffStatistics struct {
	uniqueHashesInA uint
	uniqueHashesInB uint
	sameHashes uint
	uniquePathsInA uint
	uniquePathsInB uint
	renamedPathsInA uint
	renamedPathsInB uint
	samePaths uint
	matchingPaths uint
	conflictingPaths uint
}

const (
	undefined = iota
	unique = iota
	matching = iota
	conflicting = iota
	renamed = iota
)


func main () () {
	
	if (len (os.Args) != 3) {
		abort ("invalid arguments (expected exactly 2)", nil)
	}
	
	var _recordsA *records
	{
		if _entries, _error := parseFileAtPath (os.Args[1]); _error != nil {
			abort ("failed while parsing the first record file", _error)
		} else if _records, _error := processRecords (os.Args[1], _entries); _error != nil {
			abort ("failed while processing the first record set", _error)
		} else {
			_recordsA = _records
		}
	}
	var _recordsB *records
	{
		if _entries, _error := parseFileAtPath (os.Args[2]); _error != nil {
			abort ("failed while parsing the second record file", _error)
		} else if _records, _error := processRecords (os.Args[2], _entries); _error != nil {
			abort ("failed while processing the second record set", _error)
		} else {
			_recordsB = _records
		}
	}
	
	printRecordStatistics (_recordsA)
	printRecordStatistics (_recordsB)
	
	var _diff *diff
	if _diff_, _error := processDiff (_recordsA, _recordsB); _error != nil {
		abort ("failed while diff-ing the record set", _error)
	} else {
		_diff = _diff_
	}
	
	printDiffStatistics (_diff)
	printDiffEntries (_diff)
}


func printRecordStatistics (_records *records) () {
	fmt.Fprintf (os.Stdout, "## Record set `%s` statistics ##\n", _records.name)
	fmt.Fprintf (os.Stdout, "##   * unique hashes    : %7d\n", _records.statistics.uniqueHashes)
	fmt.Fprintf (os.Stdout, "##   * duplicate hashes : %7d\n", _records.statistics.duplicateHashes)
	fmt.Fprintf (os.Stdout, "##   * paths            : %7d\n", _records.statistics.paths)
	fmt.Fprintf (os.Stdout, "##\n")
}


func printDiffStatistics (_diff *diff) () {
	fmt.Fprintf (os.Stdout, "## Record diff report\n")
	fmt.Fprintf (os.Stdout, "##   * (A) -> `%s`\n", _diff.recordsA.name)
	fmt.Fprintf (os.Stdout, "##   * (B) -> `%s`\n", _diff.recordsB.name)
	fmt.Fprintf (os.Stdout, "## (hashes)\n")
	fmt.Fprintf (os.Stdout, "##   * unique hashes in (A) : %7d\n", _diff.statistics.uniqueHashesInA)
	fmt.Fprintf (os.Stdout, "##   * unique hashes in (B) : %7d\n", _diff.statistics.uniqueHashesInB)
	fmt.Fprintf (os.Stdout, "##   * same hashes in both  : %7d\n", _diff.statistics.sameHashes)
	fmt.Fprintf (os.Stdout, "## (paths based on hashes)\n")
	fmt.Fprintf (os.Stdout, "##   * unique paths in (A)  : %7d\n", _diff.statistics.uniquePathsInA)
	fmt.Fprintf (os.Stdout, "##   * unique paths in (B)  : %7d\n", _diff.statistics.uniquePathsInB)
	fmt.Fprintf (os.Stdout, "##   * renamed paths in (A) : %7d\n", _diff.statistics.renamedPathsInA)
	fmt.Fprintf (os.Stdout, "##   * renamed paths in (B) : %7d\n", _diff.statistics.renamedPathsInA)
	fmt.Fprintf (os.Stdout, "##   * same paths in both   : %7d\n", _diff.statistics.samePaths)
	fmt.Fprintf (os.Stdout, "##     * matching paths     : %7d\n", _diff.statistics.matchingPaths)
	fmt.Fprintf (os.Stdout, "##     * conflicting paths  : %7d\n", _diff.statistics.conflictingPaths)
	fmt.Fprintf (os.Stdout, "##\n")
}


func printDiffEntries (_diff *diff) () {
	
	fmt.Fprintf (os.Stdout, "## All diff entries\n")
	fmt.Fprintf (os.Stdout, "##   * (A) -> `%s`\n", _diff.recordsA.name)
	fmt.Fprintf (os.Stdout, "##   * (B) -> `%s`\n", _diff.recordsB.name)
	fmt.Fprintf (os.Stdout, "##\n")
	for _, _entry := range _diff.entries {
		printDiffEntry (_diff, &_entry, true, true, true)
	}
	fmt.Fprintf (os.Stdout, "##\n")
	
	fmt.Fprintf (os.Stdout, "## All diff entries unique for (A)\n")
	fmt.Fprintf (os.Stdout, "##   * (A) -> `%s`\n", _diff.recordsA.name)
	fmt.Fprintf (os.Stdout, "##   * unique hashes in (A) : %7d\n", _diff.statistics.uniqueHashesInA)
	fmt.Fprintf (os.Stdout, "##\n")
	for _, _entry := range _diff.entries {
		if (_entry.status != unique) || (_entry.source != _diff.recordsA) {
			continue
		}
		printDiffEntry (_diff, &_entry, false, false, false)
	}
	fmt.Fprintf (os.Stdout, "##\n")
	
	fmt.Fprintf (os.Stdout, "## All diff entries unique for (B)\n")
	fmt.Fprintf (os.Stdout, "##   * (B) -> `%s`\n", _diff.recordsB.name)
	fmt.Fprintf (os.Stdout, "##   * unique hashes in (B) : %7d\n", _diff.statistics.uniqueHashesInB)
	fmt.Fprintf (os.Stdout, "##\n")
	for _, _entry := range _diff.entries {
		if (_entry.status != unique) || (_entry.source != _diff.recordsB) {
			continue
		}
		printDiffEntry (_diff, &_entry, false, false, false)
	}
	fmt.Fprintf (os.Stdout, "##\n")
	
	fmt.Fprintf (os.Stdout, "## All diff entries conflicting\n")
	fmt.Fprintf (os.Stdout, "##   * (A) -> `%s`\n", _diff.recordsA.name)
	fmt.Fprintf (os.Stdout, "##   * (B) -> `%s`\n", _diff.recordsB.name)
	fmt.Fprintf (os.Stdout, "##   * conflicting paths    : %7d\n", _diff.statistics.conflictingPaths)
	fmt.Fprintf (os.Stdout, "##\n")
	for _, _entry := range _diff.entries {
		if _entry.status != conflicting {
			continue
		}
		printDiffEntry (_diff, &_entry, false, false, false)
	}
	fmt.Fprintf (os.Stdout, "##\n")
}


func printDiffEntry (_diff *diff, _entry *diffEntry, _detailed bool, _alternatives bool, _duplicates bool) () {
	_sourceLabel := resolveDiffEntrySourceLabel (_diff, _entry)
	_flags := resolveDiffEntryFlags (_diff, _entry)
	switch _entry.status {
		case unique :
			if _detailed {
				fmt.Fprintf (os.Stdout, "++ %s %s %s    %s\n", _sourceLabel, _entry.hash, _flags, _entry.path)
			} else {
				fmt.Fprintf (os.Stdout, "+%s  %s\n", _sourceLabel, _entry.path)
			}
		case matching :
			if _detailed {
				fmt.Fprintf (os.Stdout, "== %s %s %s    %s\n", _sourceLabel, _entry.hash, _flags, _entry.path)
			} else {
				fmt.Fprintf (os.Stdout, "=%s  %s\n", _sourceLabel, _entry.path)
			}
		case conflicting :
			if _detailed {
				fmt.Fprintf (os.Stdout, "!! %s %s %s    %s\n", _sourceLabel, _entry.hash, _flags, _entry.path)
			} else {
				fmt.Fprintf (os.Stdout, "!%s  %s\n", _sourceLabel, _entry.path)
			}
		case renamed :
			if _detailed {
				fmt.Fprintf (os.Stdout, "~~ %s %s %s    %s\n", _sourceLabel, _entry.hash, _flags, _entry.path)
				if _alternatives {
					for _, _alternative := range _entry.pathsInOther {
						fmt.Fprintf (os.Stdout, "##                                              ~> %s\n", _alternative)
					}
				}
			} else {
				fmt.Fprintf (os.Stdout, "~%s  %s\n", _sourceLabel, _entry.path)
			}
		default :
			panic ("assertion")
	}
	if _detailed && _duplicates && _entry.duplicate {
		for _, _alternative := range _entry.source.hashIndex[_entry.hash] {
			if _alternative == _entry.path {
				continue
			}
			fmt.Fprintf (os.Stdout, "##                                              D> %s\n", _alternative)
		}
	}
}


func resolveDiffEntrySourceLabel (_diff *diff, _entry *diffEntry) (string) {
	if _entry.source == _diff.recordsA {
		return "A"
	} else if _entry.source == _diff.recordsB {
		return "B"
	} else {
		panic ("assertion")
	}
}

func resolveDiffEntryFlags (_diff *diff, _entry *diffEntry) (string) {
	if _entry.duplicate {
		return "D"
	} else {
		return " "
	}
}


func processDiff (_recordsA *records, _recordsB *records) (*diff, error) {
	
	_hashes := append (_recordsA.hashes, _recordsB.hashes ...)
	_paths := append (_recordsA.paths, _recordsB.paths ...)
	_entries := make ([]diffEntry, 0, len (_paths))
	var _statistics diffStatistics
	sort.Strings (_hashes)
	sort.Strings (_paths)
	
	for _index, _path := range _paths {
		if (_index > 0) && (_paths[_index - 1] == _path) {
			continue
		}
		
		_hashInA, _existsInA := _recordsA.pathIndex[_path]
		_hashInB, _existsInB := _recordsB.pathIndex[_path]
		
		_entryForA := diffEntry {
				source : _recordsA,
				path : _path,
				hash : _hashInA,
				status : undefined,
		}
		_entryForB := diffEntry {
				source : _recordsB,
				path : _path,
				hash : _hashInB,
				status : undefined,
		}
		
		if _existsInA && _existsInB {
			if _hashInA == _hashInB {
				_entryForA.status = matching
				_entryForB.status = matching
				_statistics.matchingPaths += 1
			} else {
				_entryForA.status = conflicting
				_entryForB.status = conflicting
				_statistics.conflictingPaths += 1
			}
			_statistics.samePaths += 1
		} else if _existsInA {
			_entryForA.status = unique
			if _pathsInB, _hashExistsInB := _recordsB.hashIndex[_hashInA]; _hashExistsInB {
				_entryForA.status = renamed
				_entryForA.pathsInOther = _pathsInB
				_statistics.renamedPathsInA += 1
			} else {
				_entryForA.status = unique
				_statistics.uniquePathsInA += 1
			}
		} else if _existsInB {
			if _pathsInA, _hashExistsInA := _recordsA.hashIndex[_hashInB]; _hashExistsInA {
				_entryForB.status = renamed
				_entryForB.pathsInOther = _pathsInA
				_statistics.renamedPathsInB += 1
			} else {
				_entryForB.status = unique
				_statistics.uniquePathsInB += 1
			}
		} else {
			panic ("assertion")
		}
		
		if _existsInA && (len (_recordsA.hashIndex[_hashInA]) > 1) {
			_entryForA.duplicate = true
		}
		if _existsInB && (len (_recordsB.hashIndex[_hashInB]) > 1) {
			_entryForB.duplicate = true
		}
		
		if _entryForA.status != undefined {
			_entries = append (_entries, _entryForA)
		}
		if _entryForB.status != undefined {
			_entries = append (_entries, _entryForB)
		}
	}
	
	for _index, _hash := range _hashes {
		if (_index > 0) && (_hashes[_index - 1] == _hash) {
			continue
		}
		_, _existsInA := _recordsA.hashIndex[_hash]
		_, _existsInB := _recordsB.hashIndex[_hash]
		if _existsInA && _existsInB {
			_statistics.sameHashes += 1
		} else if _existsInA {
			_statistics.uniqueHashesInA += 1
		} else if _existsInB {
			_statistics.uniqueHashesInB += 1
		} else {
			panic ("assertion")
		}
	}
	
	_diff := & diff {
			recordsA : _recordsA,
			recordsB : _recordsB,
			entries : _entries,
			statistics : _statistics,
	}
	
	return _diff, nil
}


func processRecords (_name string, _entries map[string][]string) (*records, error) {
	_pathIndex := make (map[string]string, len (_entries))
	_hashes := make ([]string, 0, len (_entries))
	_paths := make ([]string, 0, len (_entries))
	_duplicateHashes := 0
	for _entryHash, _entryPaths := range _entries {
		_hashes = append (_hashes, _entryHash)
		_paths = append (_paths, _entryPaths ...)
		if len (_entryPaths) > 1 {
			_duplicateHashes += 1
		}
		for _, _entryPath := range _entryPaths {
			if _, _exists := _pathIndex[_entryPath]; _exists {
				return nil, fmt.Errorf ("found duplicate path `%s`", _entryPath)
			}
			_pathIndex[_entryPath] = _entryHash
		}
	}
	sort.Strings (_hashes)
	sort.Strings (_paths)
	_records := & records {
			name : _name,
			hashIndex : _entries,
			pathIndex : _pathIndex,
			hashes : _hashes,
			paths : _paths,
			statistics : recordStatistics {
				uniqueHashes : uint (len (_hashes)),
				duplicateHashes : uint (_duplicateHashes),
				paths : uint (len (_paths)),
			},
	}
	return _records, nil
}


func parseFileAtPath (_path string) (map[string][]string, error) {
	if _file, _error := os.Open (_path); _error == nil {
		return parseFile (_file)
		defer _file.Close ()
	} else {
		return nil, _error
	}
	panic ("unreachable")
}


func parseFile (_file * os.File) (map[string][]string, error) {
	_records := make (map[string][]string, 1024)
	_scanner := bufio.NewScanner (_file)
	for _scanner.Scan () {
		if _error := parseLine (_scanner.Text (), _records); _error != nil {
			return nil, _error
		}
	}
	if _error := _scanner.Err (); _error != nil {
		return nil, _error
	}
	return _records, nil
}


func parseLine (_line string, _records map[string][]string) (error) {
	if ignoredLine.MatchString (_line) {
		return nil
	}
	if _slices := md5RecordLine.FindStringSubmatch (_line); _slices != nil {
		_hash := _slices[1]
		_path := _slices[2]
		if _paths, _exists := _records[_hash]; _exists {
			_records[_hash] = append (_paths, _path)
		} else {
			_records[_hash] = []string {_path}
		}
		return nil
	} else {
		return fmt.Errorf ("invalid record line: `%s`", _line)
	}
	panic ("unreachable")
}


func abort (_message string, _error error) () {
	fmt.Fprintf (os.Stderr, "[!!] %s\n", _message)
	fmt.Fprintf (os.Stderr, "[!!] aborting!\n")
	if _error != nil {
		fmt.Fprintf (os.Stderr, "[!!] (error:) %s\n", _error)
	}
	os.Exit (1)
	panic ("assertion-failed")
}


var ignoredLine *regexp.Regexp = regexp.MustCompile (`(^#.*$)|(^[^\t ]*$)`)
var md5RecordLine *regexp.Regexp = regexp.MustCompile (`^([0-9a-f]{32}) \*(.+)$`)


/*
type stringHeap []string

func (_index *stringHeap) Push (_value interface{}) () {
	*_index = append (*_index, _value.(string))
}

func (_index *stringHeap) Pop () (interface{}) {
	_value := _index[len (_index) - 1]
	*_index = _index[0 : len (_index) - 1]
	return _value
}

func (_index *stringHeap) Len () (int) {
	return len (_index)
}

func (_index *stringHeap) Less (i int, j int) (bool) {
	return _index[i] < _index[j]
}

func (_index *stringHeap) Swap (i int, j int) () {
	_index[i], _index[j] = _index[j], _index[i]
}
*/
