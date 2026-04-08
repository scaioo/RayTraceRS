use std::fs::File;
use std::io::{Read, BufRead, BufReader};
use endianness::ByteOrder;
use anyhow::{anyhow, Result};
use byteorder::{ReadBytesExt};

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

pub fn _read_4bytes<R: Read>(endianness: ByteOrder, buf : &mut R ) -> Result<f32> {
    match endianness {
        ByteOrder::LittleEndian =>
            {buf.read_f32::<byteorder::LittleEndian>()
            .map_err(|e| anyhow!(e))},
        ByteOrder::BigEndian =>
            {buf.read_f32::<byteorder::BigEndian>()
            .map_err(|e| anyhow!(e))},
    }
}



#[cfg(test)]
mod test {

    const BE_ARRAY: &[u8] = &[
        0x50, 0x46, 0x0a, 0x33, 0x20, 0x32, 0x0a, 0x31, 0x2e, 0x30, 0x0a, 0x42,
        0xc8, 0x00, 0x00, 0x43, 0x48, 0x00, 0x00, 0x43, 0x96, 0x00, 0x00, 0x43,
        0xc8, 0x00, 0x00, 0x43, 0xfa, 0x00, 0x00, 0x44, 0x16, 0x00, 0x00, 0x44,
        0x2f, 0x00, 0x00, 0x44, 0x48, 0x00, 0x00, 0x44, 0x61, 0x00, 0x00, 0x41,
        0x20, 0x00, 0x00, 0x41, 0xa0, 0x00, 0x00, 0x41, 0xf0, 0x00, 0x00, 0x42,
        0x20, 0x00, 0x00, 0x42, 0x48, 0x00, 0x00, 0x42, 0x70, 0x00, 0x00, 0x42,
        0x8c, 0x00, 0x00, 0x42, 0xa0, 0x00, 0x00, 0x42, 0xb4, 0x00, 0x00
    ];

    const LE_ARRAY: &[u8] = &[
        0x50, 0x46, 0x0a, 0x33, 0x20, 0x32, 0x0a, 0x2d, 0x31, 0x2e, 0x30, 0x0a,
        0x00, 0x00, 0xc8, 0x42, 0x00, 0x00, 0x48, 0x43, 0x00, 0x00, 0x96, 0x43,
        0x00, 0x00, 0xc8, 0x43, 0x00, 0x00, 0xfa, 0x43, 0x00, 0x00, 0x16, 0x44,
        0x00, 0x00, 0x2f, 0x44, 0x00, 0x00, 0x48, 0x44, 0x00, 0x00, 0x61, 0x44,
        0x00, 0x00, 0x20, 0x41, 0x00, 0x00, 0xa0, 0x41, 0x00, 0x00, 0xf0, 0x41,
        0x00, 0x00, 0x20, 0x42, 0x00, 0x00, 0x48, 0x42, 0x00, 0x00, 0x70, 0x42,
        0x00, 0x00, 0x8c, 0x42, 0x00, 0x00, 0xa0, 0x42, 0x00, 0x00, 0xb4, 0x42
    ];

    use super::*;
    use std::io;
    use crate::functions::are_close;

    // Test for read_4bytes()
    #[test]
    #[should_panic(expected = "no more floats!")]
    fn test_read_4bytes_le() {
        let mut rdr = io::Cursor::new(LE_ARRAY);
        for _ in 0..3 {
            let mut line = String::new();
            let _ =rdr.read_line(& mut line).unwrap();
        }

        for i in 0..9 {
            let val = _read_4bytes(ByteOrder::LittleEndian, &mut rdr).unwrap();
            let expected = ((i + 1) * 100) as f32;
            assert!(are_close(val, expected));
        }
        for i in 0..9 {
            let val = _read_4bytes(ByteOrder::LittleEndian, &mut rdr).unwrap();
            let expected = ((i + 1) * 10) as f32;
            assert!(are_close(val, expected));
        }
        _read_4bytes(ByteOrder::LittleEndian,&mut rdr).expect("no more floats!");
    }

    #[test]
    #[should_panic(expected = "no more floats!")]
    fn test_read_4bytes_be() {
        let mut rdr = io::Cursor::new(BE_ARRAY);
        for _ in 0..3 {
            let mut line = String::new();
            let _ =rdr.read_line(& mut line).unwrap();
        }

        for i in 0..9 {
            let val = _read_4bytes(ByteOrder::BigEndian, &mut rdr).unwrap();
            let expected = ((i + 1) * 100) as f32;
            assert!(are_close(val, expected));
        }
        for i in 0..9 {
            let val = _read_4bytes(ByteOrder::BigEndian, &mut rdr).unwrap();
            let expected = ((i + 1) * 10) as f32;
            assert!(are_close(val, expected));
        }
        _read_4bytes(ByteOrder::BigEndian,&mut rdr).expect("no more floats!");
    }
}