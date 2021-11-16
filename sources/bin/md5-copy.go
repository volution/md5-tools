

package main


import "bufio"
import "crypto/md5"
import "encoding/hex"
import "fmt"
import "io"
import "os"
import "path"
import "path/filepath"
import "regexp"
import "strconv"
import "sync"
import "syscall"




func main () () {
	
	
	if len (os.Args) != 7 {
		panic ("[0071e111]  invalid arguments, expected:  <hashes> <source> <target> <target-suffix> <target-levels> <parallelism>")
	}
	
	_hashesPath := os.Args[1]
	
	_sourcePath := os.Args[2]
	if _path, _error := filepath.EvalSymlinks (_sourcePath); _error == nil {
		_sourcePath = _path
	} else {
		panic (fmt.Sprintf ("[2c3601cc]  invalid source path (eval-links failed) `%s`:  %s", _sourcePath, _error))
	}
	
	_targetPath := os.Args[3]
	if _path, _error := filepath.EvalSymlinks (_targetPath); _error == nil {
		_targetPath = _path
	} else {
		panic (fmt.Sprintf ("[43402d50]  invalid target path (eval-links failed) `%s`:  %s", _targetPath, _error))
	}
	
	_targetSuffix := os.Args[4]
	
	_targetLevels := -1
	if _value, _error := strconv.ParseUint (os.Args[5], 10, 16); _error == nil {
		_targetLevels = int (_value)
	} else {
		panic (fmt.Sprintf ("[7f407004]  invalid target levels (parse failed) `%s`:  %s", os.Args[5], _error))
	}
	if (_targetLevels < 0) || (_targetLevels > 2) {
		panic (fmt.Sprintf ("[ef8c8ebc]  invalid target levels (must be between 0 and 2) `%s`", _targetLevels))
	}
	
	_parallelism := 16
	if _value, _error := strconv.ParseUint (os.Args[6], 10, 16); _error == nil {
		if _value != 0 {
			_parallelism = int (_value)
		}
	} else {
		panic (fmt.Sprintf ("[04d78872]  invalid parallelism (parse failed) `%s`:  %s", os.Args[6], _error))
	}
	if (_parallelism < 1) || (_parallelism > 128) {
		panic (fmt.Sprintf ("[29f6c5c4]  invalid parallelism (must be between 1 and 128) `%s`", _parallelism))
	}
	
	
	if _stat, _error := os.Stat (_sourcePath); _error == nil {
		if ! _stat.Mode () .IsDir () {
			panic (fmt.Sprintf ("[0337dae9]  invalid source folder (non folder) `%s`", _sourcePath))
		} else {
			// NOP
		}
	} else if os.IsNotExist (_error) {
		panic (fmt.Sprintf ("[e8d2029c]  invalid source folder (not found) `%s`", _sourcePath))
	} else {
		panic (fmt.Sprintf ("[9fd05bc7]  invalid source folder (unexpected error) `%s`:  %s", _sourcePath, _error))
	}
	
	if _stat, _error := os.Stat (_targetPath); _error == nil {
		if ! _stat.Mode () .IsDir () {
			panic (fmt.Sprintf ("[f6ea9a41]  invalid target folder (non folder) `%s`", _targetPath))
		} else {
			// NOP
		}
	} else if os.IsNotExist (_error) {
		panic (fmt.Sprintf ("[b9843cd6]  invalid target folder (not found) `%s`", _targetPath))
	} else {
		panic (fmt.Sprintf ("[5dacb884]  invalid target folder (unexpected error) `%s`:  %s", _targetPath, _error))
	}
	
	
	var _hashesStream *bufio.Reader
	if _stream_0 , _error := os.Open (_hashesPath); _error == nil {
		_hashesStream = bufio.NewReaderSize (_stream_0, 16 * 1024 * 1024)
	} else if os.IsNotExist (_error) {
		panic (fmt.Sprintf ("[931f9e3f]  invalid hashes file (not found) `%s`", _hashesPath))
	} else {
		panic (fmt.Sprintf ("[3d79f70b]  invalid hashes file (unexpected error) `%s`:  %s", _hashesPath, _error))
	}
	
	
	_workersQueue := make (chan [2]string, _parallelism * 1024)
	_workersDone := & sync.WaitGroup {}
	for _index := 0; _index < _parallelism; _index += 1 {
		_workersDone.Add (1)
		go func () () {
			for _hash_and_path := range _workersQueue {
				_hash := _hash_and_path[0]
				_path := _hash_and_path[1]
				copy (_hash, _path, _sourcePath, _targetPath, _targetSuffix, _targetLevels)
			}
			_workersDone.Done ()
		} ()
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
			panic (fmt.Sprintf ("[14519a7a]  unexpected error:  %s", _error))
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
		
		
		// NOTE:  Skip empty files...
		
		if _hash == md5EmptyHash {
			continue
		}
		
		// NOTE:  Skip invalid files...
		
		if _hash == md5InvalidHash {
			continue
		}
		
		_workersQueue <- [2]string { _hash, _path }
	}
	
	close (_workersQueue)
	_workersDone.Wait ()
}




