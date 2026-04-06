use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use crate::color::Color;
use crate::hdr_image::HDR;
use anyhow::anyhow;

pub enum Endianness {
    LittleEndian,
    BigEndian
}
pub enum EndiannessError{
    InvalidValue
}

// reading and writing pfm files

//_read_magic reads a line from a buffer and returns an error if the line is not PF or Pf
pub fn _read_magic(reader :&mut BufReader<File>) -> anyhow::Result<()> {
    let mut line :String = String::new();
    reader.read_line(&mut line)?;
    line = line.trim().to_string();
    if line != "PF" && line != "Pf" {
       return Err(anyhow!("magic is not PF nor Pf! file is not PFM"))
    }
    Ok(())
}

// _parse_img_size takes a BufReader as an input and returns 2 usize values:
pub fn _parse_img_size( reader: &mut BufReader<File>) -> anyhow::Result<(usize, usize)> {

    let mut line: String = String::new();

    //// checks the dimension of the image

    reader.read_line(&mut line)?;

    // turns the strings (created by split_whitespace into numbers (cols and rows)
    // map takes all the items created by split_whitespace (the dimensions of the image)
    // and parse turns them from string into usize
    let  hdr_size :Vec<usize> = line.split_whitespace()
        .map(|x| x.parse::<usize>())
        .collect::<Result<_, _>>()?;

    if hdr_size.len() != 2 {
       return Err(anyhow!("incorrect image size, _parse_img_size returns {} values", hdr_size.len()));
    }
    Ok((hdr_size[0], hdr_size[1]))
}


// _parse_endianness returns a result type containing and enum Endianness as defined above
pub fn _parse_endianness(reader: & mut BufReader<File>) -> anyhow::Result<Endianness> {

    let mut line: String = String::new();


    reader.read_line(&mut line)?;
    let endianness_number: f32 = line.trim().parse::<f32>()?;

    println!("{}", line.trim());

    if endianness_number > 0.0 {
        Ok(Endianness::BigEndian)
    } else if endianness_number < 0.0 {
        Ok(Endianness::LittleEndian)
    } else {
        Err(anyhow::anyhow!("EndiannessError::InvalidValue"))
    }
}

// _read_hdr creates an HDR and returns it with colors assigned to each pixel read from a buffer
// it supports both big and little endian. endianness is a parameter that needs to be passed from the calling function
fn _read_hdr(reader :&mut BufReader<File>, width :usize, height :usize, endianness :Endianness) -> anyhow::Result<HDR> {
    let mut hdr_img :HDR = HDR::new(width, height);
    let mut buffer = [0; 4];

    //bytes to f32 is a closure that avoids code repetition, it takes an array of four bytes and,
    //matching the endianness, it turns it into an f32
    let bytes_to_f32 = |buf: [u8; 4]| match endianness {
        Endianness::LittleEndian => f32::from_le_bytes(buf),
        Endianness::BigEndian => f32::from_be_bytes(buf),
    };

    for i in 0..height {
        for j in 0..width {
            reader.read_exact(&mut buffer)?;
            let r = bytes_to_f32(buffer);
            reader.read_exact(&mut buffer)?;
            let g = bytes_to_f32(buffer);
            reader.read_exact(&mut buffer)?;
            let b = bytes_to_f32(buffer);
            hdr_img.pixels[width*i + j] = Color::new(r, g, b);
        }

    }
     Ok(hdr_img)
}

// read_pfm_image uses the functions defined above to read all the necessary information from a pfm file
// and returns an HDR type containing the datas in the pfm.
// row-major order is used to read pixels

pub fn read_pfm_image(filename: &str) -> anyhow::Result<HDR, anyhow::Error> {
    let file = File::open(filename);
    let mut reader = BufReader::new(file?);
    let mut line: String = String::new();
    _read_magic(&mut reader)?;
    let (width, height) = _parse_img_size(&mut reader)?;

    println!("Pf image size: {}x{}", width, height);

    let endianness = _parse_endianness(&mut reader);

    match endianness {
        Ok(Endianness::LittleEndian) => {
            println!("/n little endian /n");
        }
        Ok(Endianness::BigEndian) => {
            println!("big endian");
        }
        Err(e) => {
            return Err(anyhow!(e));
        }
    }

    let hdr_img = _read_hdr(& mut reader, width, height, endianness?)?;
    Ok(hdr_img)
}