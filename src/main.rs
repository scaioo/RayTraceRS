use rstrace::hdr_image::{hdr_to_ldr};
use rstrace::pfm_func::read_pfm_file;
use rstrace::pfm_func::{ Parameter};


fn main() {
    // Leave two lines between the execution and the printing of the
    println! {"\n------------------------------------------------------\n"};

    let args: Vec<String> = std::env::args().collect();
    let mut params = Parameter::new(args).expect("invalid parameters");

    let mut img = read_pfm_file(&mut params.input_pfm_file_name).expect("error reading input file");

    img.normalization(Some(params.factor_a))
        .expect("error during image normalization");

    img.sem_clamp_image().expect("error using sem_clamp_image");

    hdr_to_ldr(&img, &mut params).expect("error converting file");
}
