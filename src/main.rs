mod symbol;
mod property;

use std::{fs, io};
use std::fs::File;
use std::io::{Cursor, Read};
use std::path::PathBuf;
use anyhow::anyhow;
use clap::Parser;
use mktemp::Temp;
use crate::symbol::KicadSymbolLib;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short = 'z', long = "zip", value_name = "INPUT ZIP FILE")]
    input_zip: PathBuf,

    #[arg(short = 'f', long = "footprint-dir", value_name = "PATH TO FOOTPRINT DIR")]
    footprint_dir: PathBuf,

    #[arg(short = 's', long = "symbol-lib", value_name = "PATH TO SYMBOL LIB")]
    symbol_lib: PathBuf

}

fn zip_file_to_bytes(path_buf: PathBuf) -> Result<Vec<u8>, io::Error> {
    let mut file = File::open(path_buf)?;
    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer)?;

    Ok(buffer)
}

fn main() -> Result<(), anyhow::Error>{
    let args = Args::parse();

    println!("Input zip file: {}", args.input_zip.display());
    println!("Footprint directory: {}", args.footprint_dir.display());
    println!("Symbol library: {}", args.symbol_lib.display());

    let temp_extraction_dir = Temp::new_dir()?;
    let input_zip_file_bytes = zip_file_to_bytes(args.input_zip)?;

    println!("Temp extraction dir: {:?}", temp_extraction_dir);

    zip_extract::extract(Cursor::new(input_zip_file_bytes), &PathBuf::from(temp_extraction_dir.as_path()), true)?;

    let entries = fs::read_dir(temp_extraction_dir.as_path())?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    println!("entries: {entries:?}");

    let footprint_files: Vec<_> = entries.iter().filter(|path| {path.extension() == Some("kicad_mod".as_ref())}).collect();
    let step_files: Vec<_> = entries.iter().filter(|path| {path.extension() == Some("step".as_ref())}).collect();
    let symbol_libs: Vec<_> = entries.iter().filter(|path| {path.extension() == Some("kicad_sym".as_ref())}).collect();

    println!("Copying {} footprint file(s) to {}", footprint_files.len(), args.footprint_dir.display());

    for file in footprint_files {
        let dest_file = args.footprint_dir.join(file.file_name().ok_or(anyhow!("File {file:?} has no filename"))?);
        println!("{file:?} -> {dest_file:?}");
        fs::copy(file, dest_file)?;
    }

    println!("Copying {} step file(s) to {}", step_files.len(), args.footprint_dir.display());

    for step_file in step_files {
        let dest_file = args.footprint_dir.join(step_file.file_name().ok_or(anyhow!("File {step_file:?} has no filename"))?);
        println!("{step_file:?} -> {dest_file:?}");
        fs::copy(step_file, dest_file)?;
    }
    
    for file in symbol_libs {
        let kicad_symbol_lib = KicadSymbolLib::from_file(File::open(file)?)?;
    }
    Ok(())
}
