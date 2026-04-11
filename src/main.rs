use rstrace::pfm_func::read_pfm_file;

fn main() {
    // Leave two lines between the execution and the printing of the
    println! {"\n------------------------------------------------------\n"};

    let img = read_pfm_file("reference_be.pfm");

    match img {
        Ok(img) => {
            println!("r pixel 1: {}", img.pixels[0].r);
            println!("g pixel 1: {}", img.pixels[0].g);
            println!("b pixel 1: {}", img.pixels[0].b);

            println!("r pixel 2: {}", img.pixels[1].r);
            println!("g pixel 2: {}", img.pixels[1].g);
            println!("b pixel 2: {}", img.pixels[1].b);
        }

        Err(e) => {
            println!("{e}");
        }
    }
}
