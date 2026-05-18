use anyhow::Result;
use clap::Parser;
use rstrace::brdf::DiffusiveBrdf;
use rstrace::camera::{Camera, OrthogonalCamera, PerspectiveCamera};
use rstrace::color::Color;
use rstrace::geometry::Vector;
use rstrace::hdr_image::HDR;
use rstrace::image_tracer::ImageTracer;
use rstrace::materials::Material;
use rstrace::pfm_func::{Endianness, pfm_to_ldr};
use rstrace::pigments::UniformPigment;
use rstrace::ray::Ray;
use rstrace::shapes::{Shape, Sphere};
use rstrace::transformations::{Scaling, Transformation, Translation};
use rstrace::world::World;
use std::fs::File;
use std::io::BufWriter;
use std::time::Instant;

#[derive(Parser)]

struct Cli {
    #[arg(long, default_value_t = 1000)]
    width: usize,

    #[arg(long, default_value_t = 750)]
    height: usize,

    #[arg(long)]
    orthogonal: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser, Clone)]
enum Commands {
    Demo {
        file_name: String,
    },

    Pfm2Png {
        input_file: String,
        output_file: String,
        factor_a: f32,
        gamma: f32,
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
        Sphere::new(
            transformation,
            Material {
                pigment: Box::new(UniformPigment::new(Color::new(10.0, 10.0, 10.0))),
                brdf: Box::new(DiffusiveBrdf {}),
            },
        )
    };

    let upper_spheres = flat_corners.iter().map(|vec| return_sphere(vec, 0.5));

    let lower_spheres = flat_corners.iter().map(|vec| return_sphere(vec, -0.5));

    let central_spheres = vec![
        Sphere::new(
            Translation::new(Vector::new(0.0, 0.0, -0.5)) * sphere_scaling,
            Material {
                pigment: Box::new(UniformPigment::new(Color::new(10.0, 10.0, 10.0))),
                brdf: Box::new(DiffusiveBrdf {}),
            },
        ),
        Sphere::new(
            Translation::new(Vector::new(0.0, 0.5, 0.0)) * sphere_scaling,
            Material {
                pigment: Box::new(UniformPigment::new(Color::new(10.0, 10.0, 10.0))),
                brdf: Box::new(DiffusiveBrdf {}),
            },
        ),
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
    println! {"\n------------------------------------------------------\n"};

    let cli = Cli::parse();

    match cli.command {
        Commands::Demo { file_name } => {
            let mat = Vector::new(-2.0, 0.0, 0.0);
            let transl = Translation::new(mat);
            let world = demo_world();

            if cli.orthogonal {
                let mut o_cam = OrthogonalCamera::new(transl);
                let img = HDR::new(cli.width, cli.height);
                let aspectratio = img.width as f32 / img.height as f32;
                o_cam.set_aspect_ratio(aspectratio)?;
                let mut imagetracer = ImageTracer::new(img, o_cam);
                imagetracer
                    .fire_all_rays(&world, color_image)
                    .expect("error firing all rays");
                println!("all done orthogonal!");
                let filename = "outputs/".to_string() + &file_name;
                let file = File::create(&filename)?;
                let disk_writer = BufWriter::new(&file);
                imagetracer
                    .image
                    .write_pfm(disk_writer, &Endianness::BigEndian)
                    .expect("error creating pfm file ");
                pfm_to_ldr(file_name, 0.18, 2.2, "outputs/first_image.png".to_string())
                    .expect("error converting file from pfm");
            } else {
                let mut p_cam = PerspectiveCamera::new(transl);
                let img = HDR::new(cli.width, cli.height);

                let aspectratio = img.width as f32 / img.height as f32;
                p_cam.set_aspect_ratio(aspectratio)?;
                let mut imagetracer = ImageTracer::new(img, p_cam);
                imagetracer
                    .fire_all_rays(&world, color_image)
                    .expect("error firing all rays");
                println!("all done!");
                let filename = "outputs/".to_string() + &file_name;
                let file = File::create(&filename)?;
                let disk_writer = BufWriter::new(&file);
                imagetracer
                    .image
                    .write_pfm(disk_writer, &Endianness::BigEndian)
                    .expect("error creating pfm file ");
                pfm_to_ldr(filename, 0.18, 2.2, "outputs/second_image.png".to_string())
                    .expect("error converting file from pfm");
            }

            // create a file

            let duration = now.elapsed();
            println!("Program finished in {:?}", duration);
            Ok(())
        }

        Commands::Pfm2Png {
            input_file,
            output_file,
            factor_a,
            gamma,
        } => {
            pfm_to_ldr(input_file, factor_a, gamma, output_file)
                .expect("error converting file from pfm");

            let duration = now.elapsed();
            println!("Program finished in {:?}", duration);
            Ok(())
        }
    }
}

fn color_image(ray: Ray, world: &World) -> Result<Color> {
    let inters = world.ray_intersection(&ray);
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
