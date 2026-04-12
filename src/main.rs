use rstrace::color;
use rstrace::hdr_image;
use rstrace::hdr_image::HDR;
use rstrace::pfm_func::_parse_endianness;
use rstrace::pfm_func::read_pfm_file;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufRead;

fn main() {
    // Leave two lines between the execution and the printing of the
    println! {"\n------------------------------------------------------\n"};

  /*  let boh = Parameter::parse_command_line();

    match boh {
        Ok(boh) => {
            println!("{}", boh.factor_a);
        }
        Err(E) => (),
    }*/
    let mut par = Parameter::new(String::from("ahia"), 1.0, 1.0, String::from("arrive"));
    pfm_func::hdr_to_ldr(&mut par);
    println!("all done");
}
