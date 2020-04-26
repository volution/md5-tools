

use ::digest;
use ::md5;
use ::sha1;
use ::sha2;
use ::sha3;


use crate::hashes::*;
use crate::prelude::*;




pub fn digest <Input : io::Read> (_hash : &HashAlgorithm, _input : &mut Input, _output : &mut Vec<u8>) -> (io::Result<()>) {
	match _hash.kind {
		HashAlgorithmKind::MD5 =>
			return digest_0::<md5::Md5, _> (_input, _output),
		HashAlgorithmKind::SHA1 =>
			return digest_0::<sha1::Sha1, _> (_input, _output),
		HashAlgorithmKind::SHA2_224 =>
			return digest_0::<sha2::Sha224, _> (_input, _output),
		HashAlgorithmKind::SHA2_256 =>
			return digest_0::<sha2::Sha256, _> (_input, _output),
		HashAlgorithmKind::SHA2_384 =>
			return digest_0::<sha2::Sha384, _> (_input, _output),
		HashAlgorithmKind::SHA2_512 =>
			return digest_0::<sha2::Sha512, _> (_input, _output),
		HashAlgorithmKind::SHA3_224 =>
			return digest_0::<sha3::Sha3_224, _> (_input, _output),
		HashAlgorithmKind::SHA3_256 =>
			return digest_0::<sha3::Sha3_256, _> (_input, _output),
		HashAlgorithmKind::SHA3_384 =>
			return digest_0::<sha3::Sha3_384, _> (_input, _output),
		HashAlgorithmKind::SHA3_512 =>
			return digest_0::<sha3::Sha3_512, _> (_input, _output),
		HashAlgorithmKind::GIT_SHA1 =>
			return digest_git_sha1::<_> (_input, _output),
	}
}


pub fn digest_0 <Hash : digest::Digest + io::Write, Input : io::Read> (_input : &mut Input, _output : &mut Vec<u8>) -> (io::Result<()>) {
	
	let mut _hasher = Hash::new ();
	io::copy (_input, &mut _hasher) ?;
	
	let _hash = _hasher.result ();
	_output.extend_from_slice (_hash.as_slice ());
	
	return Ok (());
}


pub fn digest_git_sha1 <Input : io::Read> (_input : &mut Input, _output : &mut Vec<u8>) -> (io::Result<()>) {
	
	let mut _buffer = Vec::with_capacity (128 * 1024);
	io::copy (_input, &mut _buffer) ?;
	
	use ::digest::Digest;
	let mut _hasher = sha1::Sha1::new ();
	
	let _ = write! (_hasher, "blob {}\0", _buffer.len ());
	
	_hasher.input (_buffer);
	
	let _hash = _hasher.result ();
	_output.extend_from_slice (_hash.as_slice ());
	
	return Ok (());
}

