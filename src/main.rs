use rstrace::hdr_image::{hdr_to_ldr};
use rstrace::pfm_func::read_pfm_file;
use rstrace::pfm_func::{ Parameter};
use anyhow::{Result};


fn main() -> Result<()> {
    // Leave two lines between the execution and the printing of the
    println! {"\n------------------------------------------------------\n"};

    let args: Vec<String> = std::env::args().collect();
    let mut params = Parameter::new(args)?;

    let mut img = read_pfm_file(&mut params.input_pfm_file_name)?;

    img.normalization(Some(params.factor_a))?;

    img.sem_clamp_image()?;

    hdr_to_ldr(&mut params).unwrap();

    Ok(())
}
