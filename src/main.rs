use anyhow::Result;
use rstrace::hdr_image::hdr_to_ldr;
use rstrace::pfm_func::Parameter;
use rstrace::pfm_func::read_pfm;
use std::fs::File;
use std::io::BufReader;
use clap::{Parser, Subcommand};

/*=============================================================================
PROGRAMMER NOTES:
The `demo` command:
1. Initialize a World object with the 10 spheres in the indicated positions
2. Create an OrthogonalCamera or PerspectiveCamera object
3. Rotate the observer
4. Create an ImageTracer object
5. Perform image tracing, using an “on/off” criterion
6. Save the PFM image
7. Immediately convert the image to PNG using default values for tone-mapping
 =============================================================================*/
#[derive(Parser)]

struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
#[derive(Clone)]
enum Commands {
    Demo {
        file: String
    },

    Pfm2Png {
        args: Vec<String>
    },
}

fn main() -> Result<()> {
    // Leave two lines between the execution and the printing of the
    println! {"\n------------------------------------------------------\n"};

    let cli = Cli::parse();

    match cli.command {
        Commands::Demo {file} => {
            return Ok(())
        }

        Commands::Pfm2Png {args} => {

            let mut params = Parameter::new(args)?;

            let file = File::open(&params.input_pfm_file_name);
            let mut reader: BufReader<File> = BufReader::new(file?);

            let mut img = read_pfm(&mut reader)?;

            img.normalization(Some(&params.factor_a))?;

            img.sem_clamp_image()?;

            hdr_to_ldr(&mut params)?;
            return Ok(())
        }
    }

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
