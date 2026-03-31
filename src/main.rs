use rstrace::color;
use rstrace::hdr_image::HDR;
use rstrace::hdr_image::_parse_img_size;
use rstrace::hdr_image::_parse_endianness;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufRead;
use endianness::ByteOrder;
use endianness::ByteOrder::{BigEndian, LittleEndian};
use rstrace::hdr_image;

fn main() {
    // Leave two lines between the execution and the printing of the main
    println! {"\n------------------------------------------------------\n"};



    // prove files pfm

    ////// vorrei fare in modo che l'utente possa inserire il nome del file da aprire
    // ma quando a mano metto reference_le.pfm non gli piace (mi dice che non trova il file
    //let mut file_name: String = String::new();
    //std::io::stdin()
    //   .read_line(&mut file_name).unwrap().to_string().trim();
    //println!("{file_name}");
    //let file = File::open(file_name);

    //// checks if file is pfm or not
    //// tests are yet to be written and checks are to be made on error handling etc

    let file = File::open("reference_le.pfm");
    let mut reader = io::BufReader::new(file.unwrap());
    let mut line: String = String::new();

    let mut img_dim = _parse_img_size("reference_le.pfm");


    // non sono sicura di quello che la funzione restituisce (in particolare non so bene returnare gli errori
    // controllare !!!!!!

    let _ = _parse_endianness("reference_le.pfm");
    let _ = _parse_endianness("reference_be.pfm");

    // reads endianness
    // line.clear();
    // let mut line: String = String::new();
    // reader.read_line(&mut line).unwrap();
    // let mut endianness; ////// i think there is going to be something wrong with lifetimes
    // if line.len() == 1 {
    // println!("there is something wrong with the length of endianness line: its length is {}", line.len());
    // } else {
    // let endianness_val: f32 = line.parse::<f32>().unwrap();
    // println! {"Endianness: {:?}", endianness_val};
    // if endianness_val > 0.0 {
    // endianness = BigEndian;
    // println!("it's a big endian!");
    // } else if endianness_val < 0.0 {
    // endianness = LittleEndian;
    // println!("it's a little endian!");
    // } else {
    // panic!("endianness is not a valid floating point number: {}", endianness_val); }
    //}


    ///// expressions vs statements rust ?????



}
