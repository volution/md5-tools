

use crate::core::*;
use crate::prelude::*;




pub trait HashesSink {
	fn handle (&mut self, _path : & PathValueRef, _hash : & HashBytesRef) -> (Result<(), io::Error>);
}




pub struct StandardHashesSink <'a, 'b, Stream : io::Write> {
	stream : io::BufWriter<&'a mut Stream>,
	prefix : Cow<'b, [u8]>,
	infix : Cow<'b, [u8]>,
	suffix : Cow<'b, [u8]>,
}


impl <'a, Stream : io::Write> StandardHashesSink<'a, 'static, Stream> {
	
	pub fn new (_stream : &'a mut Stream, _zero : bool) -> (Self) {
		let _stream = io::BufWriter::with_capacity (128 * 1024, _stream);
		let _sink = StandardHashesSink {
				stream : _stream,
				prefix : Cow::Borrowed (b""),
				infix : Cow::Borrowed (b" *"),
				suffix : Cow::Borrowed (if _zero { b"\0" } else { b"\n" }),
			};
		return _sink;
	}
}


impl <Stream : io::Write> HashesSink for StandardHashesSink<'_, '_, Stream> {
	
	fn handle (&mut self, _path : & PathValueRef, _hash : & HashBytesRef) -> (Result<(), io::Error>) {
		self.stream.write_all (&self.prefix) ?;
		for _byte in _hash {
			self.stream.write_fmt (format_args! ("{:02x}", _byte)) ?;
		}
		self.stream.write_all (&self.infix) ?;
		self.stream.write_all (_path.as_bytes ()) ?;
		self.stream.write_all (&self.suffix) ?;
		return Ok (());
	}
}

