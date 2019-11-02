
# **md5-tools** -- lightweight Rust MD5/SHA tools


> Table of contents:
>
> * [Features](#features): [`md5-create`](#md5-create-features)
> * [Usage examples](#usage-examples): [`md5-create`](#md5-create-usage), [`md5-diff`](#md5-diff-usage), [`md5-cpio`](#md5-cpio-usage)
> * [Installing](#installing)
> * [About](#about) and [Copyright and licensing](#notice-copyright-and-licensing)




## About

This is a small collection of lightweight Rust-based tools related to the MD5 and SHA family hash files:

* `md5-create` -- takes one argument (a folder or file) and generates to `stdout` an MD5/SHA hash file of all its children (that are files);
  (see [bellow some of its features](#md5-create-features) that set it apart from `md5deep` or similar tools;)
  (see [below for usage examples](#md5-create-usage);)
* `md5-diff` -- takes two MD5/SHA hash files and prints a report of differences between them;
  (see [below for usage examples](#md5-diff-usage);)
* `md5-cpio` -- reads from `stdin` a CPIO archive (in `newc` format, as created with `cpio -o -H newc`) and generates to `stdout` an MD5/SHA hash file of all the archive members (that are files);
  (see [below for usage examples](#md5-cpio-usage);)
* all these tools consume or generate files similar to those produced by `md5sum`/`sha1sum`/`sha256sum`;

I have used all of these to curate my personal file-systems and backups, and they can handle large amounts of data.
(For example `md5-diff` was used on a ~3 million lines long MD5 file.)

Regarding the insecurity of MD5:

* **although the tools are named `md5-*`, they do support the SHA family of hashes!**
* yes, I know that MD5 is cryptographically broken;
* yes, I know we should migrate to SHA-2 / SHA-3 family of hash functions;
* but for the purpose of identifying duplicate, missing, or corrupted files I personally think it is still acceptable;

There are also a few other tools and scripts found in `./sources/bin`:

* `md5-create.bash` (for now a Bash script, supporting only MD5) -- takes one argument (a folder) and creates within it (or if exists the `.md5` folder exists underneath it) a timestamped MD5 hash file of all the folder members (that are files);  (it ignores symlinks or sub-mount-points;  it also ignores folders that have a `.md5.excluded` file within it;)
* `md5-diff.go` -- the Go variant of the `md5-diff` tool;




## Features


### `md5-create` features

The following is a short list of the most important features that set this tool apart from other similar tools like `md5deep`:

* support for the [`posix_fadvise`](http://man7.org/linux/man-pages/man2/posix_fadvise.2.html) syscall that instructs the OS not to cache the hashed files in its buffers, thus reducing the OS memory pressure;
* support for the [`nice`](http://man7.org/linux/man-pages/man2/nice.2.html) syscall, that sets the OS scheduler priority (by default 19, the lowest value), thus reducing the OS CPU pressure;
* support for not crossing to other mount-points (i.e. like `find /path/... -xdev`);
* (not yet implemented) support for clustering files to be hashed by their inode-number, which usually reduces the I/O thrashing on magnetic disks;
* (not yet implemented) support for printing relative paths, relative to the root given as argument;  (this option is also supported by `md5deep`;)
* (not yet implemented) support for hashing symlinks as either their path contents, or the pointed-to file contents;




## Usage examples


### `md5-create` usage

Besides the example bellow it also supports the following features:

* `--help` -- the obvious "display help" flag;
* `--md5`, `--sha1`, `--sha224`, `--sha256`, `--sha384`, `--sha512`, `--sha3-224`, `--sha3-256`, `--sha3-384`, `--sha3-512` -- to handle files that contain hashes for these algorithms;
* `--zero` -- to handle files where lines are terminated by `\0` (as opposed by `\n`);
* `--xdev` -- when walking the file-system, do not cross into other mount points;
* `--follow` -- when walking the file-system, do follow any symlinks;  (without this option not even symlinks to files are hashed;)
* `--ignore-all-errors`, `--ignore-walk-errors`, `--ignore-open-errors`, `--ignore-read-errors` -- by default, if any errors are encountered while walking folders, opening or reading files, the hashing stops with an error;  with these options the hashing continues, but the final exit code is still non-zero;
* `--no-errors-to-stdout`, and its default `--errors-to-stdout` -- write an invalid hash record for any failed folder or file;  (i.e. an all `0000...` hash;)
* `--no-errors-to-stderr`, and its default `--errors-to-stderr` -- write an error message to `stderr` if any errors are encountered;
* `--nice <priority>` -- set the `nice` priority;  (i.e. `19` by default, the lowest priority;)
* `--fadvise` -- tell the OS that the files are read sequentially, and that their contents shouldn't be cached in the OS buffers;
* `--workers-count` -- number of parallel threads that compute hashes;
* `--workers-queue` -- size of the parallel threads queue;  (one should not touch this!)
* `--` -- denotes the end of flags, and the start of the folder or file to hash;

```
md5-create ./sources
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




### `md5-create.bash` usage

```
md5-create ./sources
```

```
cat ./sources/.--2019-10-08-20-10-08.md5
```
```
855190c3b695519378b057c1f48efdf7 *./md5-cpio.rs
8ecc4a7b226f0c499eed4852d43003e4 *./md5-create.bash
b687bba629fdef9f29ba734f9aac90e0 *./md5-diff.go
12626fb2d7784b35dfd6196fc703cf59 *./md5-diff.rs
```




## Installing


### Installing from sources

```
git clone https://github.com/cipriancraciun/md5-tools
```

```
cd ./md5-tools
```

```
cargo build --release
```

```
cp ./target/release/md5-create ~/bin/md5-create
cp ./target/release/md5-diff ~/bin/md5-diff
cp ./target/release/md5-cpio ~/bin/md5-cpio
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