func copy (_hash string, _path string, _sourcePath string, _targetPath string, _targetSuffix string, _targetLevels int) () {
	
	// NOTE:  Compute source and target paths...
	
	_sourceFile := path.Join (_sourcePath, _path)
	
	var _targetFolders []string
	var _targetFolder_X string
	if _targetLevels == 0 {
		_targetFolder_X = _targetPath
	} else if _targetLevels == 1 {
		_targetFolder_1 := path.Join (_targetPath, _hash[0:2])
		_targetFolders = append (_targetFolders, _targetFolder_1)
		_targetFolder_X = _targetFolder_1
	} else if _targetLevels == 2 {
		_targetFolder_1 := path.Join (_targetPath, _hash[0:2])
		_targetFolder_2 := path.Join (_targetFolder_1, _hash[0:4])
		_targetFolders = append (_targetFolders, _targetFolder_1, _targetFolder_2)
		_targetFolder_X = _targetFolder_2
	} else {
		panic ("[e48df570]")
	}
	
	_targetFile := path.Join (_targetFolder_X, _hash)
	if _targetSuffix != "" {
		_targetFile += _targetSuffix
	}
	_targetFileTmp := path.Join (_targetPath, fmt.Sprintf (".tmp.%08x.%s", os.Getpid (), _hash))
	
	
	// NOTE:  Check if target file exists...
	
	if _stat, _error := os.Lstat (_targetFile); _error == nil {
		if ! _stat.Mode () .IsRegular () {
			panic (fmt.Sprintf ("[4a0ef62d]  invalid target file (non file) `%s`", _targetFile))
		} else {
//			fmt.Fprintf (os.Stderr, "[dd] [85a8bd5a]  existing target file `%s`;  skipping!\n", _targetFile)
			return
		}
	} else if os.IsNotExist (_error) {
		// NOP
	} else if _error, _ok := _error.(*os.PathError); _ok && _error.Err == syscall.ENOTDIR {
		panic (fmt.Sprintf ("[26c24a68]  invalid target file (parent non folder) `%s`", _targetFile))
	} else {
		panic (fmt.Sprintf ("[87e53618]  unexpected error:  %s", _error))
	}
	
	
	fmt.Fprintf (os.Stderr, "[dd] [922b3386]  cloning `%s` -> `%s`...\n", _hash, _sourceFile)
	
	
	// NOTE:  Check if source file exists and open...
	
	if _stat, _error := os.Lstat (_sourceFile); _error == nil {
		if ! _stat.Mode () .IsRegular () {
			fmt.Fprintf (os.Stderr, "[ee] [6ffb7ba4]  invalid source file (non file) `%s`;  ignoring!\n", _sourceFile)
			return
		} else {
			// NOP
		}
	} else if os.IsNotExist (_error) {
		fmt.Fprintf (os.Stderr, "[ee] [6cf84aa8]  invalid source file (not found) `%s`;  ignoring!\n", _sourceFile)
		return
	} else if _error, _ok := _error.(*os.PathError); _ok && _error.Err == syscall.ENOTDIR {
		fmt.Fprintf (os.Stderr, "[ee] [9c5ed744]  invalid source file (parent non folder) `%s`;  ignoring!\n", _sourceFile)
		return
	} else {
		panic (fmt.Sprintf ("[88e79792]  unexpected error:  %s", _error))
	}
	
	var _sourceStream *os.File
	if _stream_0, _error := os.Open (_sourceFile); _error == nil {
		_sourceStream = _stream_0
	} else {
		panic (fmt.Sprintf ("[81408611]  unexpected error:  %s", _error))
	}
	
	var _sourceStat os.FileInfo
	if _stat_0, _error := _sourceStream.Stat (); _error == nil {
		_sourceStat = _stat_0
	} else {
		panic (fmt.Sprintf ("[5d4649c4]  unexpected error:  %s", _error))
	}
	
	
	// NOTE:  Check if target folders exist or create...
	
	for _, _targetFolder := range _targetFolders {
		if _stat, _error := os.Lstat (_targetFolder); _error == nil {
			if ! _stat.IsDir () {
				panic (fmt.Sprintf ("[3aa03105]  invalid target folder `%s`", _targetFolder))
			}
		} else if os.IsNotExist (_error) {
//			fmt.Fprintf (os.Stderr, "[dd] [d26e2ffd]  creating target folder `%s`...\n", _targetFolder)
			if _error := os.Mkdir (_targetFolder, 0700); (_error != nil) && ! os.IsExist (_error) {
				panic (fmt.Sprintf ("[7946185f]  unexpected error:  %s", _error))
			}
		} else {
			panic (fmt.Sprintf ("[33e41e43]  unexpected error:  %s", _error))
		}
	}
	
	
	// NOTE:  Create and write temporary target file...
	//        See also: https://github.com/golang/go/issues/22397
	
	var _targetStreamTmp_1 *os.File
	if _stream_0, _error := os.OpenFile (_targetFileTmp, os.O_WRONLY | os.O_CREATE | os.O_EXCL, 0600); _error == nil {
		_targetStreamTmp_1 = _stream_0
	} else if os.IsExist (_error) {
		_sourceStream.Close ()
		return
	} else {
		panic (fmt.Sprintf ("[cd5941c6]  unexpected error:  %s", _error))
	}
	
	{
		var _error error
		_error = Fadvise (_sourceStream.Fd (), 0, 0, FADV_SEQUENTIAL)
		if _error != nil { panic (fmt.Sprintf ("[0dce2e31]  unexpected error:  %s", _error)) }
		_error = Fadvise (_sourceStream.Fd (), 0, 0, FADV_NOREUSE)
		if _error != nil { panic (fmt.Sprintf ("[96737a83]  unexpected error:  %s", _error)) }
		_error = Fadvise (_sourceStream.Fd (), 0, 0, FADV_WILLNEED)
		if _error != nil { panic (fmt.Sprintf ("[b749c725]  unexpected error:  %s", _error)) }
	}
	
	var _dataSize int64
	if _size_0, _error := io.Copy (_targetStreamTmp_1, _sourceStream); _error == nil {
		_dataSize = _size_0
	} else {
		panic (fmt.Sprintf ("[4ea17054]  unexpected error:  %s", _error))
	}
	
	{
		var _error error
		_error = Fadvise (_sourceStream.Fd (), 0, 0, FADV_DONTNEED)
		if _error != nil { panic (fmt.Sprintf ("[210d6e1f]  unexpected error:  %s", _error)) }
	}
	
	if _error := _targetStreamTmp_1.Chmod (0400); _error != nil {
		panic (fmt.Sprintf ("[b3be47d6]  unexpected error:  %s", _error))
	}
	if _error := _targetStreamTmp_1.Sync (); _error != nil {
		panic (fmt.Sprintf ("[8bd8f281]  unexpected error:  %s", _error))
	}
	
	
	// NOTE:  Re-open temporary target file...
	
	var _targetStreamTmp_2 *os.File
	if _stream_0, _error := os.Open (_targetFileTmp); _error == nil {
		_targetStreamTmp_2 = _stream_0
	} else {
		panic (fmt.Sprintf ("[05b96651]  unexpected error:  %s", _error))
	}
	
	
	// NOTE:  Stat and sanity check both temporary target files...
	
	var _targetStatTmp_1 os.FileInfo
	if _stat, _error := _targetStreamTmp_1.Stat (); _error == nil {
		_targetStatTmp_1 = _stat
	} else {
		panic (fmt.Sprintf ("[5a22c74c]  unexpected error:  %s", _error))
	}
	
	var _targetStatTmp_2 os.FileInfo
	if _stat, _error := _targetStreamTmp_2.Stat (); _error == nil {
		_targetStatTmp_2 = _stat
	} else {
		panic (fmt.Sprintf ("[2a9b35cc]  unexpected error:  %s", _error))
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
		panic (fmt.Sprintf ("[10892bcb]  unexpected error:  %s", _error))
	}
	_hashTmp := hex.EncodeToString (_hasher.Sum (nil) [:])
	if _hashTmp != _hash {
		panic (fmt.Sprintf ("[fe9fa8a7]  invalid target file (invalid hash) `%s`", _targetFileTmp))
	}
	
	
	// NOTE:  Rename temporary target file to actual target file...
	
	if _error := os.Rename (_targetFileTmp, _targetFile); _error != nil {
		panic (fmt.Sprintf ("[90d364ab]  unexpected error:  %s", _error))
	}
	
	
	// NOTE:  Stat and sanity check actual target file...
	
	var _targetStat os.FileInfo
	if _stat_0, _error := os.Lstat (_targetFile); _error == nil {
		_targetStat = _stat_0
	} else {
		panic (fmt.Sprintf ("[aa2d7afe]  unexpected error:  %s", _error))
	}
	
	if ! os.SameFile (_targetStatTmp_1, _targetStat) {
		panic (fmt.Sprintf ("[e7dfab4d]  invalid target file (invalid inode) `%s`", _targetFile))
	}
	
	
	// NOTE:  Close source and target files...
	
	{
		var _error error
		_error = Fadvise (_targetStreamTmp_1.Fd (), 0, 0, FADV_DONTNEED)
		if _error != nil { panic (fmt.Sprintf ("[11fac409]  unexpected error:  %s", _error)) }
	}
	
	if _error := _targetStreamTmp_1.Close (); _error != nil {
		panic (fmt.Sprintf ("[293f4d9a]  unexpected error:  %s", _error))
	}
	if _error := _targetStreamTmp_2.Close (); _error != nil {
		panic (fmt.Sprintf ("[a7f2341e]  unexpected error:  %s", _error))
	}
	if _error := _sourceStream.Close (); _error != nil {
		panic (fmt.Sprintf ("[724b639a]  unexpected error:  %s", _error))
	}
	
	
	// NOTE:  Sync folders...
	
	{
		_folderPath := _targetFile
		for {
			_folderPath = path.Dir (_folderPath)
			if _folderStream, _error := os.OpenFile (_folderPath, os.O_RDONLY | syscall.O_DIRECTORY, 0); _error == nil {
				if _error := _folderStream.Sync (); _error != nil {
					panic (fmt.Sprintf ("[2e17ce7d]  unexpected error:  %s", _error))
				}
				if _error := _folderStream.Close (); _error != nil {
					panic (fmt.Sprintf ("[09934cf8]  unexpected error:  %s", _error))
				}
			} else {
				panic (fmt.Sprintf ("[e55f9fa4]  unexpected error:  %s", _error))
			}
			if _folderPath == _targetPath {
				break
			}
		}
	}
}


var md5RecordLine *regexp.Regexp = regexp.MustCompile (`^([0-9a-f]{32}) \*(.+)$`)
var md5EmptyHash string = "d41d8cd98f00b204e9800998ecf8427e"
var md5InvalidHash string = "00000000000000000000000000000000"




// NOTE:  https://github.com/golang/sys/blob/master/unix/zsyscall_linux_amd64.go#L1800
func Fadvise(fd uintptr, offset int64, length int64, advice int) (error) {
	_, _, e := syscall.Syscall6(syscall.SYS_FADVISE64, uintptr(fd), uintptr(offset), uintptr(length), uintptr(advice), 0, 0)
	if e == 0 {
		return nil
	} else {
		return e
	}
}

// NOTE:  https://github.com/golang/sys/blob/master/unix/ztypes_linux_amd64.go#L188
const (
	FADV_NORMAL     = 0x0
	FADV_RANDOM     = 0x1
	FADV_SEQUENTIAL = 0x2
	FADV_WILLNEED   = 0x3
	FADV_DONTNEED   = 0x4
	FADV_NOREUSE    = 0x5
)

