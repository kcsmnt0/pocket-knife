pub use bincode::{Decode, Encode, decode_from_reader, encode_into_writer};

use alloc::format;
use bincode::{de::read::Reader, enc::write::Writer, error::{EncodeError, DecodeError}};
use embedded_io::{Read, Write};

pub const BINCODE_CONFIG: bincode::config::Configuration = bincode::config::standard();

pub struct BincodeAdapter<'a, A>(pub &'a mut A);

impl <'a, A: Read> Reader for BincodeAdapter<'a, A> {
    fn read(&mut self, bytes: &mut [u8]) -> Result<(), DecodeError> {
        self.0.read_exact(bytes).map_err(|err| DecodeError::OtherString(format!("{:?}", err)))
    }
}

impl <'a, A: Write> Writer for BincodeAdapter<'a, A> {
    fn write(&mut self, bytes: &[u8]) -> Result<(), EncodeError> {
        self.0.write_all(bytes).map_err(|err| EncodeError::OtherString(format!("{:?}", err)))
    }
}
