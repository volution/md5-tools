

use ::digest;
use ::md5;
use ::sha1;
use ::sha2;
use ::sha3;
use ::crc32fast;


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
		HashAlgorithmKind::CRC32 =>
			return digest_hasher_u32::<_, _> (_input, _output, crc32fast::Hasher::new, crc32fast::Hasher::finalize),
	}
}


pub fn digest_0 <Hasher : digest::Digest + io::Write, Input : io::Read> (_input : &mut Input, _output : &mut Vec<u8>) -> (io::Result<()>) {
	
	let mut _hasher = Hasher::new ();
	io::copy (_input, &mut _hasher) ?;
	
	let _hash = _hasher.finalize ();
	_output.extend_from_slice (_hash.as_slice ());
	
	return Ok (());
}


pub fn digest_hasher_u32 <Hasher : hash::Hasher, Input : io::Read> (_input : &mut Input, _output : &mut Vec<u8>, _new : impl Fn () -> Hasher, _finalize : impl Fn (Hasher) -> u32) -> (io::Result<()>) {
	
	let mut _hasher = _new ();
	
	struct Write <'a, Hasher : hash::Hasher> (&'a mut Hasher);
	impl <'a, Hasher : hash::Hasher> io::Write for Write<'a, Hasher> {
		fn write (&mut self, _data : &[u8]) -> io::Result<usize> {
			self.0.write (_data);
			Ok (_data.len ())
		}
		fn flush (&mut self) -> io::Result<()> {
			Ok (())
		}
	}
	
	io::copy (_input, &mut Write (&mut _hasher)) ?;
	
	let _hash = _finalize (_hasher) .to_be_bytes ();
	_output.extend_from_slice (_hash.as_slice ());
	
	return Ok (());
}


pub fn digest_git_sha1 <Input : io::Read> (_input : &mut Input, _output : &mut Vec<u8>) -> (io::Result<()>) {
	
	let mut _buffer = Vec::with_capacity (128 * 1024);
	io::copy (_input, &mut _buffer) ?;
	
	use ::digest::Digest;
	let mut _hasher = sha1::Sha1::new ();
	
	let _ = write! (_hasher, "blob {}\0", _buffer.len ());
	
	_hasher.update (_buffer);
	
	let _hash = _hasher.finalize ();
	_output.extend_from_slice (_hash.as_slice ());
	
	return Ok (());
}

