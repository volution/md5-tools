
# **md5-tools** -- lightweight Rust MD5/SHA tools


> Table of contents:
>
> * [Features](#features): [`md5-create`](#md5-create-features)
> * [Usage examples](#usage-examples): [`md5-create`](#md5-create-usage), [`md5-diff`](#md5-diff-usage), [`md5-cpio`](#md5-cpio-usage), [`md5-copy`](#md5-copy-usage)
> * [Installing](#installing)
> * [About](#about) and [Copyright and licensing](#notice-copyright-and-licensing)




## About

This is a small collection of lightweight and efficient Rust-based tools related to the MD5 and SHA family hash files:

* `md5-create` -- takes one argument (a folder or file) and generates an MD5/SHA hash file of all its children (that are files);
  (**this is the star of this project**, see [bellow some of its features](#md5-create-features) that set it apart from `md5deep` and other similar tools;)
  (also see [below for usage examples](#md5-create-usage);)
* `md5-diff` -- takes two MD5/SHA hash files and prints a report of differences between them;
  (see [below for usage examples](#md5-diff-usage);)
* `md5-cpio` -- reads from `stdin` a CPIO archive (in `newc` format, as created by `cpio -o -H newc`) and generates to `stdout` an MD5/SHA hash file of all the archive members (that are files);
  (see [below for usage examples](#md5-cpio-usage);)
* all these tools consume or generate files similar to those produced by `md5sum`/`sha1sum`/`sha256sum`;

I have used all of these to curate my personal file-systems and backups, and they can handle large amounts of data.
(For example `md5-diff` was used on a ~3 million lines long MD5 file.)

Regarding the insecurity of MD5:

* **although the tools are named `md5-*`, they do support the SHA family of hashes!**
* yes, I know that MD5 is cryptographically broken;
* yes, I know we should migrate to SHA-2 / SHA-3 family of hash functions;
* but for the purpose of identifying duplicate, missing, or corrupted files I personally think that MD5 it is still acceptable;

There are also a few other tools and scripts found in `./sources/bin` (which support only MD5):

* `md5-copy.go` -- takes four arguments, an MD5 hash file, a source folder, a "blobs store" folder, and a number of concurrent workers;  it will iterate through the MD5 hash file and copy from the source folder those "blobs" that are missing from the "blobs store" folder;
  this tool can be used to make a backup of all unique files (even from multiple sources), thus removing duplicates, and all that is needed to recover the files is the "blob store" and the initial MD5 hash file;
  (a "blob" is a file with a name equal to its hash value;  a "blob store" is a folder that contains (dispersed on two levels), the "blob" files;)
* `md5-diff.go` -- the Go variant of the `md5-diff` tool;
* `md5-create.bash` -- (supporting only MD5) -- takes one argument (a folder) and creates within it (or if exists the `.md5` folder exists underneath it) a timestamped MD5 hash file of all the folder members (that are files);  (it ignores symlinks or sub-mount-points;  it also ignores folders that have a `.md5.excluded` file within it;)




## Features


### `md5-create` features

The following is a short list of the most important features that set this tool apart from other similar tools like `md5deep`:

* **support for various file-access patterns** that (especially for rotating disks and RAID arrays) reduce read latencies, and thus increase read bandwidth:
  * (by default) ordered by inode, which on most filesystems translates roughly to a pattern that reads files in the order they are stored on the disk;
  * (on Linux only) ordered by extent via the [`ioctl / fiemap`](https://www.kernel.org/doc/Documentation/filesystems/fiemap.txt) syscall that yields an almost sequential access pattern, thus maximizing the I/O bandwidth and approaching the raw performance of the disks;
  * randomized, especially over networked file-systems, or other file-systems where no clear insight into how the data is actually stored;  (for example linear RAID arrays, or virtual disks;)
* **support for the [`posix_fadvise`](http://man7.org/linux/man-pages/man2/posix_fadvise.2.html) syscall** that instructs the OS not to cache the hashed files in its buffers, thus reducing the OS memory pressure;
* support for the [`nice`](http://man7.org/linux/man-pages/man2/nice.2.html) syscall, that sets the OS scheduler priority (by default 19, the lowest value), thus reducing the OS CPU pressure;
* support for progress monitoring displaying both the number of files (processed and yet to be processed), but also the size of these files;
* support for not crossing to other mount-points (i.e. like `find /path/... -xdev`);
* support for printing relative paths, relative to the root given as argument;  (this option is also supported by `md5deep`;)




## Usage examples


### `md5-create` usage

Besides the example bellow it also supports the following features:

* `--help` -- the obvious "display help" flag;
* `--md5`, `--sha1`, `--sha224`, `--sha256`, `--sha384`, `--sha512`, `--sha3-224`, `--sha3-256`, `--sha3-384`, `--sha3-512` -- to generate hashes files that contain hashes for these algorithms;  (MD5 by default;)
* `--output` -- to specify where to write the hashes (it can get quite complex, but it all should make sense in the end):
    * `-` -- write them to stdout;  (also if stdout is a terminal, it disables the progress monitoring);
    * `/dev/stdout`, `/dev/stderr`, `/dev/null`, `/dev/fd/...` -- as a special case that don't involve temporary files (as the next case does);
    * a path that does not exist -- a temporary file will be created (by using the `.tmp` suffix), and then renamed as such;  (in fact all the next cases do create a temporary file and rename it at the end;)
    * a path that exists and is a folder -- a timestamped file will be created into this folder;  (thus allowing one to keep track of the source's evolution;)
    * a path that exists and and is not a folder -- this is an error;
    * (by default) `.` which does the following auto-detection:
        * if the source is a file, then a timestamped suffix is added to this path and used as an output;  (similar to how for example `gnupg2` generates detached signature files;)
        * if the source is a folder, then:
            * if inside the source folder there is a sub-folder named `.{hash}` (where `{hash}` is `md5`, `sha1`, i.e. the name of the algorithm), or a sub-folder named `.hashes` or `.md5`, then a timestamped file is created into this sub-folder;
            * if inside the source folder there is a file named `.{hash}`, `.hashes` or `.md5` then a timestamped suffix is added to this path and used as an output;
            * else create a timestamped file prefixed with `.--` into the source folder;
* `--zero` and `--no-zero` -- whether to output hashes file where lines are terminated by `\0` (as opposed by `\n`);  (disabled by default;)
* `--relative` and `--no-relative` -- whether to output relative paths (to the source folder) in the resulting hashes file;  (enabled by default;)
* `--xdev` and `--no-xdev` -- when walking the file-system, do not cross into other mount points;  (disabled by default;)
* `--follow` and `--no-follow` -- when walking the file-system, do follow any symlinks;  (without this option not even symlinks to files are hashed;)  (disabled by default;)
* `--workers-count` -- number of parallel threads that compute hashes;  (16 by default;)
* `--workers-queue` -- size of the parallel threads queue;  (one should not touch this!)
* `--workers-batch` -- size of the files batch that is sorted before being enqueued;  (the larger the better data locality;)  (use `1` to disable batching, and thus sorting;  use the same value as the queue size, and the file-system walking and file reading become mutually exclusive (especially useful for slow rotating disks);)
* `--workers-sort` -- the sorting method for the files batch:
    * `walk` -- basically no ordering is done;
    * (by default) `inode` -- sort by inode number, which should roughly translate to sequential access patterns;
    * `inode-and-size` -- first group files by inodes modulo 128k (which basically clusters the access), then group by log2 of the size, and then order by inode;  (useful when in the same folder there are lots of small files and lots of large files intermixed;)
    * (on Linux only) `extent` -- sort by the actual physical location of the file, which yields an almost perfect sequential access pattern, thus approaching to the raw physical bandwidth;
    * `random` -- randomize files;  (useful especially for networked file-systems, or where there is no clear storage layout;)
* `--fadvise` and `--no-fadvise` -- tell the OS that the files are read sequentially, and that their contents shouldn't be cached in the OS buffers;  (enabled by default;)
* `--nice <priority>` -- set the `nice` priority;  (`19` by default, i.e. the lowest priority;)
* `--progress` and `--no-progress` -- do not monitor the progress by showing a progress bar;  (enabled by default;)
* `--errors-to-stdout` and `--no-errors-to-stdout` -- write an invalid hash record for any failed folder or file;  (i.e. an all `0000...` hash;)  (enabled by default;)
* `--errors-to-stderr` and `--no-errors-to-stderr` -- write an error message to `stderr` if any errors are encountered;  (enabled by default;)
* `--ignore-all-errors`, `--ignore-walk-errors`, `--ignore-open-errors`, `--ignore-read-errors` -- if any errors are encountered while walking folders, opening or reading files, the hashing stops with an error;  with these options the hashing continues, but the final exit code is still non-zero;  (disabled by default;)
* `--` -- denotes the end of flags, and the start of the folder or file to hash;

Example with output to timestamped file:
```
md5-create ./sources
```
```
[ii] [8cc8542c]  creating `./sources/.--2019-11-02-13-54-14.md5`...
| 00:00:00 | ==================== |     4073/s |         16 |         16 | 100% |
| 00:00:00 | ==================== |   6.67MB/s |    90.35KB |    90.35KB | 100% |
```

Example with output to stdout:
```
md5-create -o /dev/stdout ./sources
```
```
b687bba629fdef9f29ba734f9aac90e0 *./sources/md5-diff.go
855190c3b695519378b057c1f48efdf7 *./sources/md5-cpio.rs
8ecc4a7b226f0c499eed4852d43003e4 *./sources/md5-create.bash
12626fb2d7784b35dfd6196fc703cf59 *./sources/md5-diff.rs
```




### `md5-diff` usage

Besides the example bellow it also supports the following features:

* `--help` -- the obvious "display help" flag;
* `--md5`, `--sha1`, `--sha224`, `--sha256`, `--sha384`, `--sha512`, `--sha3-224`, `--sha3-256`, `--sha3-384`, `--sha3-512` -- to handle files that contain hashes for these algorithms;
* `--gzip`, `--bzip2`, `--lzip`, `--xz`, `--lzma`, `--lz4`, `--lzo`, `--zstd` -- to handle files that are compressed;  (requires those decompressors to be installed);
* `--zero` -- to handle files where lines are terminated by `\0` (as opposed by `\n`);
* `--` -- denotes the end of flags, and the start of the two files to compare;

Please note that an all zero hash (i.e. `0000....`) of the proper length is considered an "invalid file";  the normal hashing tools don't generate these hashes, but `md5-create` does it for files or folders that fail to be open or read (either due to permission or I/O errors), also `md5-cpio` does for hard-links.
Also empty files are detected by the hash of an empty string (i.e. for MD5 an empty file has the hash `d41d8cd98f00b204e9800998ecf8427e`).

Example:
```
md5-diff ./old.md5 ./new.md5
```
```
##  Diff statistics (A) vs (B)
##    * hashes
##      * distinct hashes       :     8783
##      * unique hashes in (A)  :      879
##      * unique hashes in (B)  :      884
##      * common hashes         :     7020
##        * matching paths      :     7019
##        * conflicting paths   :        1
##    * paths
##      * distinct paths        :     8353
##      * unique paths in (A)   :        1
##      * unique paths in (B)   :        6
##      * common paths          :     8346
##        * matching hashes     :     7467
##        * conflicting hashes  :      879

##  Dataset (A) statistics
##    * records                 :     8347
##    * hashes
##      * distinct hashes       :     7899
##      * unique hashes         :     7731
##      * duplicate hashes      :      168
##    * files
##      * unique files          :     7731
##      * duplicate files       :      616
##      * empty files           :        0
##      * invalid files         :        0
##    * source: `/tmp/man-a.md5`

##  Dataset (B) statistics
##    * records                 :     8352
##    * hashes
##      * distinct hashes       :     7904
##      * unique hashes         :     7736
##      * duplicate hashes      :      168
##    * files
##      * unique files          :     7736
##      * duplicate files       :      616
##      * empty files           :        0
##      * invalid files         :        0
##    * source: `/tmp/man-b.md5`

####  Hashes unique in (A) :: 879

+A  6e71ef15d96f410da0077db29dbdc0e2  */usr/share/man/man1/base32.1.gz
+A  818f379930ca7e4260795d89ef36d802  */usr/share/man/man1/base64.1.gz
+A  f590fe438cfd63d31dd8c1f4b844fc7b  */usr/share/man/man1/basename.1.gz
+A  c9361a23658e759af43c398ea7953a54  */usr/share/man/man1/basenc.1.gz
[...]

####  Hashes unique in (B) :: 884

+B  cb60a4b041a9591ecc3fba278f9fcbe5  */usr/share/man/man1/base32.1.gz
+B  851aa14b318c7a6fad7081564e04355c  */usr/share/man/man1/base64.1.gz
+B  a24f0721d88b551411de2e3f45e597ed  */usr/share/man/man1/basename.1.gz
+B  2e932d6cc6c7617c1f6e6527fe98d108  */usr/share/man/man1/basenc.1.gz
[...]

####  Paths conflicting in (A) and (B) :: 879

!A  6e71ef15d96f410da0077db29dbdc0e2  */usr/share/man/man1/base32.1.gz
!B  cb60a4b041a9591ecc3fba278f9fcbe5  */usr/share/man/man1/base32.1.gz
!A  818f379930ca7e4260795d89ef36d802  */usr/share/man/man1/base64.1.gz
!B  851aa14b318c7a6fad7081564e04355c  */usr/share/man/man1/base64.1.gz
!A  f590fe438cfd63d31dd8c1f4b844fc7b  */usr/share/man/man1/basename.1.gz
!B  a24f0721d88b551411de2e3f45e597ed  */usr/share/man/man1/basename.1.gz
!A  c9361a23658e759af43c398ea7953a54  */usr/share/man/man1/basenc.1.gz
!B  2e932d6cc6c7617c1f6e6527fe98d108  */usr/share/man/man1/basenc.1.gz
[...]

####  Files re-organized in (A) and (B) :: 1 (hashes)

~A  a1c8dc05804ea038e21cb3c175ce936c  */usr/share/man/man3/sd_event_source_ref.3.gz
~B  a1c8dc05804ea038e21cb3c175ce936c  */usr/share/man/man3/sd_event_source_disable_unref.3.gz
```




### `md5-cpio` usage

Besides the example bellow it also supports the following features:

* `--help` -- the obvious "display help" flag;
* `--md5`, `--sha1`, `--sha224`, `--sha256`, `--sha384`, `--sha512`, `--sha3-224`, `--sha3-256`, `--sha3-384`, `--sha3-512` -- to generate hashes for one of these algorithms;
* `--zero` -- to generate lines that are terminated by `\0` (as opposed by `\n`);

Example:
```
find ./sources -depth -print | cpio -o -H newc | gzip > ./archive.cpio.gz
```
```
gunzip < ./archive.cpio.gz | cpio -t -v
```
```
-rw-------   1 ciprian  ciprian     14224 Oct  8 14:02 sources/md5-diff.go
-rw-------   1 ciprian  ciprian      1698 Oct  8 01:32 sources/md5-cpio.rs
-rwx------   1 ciprian  ciprian      1017 Oct  8 20:00 sources/md5-create.bash
-rw-------   1 ciprian  ciprian     21154 Oct  8 18:13 sources/md5-diff.rs
drwx------   2 ciprian  ciprian         0 Oct  8 20:01 sources
```
```
gunzip < ./archive.cpio.gz | md5-cpio
```
```
b687bba629fdef9f29ba734f9aac90e0 *./sources/md5-diff.go
855190c3b695519378b057c1f48efdf7 *./sources/md5-cpio.rs
8ecc4a7b226f0c499eed4852d43003e4 *./sources/md5-create.bash
12626fb2d7784b35dfd6196fc703cf59 *./sources/md5-diff.rs
```




### `md5-copy` usage

Example (it expects a zero delimited file):
```
md5-copy <( tr '\n' '\0' < ./sources/.--2019-11-02-13-54-14.md5 ) ./sources /tmp/blobs 4
```
```
[dd] [922b3386]  cloning `8ecc4a7b226f0c499eed4852d43003e4` -> `sources/bin/md5-create.bash`...
[dd] [922b3386]  cloning `2dea36d55be0022488d5ee6efc9c51a2` -> `sources/bin/md5-create.rs`...
[dd] [922b3386]  cloning `68198ae4918c38335238d4d36bd1b919` -> `sources/bin/md5-diff.rs`...
[dd] [922b3386]  cloning `f7462b371a995bdb1f3974b7df5eb961` -> `sources/bin/md5-cpio.rs`...
[dd] [922b3386]  cloning `1ffde758ad4cd0383c22cbc218c51a15` -> `sources/lib/prelude.rs`...
[dd] [922b3386]  cloning `0ee1e0a22576ecf992ca61e95b502cab` -> `sources/lib/lib.rs`...
[dd] [922b3386]  cloning `dcde2297538da7268443da188d363f66` -> `sources/lib/core.rs`...
[dd] [922b3386]  cloning `a46f7044a801a39eb86dac72abd5d11e` -> `sources/lib/hashes.rs`...
[dd] [922b3386]  cloning `9757f2a654d3cadc0ee303d214d5aa05` -> `sources/lib/main_cpio.rs`...
[dd] [922b3386]  cloning `f1fcd1173154d92e2eebb7ffd1a3b082` -> `sources/bin/md5-copy.go`...
[dd] [922b3386]  cloning `30af67bf40d79ad453387fa014fa29d0` -> `sources/lib/sinks.rs`...
[dd] [922b3386]  cloning `a3f97c2ef7cf4b36d32ed08a6356d0fd` -> `sources/lib/digests.rs`...
[dd] [922b3386]  cloning `495e488e9069ce1e83b4af61cdc886d2` -> `sources/bin/md5-diff.go`...
[dd] [922b3386]  cloning `e47f2ae37b592a7d18e1efa92b43f433` -> `sources/lib/flags.rs`...
[dd] [922b3386]  cloning `d8866c9528b47056be576cd072bc9704` -> `sources/lib/main_diff.rs`...
[dd] [922b3386]  cloning `29bb1db4f8f90d8782c1643b0a9f072b` -> `sources/lib/main_create.rs`...
```




## Installing


### Installing from sources

Checkout the sources:
```
git clone https://github.com/cipriancraciun/md5-tools
```
```
cd ./md5-tools
```

Build and deploy the Rust tools:
```
cargo build --release
```
```
cp ./target/release/md5-create ~/bin/md5-create
cp ./target/release/md5-diff ~/bin/md5-diff
cp ./target/release/md5-cpio ~/bin/md5-cpio
```

Build and deploy the Go tools:
```
go build -o ./target/md5-copy ./sources/bin/md5-copy.go
```
```
cp ./target/md5-copy ~/bin/md5-copy
```




## Notice (copyright and licensing)


### Notice -- short version

The code is licensed under GPL 3 or later.


### Notice -- long version

For details about the copyright and licensing, please consult the `notice.txt` file in the `documentation/licensing` folder.

If someone requires the sources and/or documentation to be released
under a different license, please send an email to the authors,
stating the licensing requirements, accompanied with the reasons
and other details; then, depending on the situation, the authors might
release the sources and/or documentation under a different license.
