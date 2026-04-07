use std::fs::File;
use std::io::{BufRead, BufReader};
use byteorder::{BigEndian,LittleEndian, ReadBytesExt};
use anyhow::{Result, anyhow};
#[derive(Debug)]
pub enum EndiannessError{
    InvalidValue
}

// reading and writing pfm files

//read_line already exists in Rust's standard library

pub fn _parse_img_size(filename: &str) -> anyhow::Result<Vec<u8>, anyhow::Error> {

    let file = File::open(filename);
    let mut reader = BufReader::new(file?);
    let mut line: String = String::new();

    reader.read_line(&mut line)?;
    line = line.trim().to_string();

    if line != "PF" && line != "Pf" {
        println!("NON è PFM RENDILO UN ERRORE");
        println!("{}", line);
    }

    //// checks the dimension of the image
    line.clear();

    reader.read_line(&mut line)?;

    // turns the strings (created by split_whitespace into numbers (cols and rows)
    let   line_u8 = line.split_whitespace()
        .map(|x| x.parse::<u8>())
        .collect::<Result<Vec<u8>, _>>();

    match line_u8 {
        Ok(line) => {
            return Ok(line);
        }
        Err(e) => {
            println!("{:?}", e);
            return Err(anyhow::anyhow!("something's wrong with image dimensions declared in pfm"));
        }
    }
}

// read endianness takes the name of a file as an input
// Result<ByteOrder, &str>
// Questo codice è tutto da rivedere!!!!!!
pub enum BO{
    LittleEndian,
    BigEndian,
}
pub fn _parse_endianness(filename: &str) -> Result<BO, EndiannessError> {
    let file = File::open(filename);
    let mut reader = BufReader::new(file.unwrap());
    let mut line: String = String::new();

    // reads PF line (read_line reads the lines in order,
    // to read the third i need to read the other two first
    reader.read_line(&mut line).unwrap();

    // reads line cols rows
    line.clear();
    reader.read_line(&mut line).unwrap();

    //reads endianness
    line.clear();
    reader.read_line(&mut line).unwrap();
    let endianness_number: f32 = line.trim().parse().unwrap();

    println!("{}", line.trim());

    if endianness_number > 0.0 {
        Ok(BO::LittleEndian)
    } else if endianness_number < 0.0 {
        Ok(BO::BigEndian)
    } else {
        Err(EndiannessError::InvalidValue)
    }
}

// Crate used: as-bytes, byteorder
// https://crates.io/crates/as-bytes
// https://docs.rs/byteorder/latest/byteorder/trait.ByteOrder.html


pub fn _read_float<R: BufRead>(mut reader: R, endianness : BO) -> Result<f32> {
    Ok(3.0)
}