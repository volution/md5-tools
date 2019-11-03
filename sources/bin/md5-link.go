

package main


import "bufio"
import "fmt"
import "io"
import "os"
import "path"
import "regexp"
import "strconv"
import "sync"
import "syscall"




func main () () {
	
	if len (os.Args) != 5 {
		panic ("[0071e111]  invalid arguments")
	}
	
	_hashesPath := os.Args[1]
	_blobsPath := os.Args[2]
	_targetPath := os.Args[3]
	_parallelism := 16
	if _value, _error := strconv.ParseUint (os.Args[4], 10, 16); _error == nil {
		if _value != 0 {
			_parallelism = int (_value)
		}
	} else {
		panic (_error)
	}
	
	
	if _stat, _error := os.Stat (_blobsPath); _error == nil {
		if ! _stat.Mode () .IsDir () {
			panic (fmt.Sprintf ("[0337dae9]  invalid blobs folder (non folder) `%s`", _blobsPath))
		} else {
			// NOP
		}
	} else if os.IsNotExist (_error) {
		panic (fmt.Sprintf ("[e8d2029c]  invalid blobs folder (not found) `%s`", _blobsPath))
	} else {
		panic (_error)
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
		panic (_error)
	}
	
	
	var _hashesStream *bufio.Reader
	if _stream_0 , _error := os.Open (_hashesPath); _error == nil {
		_hashesStream = bufio.NewReaderSize (_stream_0, 16 * 1024 * 1024)
	} else {
		panic (_error)
	}
	
	
	_workersQueue := make (chan [2]string, _parallelism * 1024)
	_workersDone := & sync.WaitGroup {}
	for _index := 0; _index < _parallelism; _index += 1 {
		_workersDone.Add (1)
		go func () () {
			for _hash_and_path := range _workersQueue {
				_hash := _hash_and_path[0]
				_path := _hash_and_path[1]
				link (_hash, _path, _blobsPath, _targetPath)
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
		
		
		// NOTE:  Do not skip empty files...
		
		if _hash == md5EmptyHash {
			// NOP
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




func link (_hash string, _path string, _blobsPath string, _targetPath string) () {
	
	
	// NOTE:  Compute source and target paths...
	
	_blobFolder_1 := path.Join (_blobsPath, _hash[0:2])
	_blobFolder_2 := path.Join (_blobFolder_1, _hash[0:4])
	_blobFile := path.Join (_blobFolder_2, _hash)
	
	_targetFile := path.Join (_targetPath, _path)
	_targetFolder := path.Dir (_targetFile)
	
	
	// NOTE:  Check if blob file exists...
	
	var _blobStat os.FileInfo
	if _hash == md5EmptyHash {
		// NOP
	} else if _stat, _error := os.Lstat (_blobFile); _error == nil {
		if ! _stat.Mode () .IsRegular () {
			panic (fmt.Sprintf ("[8484e3c6]  invalid blob file (non file) `%s`", _blobFile))
		} else {
			_blobStat = _stat
			// NOP
		}
	} else if os.IsNotExist (_error) {
		fmt.Fprintf (os.Stderr, "[ee] [b888be36]  missing blob file `%s`;  skipping!\n", _blobFile)
		return
	} else if _error, _ok := _error.(*os.PathError); _ok && _error.Err == syscall.ENOTDIR {
		panic (fmt.Sprintf ("[931d8f4d]  invalid blob file (parent non folder) `%s`", _blobFile))
	} else {
		panic (_error)
	}
	
	
	// NOTE:  Check if source target file exists...
	
	if _stat, _error := os.Lstat (_targetFile); _error == nil {
		if ! _stat.Mode () .IsRegular () {
			fmt.Fprintf (os.Stderr, "[ee] [6ffb7ba4]  invalid target file (non file) `%s`;  ignoring!\n", _targetFile)
			return
		} else if (_hash != md5EmptyHash) && ! os.SameFile (_blobStat, _stat) {
			fmt.Fprintf (os.Stderr, "[ee] [d5c5c73f]  invalid target file (existing) `%s`;  ignoring!\n", _targetFile)
			return
		} else if (_hash == md5EmptyHash) && (_stat.Size () != 0) {
			fmt.Fprintf (os.Stderr, "[ee] [f2b11a94]  invalid target file (not empty) `%s`;  ignoring!\n", _targetFile)
			return
		} else {
//			fmt.Fprintf (os.Stderr, "[dd] [518cc370]  existing target file `%s`;  skipping!\n", _targetFile)
			return
		}
	} else if os.IsNotExist (_error) {
		// NOP
	} else if _error, _ok := _error.(*os.PathError); _ok && _error.Err == syscall.ENOTDIR {
		fmt.Fprintf (os.Stderr, "[ee] [7cd24e86]  invalid target file (parent non folder) `%s`;  ignoring!\n", _targetFile)
		return
	} else {
		panic (_error)
	}
	
	
	fmt.Fprintf (os.Stderr, "[dd] [922b3386]  linking `%s` -> `%s`...\n", _targetFile, _hash)
	
	if _error := os.MkdirAll (_targetFolder, 0700); _error != nil {
		fmt.Fprintf (os.Stderr, "[ee] [cefec6b9]  failed creating target folder `%s`;  ignoring!\n", _targetFolder);
		return
	}
	
	if _hash != md5EmptyHash {
		if _error := os.Link (_blobFile, _targetFile); _error != nil {
			fmt.Fprintf (os.Stderr, "[ee] [cefec6b9]  failed linking target file `%s`;  ignoring!\n", _targetFile);
			return
		}
	} else {
		if _file, _error := os.OpenFile (_targetFile, os.O_CREATE | os.O_EXCL | os.O_RDONLY, 0400); _error != nil {
			fmt.Fprintf (os.Stderr, "[ee] [315e2a09]  failed creating target file `%s`;  ignoring!\n", _targetFile);
			return
		} else {
			if _error := _file.Close (); _error != nil {
				fmt.Fprintf (os.Stderr, "[ee] [4eb1ecb6]  failed creating target file `%s`;  ignoring!\n", _targetFile);
				return
			}
		}
	}
}


var md5RecordLine *regexp.Regexp = regexp.MustCompile (`^([0-9a-f]{32}) \*(.+)$`)
var md5EmptyHash string = "d41d8cd98f00b204e9800998ecf8427e"
var md5InvalidHash string = "00000000000000000000000000000000"

