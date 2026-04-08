use std::fs::File;
use std::io::{BufRead, BufReader};
use endianness::ByteOrder;

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
        .collect::<anyhow::Result<Vec<u8>, _>>();

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
//Result<ByteOrder, &str>
pub fn _parse_endianness(filename: &str) -> anyhow::Result<ByteOrder, EndiannessError> {
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
        Ok(ByteOrder::BigEndian)
    } else if endianness_number < 0.0 {
        Ok(ByteOrder::LittleEndian)
    } else {
        Err(EndiannessError::InvalidValue)
    }
}