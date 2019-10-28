

package main


import "bufio"
import "crypto/md5"
import "encoding/hex"
import "fmt"
import "io"
import "os"
import "path"
import "regexp"
import "syscall"




func main () () {
	
	if len (os.Args) != 4 {
		panic ("[0071e111]  invalid arguments")
	}
	
	_hashesPath := os.Args[1]
	_sourcePath := os.Args[2]
	_targetPath := os.Args[3]
	
	var _hashesStream *bufio.Reader
	if _stream_0 , _error := os.Open (_hashesPath); _error == nil {
		_hashesStream = bufio.NewReaderSize (_stream_0, 16 * 1024 * 1024)
	} else {
		panic (_error)
	}
	
	
	for {
		
		
		// NOTE:  Parse hash lines...
		
		var _line string
		if _line_0, _error := _hashesStream.ReadString (0); _error == nil {
			_line = _line_0
			if _line[len (_line) - 1] == 0 {
				_line = _line[: len (_line) - 1]
			}
		} else if _error == io.EOF {
			if _line == "" {
				break
			} else {
				panic (fmt.Sprintf ("[9e33c96b]  invalid line `%s`", _line))
			}
		} else {
			panic (_error)
		}
		
		var _hash string
		var _path string
		if _slices := md5RecordLine.FindStringSubmatch (_line); _slices != nil {
			_hash = _slices[1]
			_path = _slices[2]
		} else {
			panic (fmt.Sprintf ("[4ac97db6]  invalid line `%s`", _line))
		}
		
		
		// NOTE:  Sanity check paths...
		
		if _path[0:2] == "./" {
			_path = _path[2:]
		} else if _path[0:1] == "/" {
			_path = _path[1:]
		}
		if (_path != path.Clean (_path)) || (_path[0:1] == "/") {
			panic (fmt.Sprintf ("[a28f4f30]  invalid path `%s`", _path))
		}
		
		
		// NOTE:  Compute source and target paths...
		
		_sourceFile := path.Join (_sourcePath, _path)
		_targetFolder_1 := path.Join (_targetPath, _hash[0:2])
		_targetFolder_2 := path.Join (_targetFolder_1, _hash[0:4])
		_targetFile := path.Join (_targetFolder_2, _hash)
		_targetFileTmp := path.Join (_targetPath, fmt.Sprintf (".tmp.%08x.%s", os.Getpid (), _hash))
		
		
		// NOTE:  Check if target file exists...
		
		if _stat, _error := os.Lstat (_targetFile); _error == nil {
			if ! _stat.Mode () .IsRegular () {
				panic (fmt.Sprintf ("[4a0ef62d]  invalid target file (non file) `%s`", _targetFile))
			} else {
//				fmt.Fprintf (os.Stderr, "[dd] [85a8bd5a]  existing target file `%s`;  skipping!\n", _targetFile)
				continue;
			}
		} else if os.IsNotExist (_error) {
			// NOP
		} else {
			panic (_error)
		}
		
		
		fmt.Fprintf (os.Stderr, "[dd] [922b3386]  cloning `%s` -> `%s`...\n", _hash, _sourceFile)
		
		
		// NOTE:  Check if source file exists and open...
		
		if _stat, _error := os.Lstat (_sourceFile); _error == nil {
			if ! _stat.Mode () .IsRegular () {
				panic (fmt.Sprintf ("[6ffb7ba4]  invalid source file (non file) `%s`", _sourceFile))
			}
		} else if os.IsNotExist (_error) {
			panic (fmt.Sprintf ("[4099f6dc]  invalid source file (not found) `%s`", _sourceFile))
		} else {
			panic (_error)
		}
		
		var _sourceStream *os.File
		if _stream_0, _error := os.Open (_sourceFile); _error == nil {
			_sourceStream = _stream_0
		} else {
			panic (_error)
		}
		
		var _sourceStat os.FileInfo
		if _stat_0, _error := _sourceStream.Stat (); _error == nil {
			_sourceStat = _stat_0
		} else {
			panic (_error)
		}
		
		
		// NOTE:  Check if target folders exist or create...
		
		if _stat, _error := os.Lstat (_targetFolder_1); _error == nil {
			if ! _stat.IsDir () {
				panic (fmt.Sprintf ("[3aa03105]  invalid target folder `%s`", _targetFolder_1))
			}
		} else if os.IsNotExist (_error) {
//			fmt.Fprintf (os.Stderr, "[dd] [d26e2ffd]  creating target folder `%s`...\n", _targetFolder_1)
			if _error := os.Mkdir (_targetFolder_1, 0700); _error != nil {
				panic (_error)
			}
		} else {
			panic (_error)
		}
		
		if _stat, _error := os.Lstat (_targetFolder_2); _error == nil {
			if ! _stat.IsDir () {
				panic (fmt.Sprintf ("[589d7b6b]  invalid target folder `%s`", _targetFolder_2))
			}
		} else if os.IsNotExist (_error) {
//			fmt.Fprintf (os.Stderr, "[dd] [d26e2ffd]  creating target folder `%s`...\n", _targetFolder_2)
			if _error := os.Mkdir (_targetFolder_2, 0700); _error != nil {
				panic (_error)
			}
		} else {
			panic (_error)
		}
		
		
		// NOTE:  Create and write temporary target file...
		//        See also: https://github.com/golang/go/issues/22397
		
		var _targetStreamTmp_1 *os.File
		if _stream_0, _error := os.OpenFile (_targetFileTmp, os.O_WRONLY | os.O_CREATE | os.O_EXCL, 0600); _error == nil {
			_targetStreamTmp_1 = _stream_0
		} else {
			panic (_error)
		}
		
		var _dataSize int64
		if _size_0, _error := io.Copy (_targetStreamTmp_1, _sourceStream); _error == nil {
			_dataSize = _size_0
		} else {
			panic (_error)
		}
		
		if _error := _targetStreamTmp_1.Chmod (0400); _error != nil {
			panic (_error)
		}
		if _error := _targetStreamTmp_1.Sync (); _error != nil {
			panic (_error)
		}
		
		
		// NOTE:  Re-open temporary target file...
		
		var _targetStreamTmp_2 *os.File
		if _stream_0, _error := os.Open (_targetFileTmp); _error == nil {
			_targetStreamTmp_2 = _stream_0
		} else {
			panic (_error)
		}
		
		
		// NOTE:  Stat and sanity check both temporary target files...
		
		var _targetStatTmp_1 os.FileInfo
		if _stat, _error := _targetStreamTmp_1.Stat (); _error == nil {
			_targetStatTmp_1 = _stat
		} else {
			panic (_error)
		}
		
		var _targetStatTmp_2 os.FileInfo
		if _stat, _error := _targetStreamTmp_2.Stat (); _error == nil {
			_targetStatTmp_2 = _stat
		} else {
			panic (_error)
		}
		
		if ! os.SameFile (_targetStatTmp_1, _targetStatTmp_2) {
			panic (fmt.Sprintf ("[6a8783b9]  invalid target file (invalid inode) `%s`", _targetFileTmp))
		}
		if _dataSize != _sourceStat.Size () {
			panic (fmt.Sprintf ("[ff0c6916]  invalid target file (invalid size) `%s`", _targetFileTmp))
		}
		if _dataSize != _targetStatTmp_1.Size () {
			panic (fmt.Sprintf ("[26176a7e]  invalid target file (invalid size) `%s`", _targetFileTmp))
		}
		if _dataSize != _targetStatTmp_2.Size () {
			panic (fmt.Sprintf ("[8df8e12d]  invalid target file (invalid size) `%s`", _targetFileTmp))
		}
		
		
		// NOTE:  Hash temporary target file...
		
		_hasher := md5.New ()
		if _size_0, _error := io.Copy (_hasher, _targetStreamTmp_2); _error == nil {
			if _size_0 != _dataSize {
				panic (fmt.Sprintf ("[fe9fa8a7]  invalid target file (invalid size) `%s`", _targetFileTmp))
			}
		} else {
			panic (_error)
		}
		_hashTmp := hex.EncodeToString (_hasher.Sum (nil) [:])
		if _hashTmp != _hash {
			panic (fmt.Sprintf ("[fe9fa8a7]  invalid target file (invalid hash) `%s`", _targetFileTmp))
		}
		
		
		// NOTE:  Rename temporary target file to actual target file...
		
		if _error := os.Rename (_targetFileTmp, _targetFile); _error != nil {
			panic (_error)
		}
		
		
		// NOTE:  Stat and sanity check actual target file...
		
		var _targetStat os.FileInfo
		if _stat_0, _error := os.Lstat (_targetFile); _error == nil {
			_targetStat = _stat_0
		} else {
			panic (_error)
		}
		
		if ! os.SameFile (_targetStatTmp_1, _targetStat) {
			panic (fmt.Sprintf ("[e7dfab4d]  invalid target file (invalid inode) `%s`", _targetFile))
		}
		
		
		// NOTE:  Close source and target files...
		
		if _error := _targetStreamTmp_1.Close (); _error != nil {
			panic (_error)
		}
		if _error := _targetStreamTmp_2.Close (); _error != nil {
			panic (_error)
		}
		if _error := _sourceStream.Close (); _error != nil {
			panic (_error)
		}
		
		
		// NOTE:  Sync folders...
		
		{
			_folderPath := _targetFile
			for {
				_folderPath = path.Dir (_folderPath)
				if _folderStream, _error := os.OpenFile (_folderPath, os.O_RDONLY | syscall.O_DIRECTORY, 0); _error == nil {
					if _error := _folderStream.Sync (); _error != nil {
						panic (_error)
					}
					if _error := _folderStream.Close (); _error != nil {
						panic (_error)
					}
				} else {
					panic (_error)
				}
				if _folderPath == _targetPath {
					break
				}
			}
		}
	}
}


var md5RecordLine *regexp.Regexp = regexp.MustCompile (`^([0-9a-f]{32}) \*(.+)$`)

