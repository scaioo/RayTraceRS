use anyhow::Result;
use rstrace::hdr_image::hdr_to_ldr;
use rstrace::pfm_func::Parameter;
use rstrace::pfm_func::read_pfm;
use std::fs::File;
use std::io::BufReader;
use clap::{Parser, Subcommand};
use rstrace::geometry::Vector;
use rstrace::shapes::{Shape, Sphere};
use rstrace::transformations::{Scaling, Transformation, Translation};
use rstrace::world::World;
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



fn demo_world() -> World {
    let sphere_scaling = Scaling::new([0.1, 0.1, 0.1]);

    let flat_corners: [Vector; 4] = [
        Vector::new(0.5, 0.5, 0.0),
        Vector::new(0.5, -0.5, 0.0),
        Vector::new(-0.5, 0.5, 0.0),
        Vector::new(-0.5, -0.5, 0.0),
    ];

    let return_sphere = |vec: &Vector, z: f32| -> Sphere<Transformation> {
        let new_vec = *vec + Vector::new(0.0, 0.0, z);
        let transformation = Translation::new(new_vec) * sphere_scaling;
        Sphere::new(transformation)
    };

    let upper_spheres = flat_corners
        .iter()
        .map(|vec| return_sphere(vec, 0.5));

    let lower_spheres = flat_corners
        .iter()
        .map(|vec| return_sphere(vec, -0.5));

    let central_spheres = vec![
        Sphere::new(Translation::new(Vector::new(0.0, 0.0, -0.5)) * sphere_scaling),
        Sphere::new(Translation::new(Vector::new(0.0, 0.5, 0.0)) * sphere_scaling),
    ];

    let objects: Vec<Box<dyn Shape>> = central_spheres
        .into_iter()
        .chain(upper_spheres)
        .chain(lower_spheres)
        .map(|s| Box::new(s) as Box<dyn Shape>)
        .collect();

    World { objects }
}

fn main() -> Result<()> {
    // Leave two lines between the execution and the printing of the
    println! {"\n------------------------------------------------------\n"};

    let cli = Cli::parse();

    match cli.command {
        Commands::Demo {file} => {
            let world = demo_world();

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
}
