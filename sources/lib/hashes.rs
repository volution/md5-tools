

#[ derive (Copy, Clone, Eq, PartialEq) ]
pub enum HashAlgorithmKind {
	MD5,
	SHA1,
	SHA2_224,
	SHA2_256,
	SHA2_384,
	SHA2_512,
	SHA3_224,
	SHA3_256,
	SHA3_384,
	SHA3_512,
}


#[ derive (Copy, Clone, Eq, PartialEq) ]
pub struct HashAlgorithm {
	pub kind : HashAlgorithmKind,
	pub name : &'static str,
	pub name_lower : &'static str,
	pub empty : &'static str,
	pub invalid : &'static str,
	pub invalid_raw : &'static [u8],
	pub pattern : &'static str,
	pub suffix : &'static str,
}




pub static MD5 : HashAlgorithm = HashAlgorithm {
		kind : HashAlgorithmKind::MD5,
		name : "MD5", name_lower : "md5",
		empty :        "d41d8cd98f00b204e9800998ecf8427e",
		invalid :      "00000000000000000000000000000000",
		invalid_raw : b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
		pattern : r"^(?-u)([0-9a-f]{32}) ([ *])(.+)$",
		suffix : ".md5",
	};


pub static SHA1 : HashAlgorithm = HashAlgorithm {
		kind : HashAlgorithmKind::SHA1,
		name : "SHA1", name_lower : "sha1",
		empty :        "da39a3ee5e6b4b0d3255bfef95601890afd80709",
		invalid :      "0000000000000000000000000000000000000000",
		invalid_raw : b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
		pattern : r"^(?-u)([0-9a-f]{40}) ([ *])(.+)$",
		suffix : ".sha1",
	};


pub static SHA2_224 : HashAlgorithm = HashAlgorithm {
		kind : HashAlgorithmKind::SHA2_224,
		name : "SHA224", name_lower : "sha224",
		empty :        "d14a028c2a3a2bc9476102bb288234c415a2b01f828ea62ac5b3e42f",
		invalid :      "00000000000000000000000000000000000000000000000000000000",
		invalid_raw : b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
		pattern : r"^(?-u)([0-9a-f]{56}) ([ *])(.+)$",
		suffix : ".sha224",
	};

pub static SHA2_256 : HashAlgorithm = HashAlgorithm {
		kind : HashAlgorithmKind::SHA2_256,
		name : "SHA256", name_lower : "sha256",
		empty :        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
		invalid :      "0000000000000000000000000000000000000000000000000000000000000000",
		invalid_raw : b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
		pattern : r"^(?-u)([0-9a-f]{64}) ([ *])(.+)$",
		suffix : ".sha256",
	};

pub static SHA2_384 : HashAlgorithm = HashAlgorithm {
		kind : HashAlgorithmKind::SHA2_384,
		name : "SHA384", name_lower : "sha384",
		empty :        "38b060a751ac96384cd9327eb1b1e36a21fdb71114be07434c0cc7bf63f6e1da274edebfe76f65fbd51ad2f14898b95b",
		invalid :      "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
		invalid_raw : b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
		pattern : r"^(?-u)([0-9a-f]{96}) ([ *])(.+)$",
		suffix : ".sha384",
	};

pub static SHA2_512 : HashAlgorithm = HashAlgorithm {
		kind : HashAlgorithmKind::SHA2_512,
		name : "SHA512", name_lower : "sha512",
		empty :        "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e",
		invalid :      "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
		invalid_raw : b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
		pattern : r"^(?-u)([0-9a-f]{128}) ([ *])(.+)$",
		suffix : ".sha512",
	};


pub static SHA3_224 : HashAlgorithm = HashAlgorithm {
		kind : HashAlgorithmKind::SHA3_224,
		name : "SHA3-224", name_lower : "sha3-224",
		empty :        "6b4e03423667dbb73b6e15454f0eb1abd4597f9a1b078e3f5b5a6bc7",
		invalid :      "00000000000000000000000000000000000000000000000000000000",
		invalid_raw : b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
		pattern : r"^(?-u)([0-9a-f]{56}) ([ *])(.+)$",
		suffix : ".sha3-224",
	};

pub static SHA3_256 : HashAlgorithm = HashAlgorithm {
		kind : HashAlgorithmKind::SHA3_256,
		name : "SHA3-256", name_lower : "sha3-256",
		empty :        "a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a",
		invalid :      "0000000000000000000000000000000000000000000000000000000000000000",
		invalid_raw : b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
		pattern : r"^(?-u)([0-9a-f]{64}) ([ *])(.+)$",
		suffix : ".sha3-256",
	};

pub static SHA3_384 : HashAlgorithm = HashAlgorithm {
		kind : HashAlgorithmKind::SHA3_384,
		name : "SHA3-384", name_lower : "sha3-384",
		empty :        "0c63a75b845e4f7d01107d852e4c2485c51a50aaaa94fc61995e71bbee983a2ac3713831264adb47fb6bd1e058d5f004",
		invalid :      "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
		invalid_raw : b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
		pattern : r"^(?-u)([0-9a-f]{96}) ([ *])(.+)$",
		suffix : ".sha3-384",
	};

pub static SHA3_512 : HashAlgorithm = HashAlgorithm {
		kind : HashAlgorithmKind::SHA3_512,
		name : "SHA3-512", name_lower : "sha3-512",
		empty :        "a69f73cca23a9ac5c8b567dc185a756e97c982164fe25859e0d1dcc1475c80a615b2123af1f5f94c11e3e9402c3ac558f500199d95b6d3e301758586281dcd26",
		invalid :      "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
		invalid_raw : b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
		pattern : r"^(?-u)([0-9a-f]{128}) ([ *])(.+)$",
		suffix : ".sha3-512",
	};

