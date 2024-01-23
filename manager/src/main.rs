#![feature(ascii_char)]

mod io;
pub use io::*;

use pocket_knife_file_format::FileTable;

use std::io::Write;
use std::path::Path;
use std::{env, fs};

fn main() {
    let args: Vec<String> = env::args().collect();

    match args[1].as_str() {
        "pack" => pack(&args[2], &args[3..]).unwrap(),
        "info" => info(&args[2]).unwrap(),
        "unpack" => unpack(&args[2], &args[3..]).unwrap(),
        _ => panic!("unsupported command"),
    }
}

fn pack(archive_name: &String, input_path_strs: &[String]) -> Result<(), Error> {
    let mut archive = ArchiveFile(fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(archive_name).unwrap());

    let input_paths: Vec<_> = input_path_strs.iter().map(|input_path_str| {
        InputFile(Box::from(Path::new(input_path_str)))
    }).collect();

    let file_table = FileTable::create(&mut archive, input_paths.as_slice())?;

    println!("{:?}", file_table.0);

    Ok(())
}

fn info(archive_path: &String) -> Result<(), Error> {
    let mut archive = ArchiveFile(fs::OpenOptions::new()
        .read(true)
        .create_new(false)
        .open(archive_path)?
    );

    let file_table = FileTable::read(&mut archive)?;

    println!("{:?}", file_table.0);

    Ok(())
}

fn unpack(archive_path: &String, output_path_strs: &[String]) -> Result<(), Error> {
    let mut archive = ArchiveFile(fs::OpenOptions::new()
        .read(true)
        .create_new(false)
        .open(archive_path)?
    );

    let file_table = FileTable::read(&mut archive)?;

    for output_path_str in output_path_strs {
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output_path_str)?
            .write_all(&file_table.open_file(&mut archive, output_path_str.clone())?)?;
    }

    Ok(())
}
