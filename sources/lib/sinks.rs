

use crate::core::*;
use crate::prelude::*;




pub trait HashesSink {
	fn handle (&mut self, _path : & PathValueRef, _hash : & HashBytesRef) -> (Result<(), io::Error>);
	fn flush (&mut self) -> (Result<(), io::Error>);
}




pub struct StandardHashesSink <'a, Stream : io::Write> {
	stream : io::BufWriter<Stream>,
	prefix : Cow<'a, [u8]>,
	infix : Cow<'a, [u8]>,
	suffix : Cow<'a, [u8]>,
	path : bool,
	flush : bool,
}


impl <Stream : io::Write> StandardHashesSink<'static, Stream> {
	
	pub fn new (_stream : Stream, _zero : bool, _path : bool) -> (Self) {
		let _stream = io::BufWriter::with_capacity (128 * 1024, _stream);
		let _sink = StandardHashesSink {
				stream : _stream,
				prefix : Cow::Borrowed (b""),
				infix : Cow::Borrowed (b" *"),
				suffix : Cow::Borrowed (if _zero { b"\0" } else { b"\n" }),
				path : _path,
				flush : true,
			};
		return _sink;
	}
	
	pub fn done (self) -> (Result<Stream, io::Error>) {
		let _stream = self.stream.into_inner () ?;
		return Ok (_stream);
	}
}


impl <Stream : io::Write> HashesSink for StandardHashesSink<'_, Stream> {
	
	fn handle (&mut self, _path : & PathValueRef, _hash : & HashBytesRef) -> (Result<(), io::Error>) {
		self.stream.write_all (&self.prefix) ?;
		for _byte in _hash {
			self.stream.write_fmt (format_args! ("{:02x}", _byte)) ?;
		}
		if self.path {
			self.stream.write_all (&self.infix) ?;
			self.stream.write_all (_path.as_bytes ()) ?;
		}
		self.stream.write_all (&self.suffix) ?;
		if self.flush {
			self.stream.flush () ?;
		}
		return Ok (());
	}
	
	fn flush (&mut self) -> (Result<(), io::Error>) {
		return self.stream.flush ();
	}
}

