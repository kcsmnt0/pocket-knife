use pocket_knife_file_format::{Archivable, ReadError, CreateError, OpenError};

use std::{fs::File, path::Path, io::{Read, Write, Seek, SeekFrom}, fmt::Debug};

#[derive(Debug)]
pub struct Error(pub String);

#[derive(Debug)]
pub struct ArchiveFile(pub File);

#[derive(Debug)]
pub struct InputFile(pub Box<Path>);

impl Archivable<ArchiveFile> for InputFile {
    fn filename(&self) -> Result<String, Error> {
        self.0.file_name()
            .and_then(|filename| filename.to_str())
            .map(String::from)
            .ok_or(Error(format!("invalid filename {:?}", self.0)))
    }

    fn write_into(&self, archive: &mut ArchiveFile) -> Result<u64, Error> {
        let mut input_file = File::open(self.0.clone())?;
        Ok(std::io::copy(&mut input_file, &mut archive.0)?)
    }
}

impl embedded_io::Read for ArchiveFile {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        Ok(self.0.read(buf)?)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), embedded_io::ReadExactError<Error>> {
        self.0.read_exact(buf).map_err(|err| embedded_io::ReadExactError::Other(err.into()))
    }
}

impl embedded_io::Write for ArchiveFile {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        Ok(self.0.write(buf)?)
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), Error> {
        Ok(self.0.write_all(buf)?)
    }

    fn flush(&mut self) -> Result<(), Error> {
        Ok(self.0.flush()?)
    }
}

impl embedded_io::Seek for ArchiveFile {
    fn seek(&mut self, pos: embedded_io::SeekFrom) -> Result<u64, Error> {
        Ok(self.0.seek(SeekFrom::from(pos))?)
    }

    fn rewind(&mut self) -> Result<(), Error> {
        Ok(self.0.rewind()?)
    }

    fn stream_position(&mut self) -> Result<u64, Error> {
        Ok(self.0.stream_position()?)
    }
}

impl embedded_io::ErrorType for ArchiveFile {
    type Error = Error;
}

impl embedded_io::Error for Error {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error(format!("{:?}", err))
    }
}

impl From<embedded_io::ReadExactError<Error>> for Error {
    fn from(err: embedded_io::ReadExactError<Error>) -> Self {
        Error(format!("{:?}", err))
    }
}

impl <E: Debug> From<ReadError<E>> for Error where Error: From<E> {
    fn from(err: ReadError<E>) -> Self {
        Error(format!("{:?}", err))
    }
}

impl <E: Debug> From<CreateError<E>> for Error where Error: From<E> {
    fn from(err: CreateError<E>) -> Self {
        Error(format!("{:?}", err))
    }
}

impl <E: Debug> From<OpenError<E>> for Error where Error: From<E> {
    fn from(err: OpenError<E>) -> Self {
        Error(format!("{:?}", err))
    }
}
