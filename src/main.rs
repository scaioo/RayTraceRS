use rstrace::color;
use rstrace::hdr_image::HDR;
use rstrace::pfm_func::{_parse_img_size, read_pfm_file};
use rstrace::pfm_func::_parse_endianness;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufRead;
use rstrace::hdr_image;

fn main() {
    // Leave two lines between the execution and the printing of the main
    println! {"\n------------------------------------------------------\n"};

    let _hdr_img = read_pfm_file("reference_le.pfm").unwrap();

}
