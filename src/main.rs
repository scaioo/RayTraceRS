use anyhow::Result;
use clap::{Parser};
use rstrace::camera::{Camera, OrthogonalCamera, PerspectiveCamera};
use rstrace::color::Color;
use rstrace::geometry::Point;
use rstrace::geometry::Vector;
use rstrace::hdr_image::HDR;
use rstrace::image_tracer::ImageTracer;
use rstrace::pfm_func::{pfm_to_png, Endianness};
use rstrace::ray::Ray;
use rstrace::shapes::{Shape, Sphere};
use rstrace::transformations::{Scaling, Transformation, Translation};
use rstrace::world::World;
use std::fs::File;
use std::io::BufWriter;
use std::time::Instant;
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
    #[arg(long, default_value_t = 1000)]
    width: usize,

    #[arg(long, default_value_t = 700)]
    height: usize,

    #[arg(long)]
    orthogonal: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser, Clone)]
enum Commands {
    Demo { file_name: String },

    Pfm2Png { input_file: String,
            output_file: String,
            factor_a: f32,
            gamma: f32,
        },
    
    Debug { file_name: String,
        output_file: String,
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

    let upper_spheres = flat_corners.iter().map(|vec| return_sphere(vec, 0.5));

    let lower_spheres = flat_corners.iter().map(|vec| return_sphere(vec, -0.5));

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
    let now = Instant::now();
    // Leave two lines between the execution and the printing of the
    println! {"\n------------------------------------------------------\n"};

    let cli = Cli::parse();

    match cli.command {
        Commands::Demo { file_name } => {
            //let origin: Point = Point::new(-2.0, 0.0, 0.0);
            //let screen_center: Point = Point::new(-1.0, 0.0, 0.0);
            let mat = Vector::new(-2.0, 0.0, 0.0);
            let transl = Translation::new(mat);
            let world = demo_world();

            if cli.orthogonal {
                let mut o_cam = OrthogonalCamera::new(transl);
                let img = HDR::new(cli.width, cli.height);
                let aspectratio = (img.width as f32 / img.height as f32) ;
                o_cam.set_aspect_ratio(aspectratio);
                let mut imagetracer = ImageTracer::new(img, o_cam);
                imagetracer
                    .fire_all_rays(&world, color_image)
                    .expect("error firing all rays");
                println!("all done orthogonal!");
                let filename = "files/".to_string() + &file_name;
                let file = File::create(&filename)?;
                let disk_writer = BufWriter::new(&file);
                imagetracer.image.write_pfm(disk_writer, &Endianness::BigEndian).expect("error creating pfm file ");
                pfm_to_png(file_name, 0.18, 2.2, "files/first_image.png".to_string()).expect("error converting file from pfm");

            } else {
                let mut p_cam = PerspectiveCamera::new(transl);
                let img = HDR::new(cli.width, cli.height);

                let aspectratio = (img.width as f32 / img.height as f32) ;
                p_cam.set_aspect_ratio(aspectratio);
                let mut imagetracer = ImageTracer::new(img, p_cam);
                imagetracer
                    .fire_all_rays(
                        &world,
                        color_image
                    )
                    .expect("error firing all rays");
                println!("all done!");
                let filename = "files/".to_string() + &file_name;
                let file = File::create(&filename)?;
                let disk_writer = BufWriter::new(&file);
                imagetracer.image.write_pfm(disk_writer, &Endianness::BigEndian).expect("error creating pfm file ");
                pfm_to_png(filename, 0.18, 2.2, "files/second_image.png".to_string()).expect("error converting file from pfm");

            }

            // create a file


            let duration = now.elapsed();
            println!("Program finished in {:?}", duration);
            return Ok(());
        }

        Commands::Pfm2Png { input_file, output_file, factor_a, gamma } => {
            pfm_to_png(input_file, factor_a, gamma, output_file).expect("error converting file from pfm");
            
            let duration = now.elapsed();
            println!("Program finished in {:?}", duration);
            return Ok(());
        }
        
        Commands::Debug { file_name, output_file } => {
            let mat = Vector::new(-2.0, 0.0, 0.0);
            let transl = Translation::new(mat);
            let world = demo_world();

            let o_cam = OrthogonalCamera::new(transl);
            let img = HDR::new(cli.width, cli.height);
            let mut imagetracer = ImageTracer::new(img, o_cam);
            imagetracer.fire_all_rays(&world, color_image)?;
            println!("All fired!");

            let filename = "files/".to_string() + &file_name;
            let file = File::create(&filename)?;
            let disk_writer = BufWriter::new(&file);
            imagetracer.image.write_pfm(disk_writer, &Endianness::BigEndian).expect("error creating pfm file ");
            let output_filename = "files/".to_string() + &output_file;
            pfm_to_png(filename, 0.18, 2.2, output_filename).expect("error converting file from pfm");

            Ok(())
        }
    }
}

fn color_image(ray: Ray, world: &World) -> Result<Color> {
    let inters = world.ray_intersection(ray);
    match inters {
        Some(_x) => {
            let color = Color::new(1.0, 1.0, 1.0);
            Ok(color)
        }
        None => {
            let color = Color::new(0.0, 0.0, 0.0);
            Ok(color)
        }
    }
}
