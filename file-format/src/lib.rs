#![no_std]

pub mod error;
pub use error::*;

mod bincode;
use bincode::*;

extern crate alloc;

use alloc::vec::Vec;
use alloc::{collections::btree_map::BTreeMap, vec};
use alloc::string::String;
// use bincode::{Decode, Encode};
use embedded_io::{Read, Seek, SeekFrom, ErrorType, Write};

// 0  .. 20    signature
// 20 .. 84    table address
// 84 .. ?     files
// ?  .. EOF   table

pub const SIGNATURE: &'static str = "Pocket Knife Archive";

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FileTable(pub BTreeMap<String, Entry>);

#[derive(Debug, PartialEq, Eq, Copy, Clone, Decode, Encode)]
pub struct Entry {
    pub offset: u64,
    pub length: u64,
}

pub trait Archivable<T: ErrorType + ?Sized> {
    fn filename(&self) -> Result<String, T::Error>;
    fn write_into(&self, archive: &mut T) -> Result<u64, T::Error>;
}

impl FileTable {
    pub fn create<A: Write + Seek, I: Archivable<A>>(
        archive: &mut A,
        input_files: &[I],
    ) -> Result<FileTable, CreateError<A::Error>> {
        archive.write(SIGNATURE.as_bytes()).map_err(CreateError::SignatureWrite)?;

        // skip 64-bit table address for now, we don't know it yet
        archive.seek(SeekFrom::Current(8)).map_err(CreateError::SkipAddress)?;

        // write the input files, build the table
        let mut table = BTreeMap::new();
        for input_file in input_files.iter() {
            let filename = input_file.filename().map_err(|err| CreateError::InvalidFilename(err))?;
            if table.contains_key(&filename) {
                return Err(CreateError::DuplicateFilename(filename));
            }
            let offset = archive.stream_position().map_err(CreateError::GetOffset)?;
            let length = input_file.write_into(archive).map_err(|err| CreateError::AddFile(filename.clone(), err))?;
            table.insert(filename, Entry { offset, length });
        }

        // keep track of where the table is about to get written
        let table_address: u64 = archive.stream_position().map_err(CreateError::GetFileTableAddress)?;

        // write the table
        bincode::encode_into_writer(&table, BincodeAdapter(archive), BINCODE_CONFIG).map_err(CreateError::SerializeFileTable)?;

        // write table address back near start of file, after the signature
        archive.seek(SeekFrom::Start(SIGNATURE.as_bytes().len() as u64)).map_err(CreateError::SeekToFileTableAddress)?;
        archive.write(&table_address.to_le_bytes()).map_err(CreateError::WriteFileTableAddress)?;

        Ok(FileTable(table))
    }

    pub fn read<A: Read + Seek>(
        archive: &mut A
    ) -> Result<FileTable, ReadError<A::Error>> {
        // validate signature
        let mut signature = [0u8; SIGNATURE.as_bytes().len()];
        archive.read_exact(&mut signature).map_err(ReadError::NoSignature)?;
        if signature != *SIGNATURE.as_bytes() {
            return Err(ReadError::InvalidSignature(signature));
        }

        // read table address
        let table_address = {
            let mut table_address_bytes: [u8; 8] = [0; 8];
            archive.read_exact(&mut table_address_bytes).map_err(ReadError::ReadFileTableAddress)?;
            u64::from_le_bytes(table_address_bytes)
        };

        // read table
        archive.seek(SeekFrom::Start(table_address)).map_err(ReadError::SeekToFileTable)?;
        let table = bincode::decode_from_reader(BincodeAdapter(archive), BINCODE_CONFIG).map_err(ReadError::DeserializeFileTable)?;

        Ok(FileTable(table))
    }

    pub fn open_file<A: Read + Seek>(
        &self,
        archive: &mut A,
        filename: String,
    ) -> Result<Vec<u8>, OpenError<A::Error>> {
        let entry = self.0.get(&filename).ok_or(OpenError::NoSuchFile(filename.into()))?;
        let mut buffer = vec![0u8; entry.length as usize];
        archive.seek(SeekFrom::Start(entry.offset)).map_err(OpenError::SeekToStart)?;
        archive.read_exact(&mut buffer).map_err(OpenError::ReadFile)?;
        Ok(buffer)
    }
}
