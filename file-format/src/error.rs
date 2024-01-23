use alloc::string::String;
use bincode::error::{EncodeError, DecodeError};
use embedded_io::ReadExactError;

#[derive(Debug)]
pub enum ReadError<E> {
    NoSignature(ReadExactError<E>),
    InvalidSignature([u8; 20]),
    ReadFileTableAddress(ReadExactError<E>),
    SeekToFileTable(E),
    DeserializeFileTable(DecodeError),
}

#[derive(Debug)]
pub enum CreateError<E> {
    SignatureWrite(E),
    SkipAddress(E),
    InvalidFilename(E),
    DuplicateFilename(String),
    GetOffset(E),
    AddFile(String, E),
    GetFileTableAddress(E),
    SerializeFileTable(EncodeError),
    SeekToFileTableAddress(E),
    WriteFileTableAddress(E),
}

#[derive(Debug)]
pub enum OpenError<E> {
    NoSuchFile(String),
    SeekToStart(E),
    ReadFile(ReadExactError<E>),
}
