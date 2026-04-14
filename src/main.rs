use anyhow::Result;
use rstrace::hdr_image::hdr_to_ldr;
use rstrace::pfm_func::Parameter;
use rstrace::pfm_func::read_pfm;
use std::fs::File;
use std::io::BufReader;

fn main() -> Result<()> {
    // Leave two lines between the execution and the printing of the
    println! {"\n------------------------------------------------------\n"};

    let args: Vec<String> = std::env::args().collect();
    let mut params = Parameter::new(args)?;

    let file = File::open(&params.input_pfm_file_name);
    let mut reader: BufReader<File> = BufReader::new(file?);

    let mut img = read_pfm(&mut reader)?;

    img.normalization(Some(&params.factor_a))?;

    img.sem_clamp_image()?;

    hdr_to_ldr(&mut params)?;

    Ok(())
}
