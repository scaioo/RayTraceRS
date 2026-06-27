use anyhow::{Result, anyhow};
use clap::Parser;
use rstrace::brdf::{DiffusiveBrdf, SpecularBrdf};
use rstrace::camera::{Camera, OrthogonalCamera, PerspectiveCamera};
use rstrace::color::{BLACK, Color};
use rstrace::geometry::Vector;
use rstrace::hdr_image::HDR;
use rstrace::image_tracer::ImageTracer;
use rstrace::materials::Material;
use rstrace::pcg::PCG;
use rstrace::pfm_func::{Endianness, pfm_to_ldr};
use rstrace::pigments::{CheckeredPigment, UniformPigment};
use rstrace::ray::Ray;
use rstrace::renderer::{FlatRenderer, OnOffRenderer, PathTracer, PointLightRenderer, Renderer};
use rstrace::shapes::{Plane, Shape, Sphere};
use rstrace::transformations::{Scaling, Transformation, Translation, ZRotation};
use rstrace::world::World;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::time::Instant;

#[derive(Parser)]
struct Cli {
    #[arg(long, default_value_t = 1000)]
    width: usize,

    #[arg(long, default_value_t = 750)]
    height: usize,

    #[arg(long)]
    orthogonal: bool,

    #[arg(long, default_value = "png")]
    format: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser, Clone)]
enum Commands {
    Demo {
        file_name: String,

        #[arg(long, default_value_t = 5)]
        antialiasing: usize,

        #[arg(long, default_value_t = 0.0)]
        angle_deg: f32,

        #[arg(long, default_value = "pathtracing")]
        algorithm: String,

        #[arg(long, default_value_t = 5)]
        num_of_rays: usize,

        #[arg(long, default_value_t = 3)]
        max_depth: usize,
    },

    Pfm2Png {
        input_file: String,
        output_file: String,
        factor_a: f32,
        gamma: f32,
    },

    Render {
        input_scene_name: String,

        #[arg(long, default_value = "pathtracing")]
        algorithm: String,

        #[arg(long, default_value = "output.pfm")]
        pfm_output: String,

        #[arg(long, default_value = "output.png")]
        png_output: String,

        #[arg(long, default_value_t = 10)]
        num_of_rays: usize,

        #[arg(long, default_value_t = 3)]
        max_depth: usize,

        #[arg(long, default_value_t = 45)]
        init_state: u64,

        #[arg(long, default_value_t = 54)]
        init_seq: u64,

        #[arg(long, default_value_t = 1)]
        samples_per_pixel: usize,

        /// Declare a variable. Syntax: VAR:VALUE. Example: --declare-float=clock:150
        #[arg(short = 'd', long = "declare-float")]
        declare_float: Vec<String>,
    },
}

// ====================================================================
// CONSTRUCTION OF THE SCENE
// ====================================================================
fn demo_world() -> World {
    let mut objects: Vec<Box<dyn Shape>> = Vec::new();

    // 1. THE SKY (A giant sphere)
    let sky_material = Material {
        // FlatRenderer uses pigment, we give the color of the sky
        pigment: Box::new(UniformPigment::new(Color::new(0.5, 0.9, 1.0))),
        brdf: Box::new(DiffusiveBrdf {}),
        emitted_radiance: Box::new(UniformPigment::new(Color::new(0.5, 0.9, 1.0))),
    };
    let sky_transform =
        Scaling::from(200.0) * Translation::new(Vector::new(0.0, 0.0, 0.4));
    objects.push(Box::new(Sphere::new(sky_transform, sky_material)));

    // 2. THE FLOOR (An infinite chequered floor)
    let ground_material = Material {
        // Let’s create large squares by setting a low step size (e.g. 10, or depending on the scale)
        pigment: Box::new(CheckeredPigment::new(
            Color::new(0.3, 0.5, 0.1),
            Color::new(0.1, 0.2, 0.5),
            5,
        )),
        brdf: Box::new(DiffusiveBrdf {}),
        emitted_radiance: Box::new(UniformPigment::new(BLACK)),
    };
    let ground_transform = Transformation::new(rstrace::functions::IDENTITY_4X4);
    objects.push(Box::new(Plane::new(
        ground_transform,
        ground_material,
        true,
    )));

    // 3. DIFFUSE SPHERE
    let sphere_material = Material {
        pigment: Box::new(UniformPigment::new(Color::new(0.3, 0.4, 0.8))),
        brdf: Box::new(DiffusiveBrdf {}),
        emitted_radiance: Box::new(UniformPigment::new(BLACK)),
    };
    let s1_transform = Translation::new(Vector::new(0.0, 0.0, 1.0));
    objects.push(Box::new(Sphere::new(s1_transform, sphere_material)));

    // 4. MIRROR SPHERE
    let mirror_material = Material {
        pigment: Box::new(UniformPigment::new(Color::new(0.6, 0.2, 0.3))),
        brdf: Box::new(SpecularBrdf {}),
        emitted_radiance: Box::new(UniformPigment::new(BLACK)),
    };
    let s2_transform = Translation::new(Vector::new(1.0, 2.5, 0.0));
    objects.push(Box::new(Sphere::new(s2_transform, mirror_material)));

    World {
        objects,
        light_sources: vec![],
    }
}

/// Helper function to parse variables from the CLI into a HashMap
fn build_variable_table(declare_float: &[String]) -> HashMap<String, f32> {
    let mut variables = HashMap::new();
    for decl in declare_float {
        let parts: Vec<&str> = decl.split(':').collect();
        if parts.len() == 2 {
            if let Ok(val) = parts[1].parse::<f32>() {
                variables.insert(parts[0].to_string(), val);
            } else {
                eprintln!("Warning: Could not parse value for variable '{}'", parts[0]);
            }
        } else {
            eprintln!(
                "Warning: Invalid variable declaration format: '{}'. Use VAR:VALUE",
                decl
            );
        }
    }
    variables
}
// ====================================================================
// MAIN PROGRAM
// ====================================================================
fn main() -> Result<()> {
    let now = Instant::now();
    println!("\n------------------------------------------------------\n");

    let cli = Cli::parse();

    if cli.format != "png" && cli.format != "jpeg" && cli.format != "jpg" {
        panic!(
            "invalid extension for --format \n try \tpng \n\tjpg \n\tjpeg \nextension is automatically set to png"
        )
    }

    match cli.command {
        Commands::Demo {
            file_name,
            antialiasing,
            angle_deg,
            algorithm,
            num_of_rays,
            max_depth,
        } => {
            println!(
                "Generating a {}x{} image, with the camera tilted by {}°",
                cli.width, cli.height, angle_deg
            );

            let world = demo_world();
            let mut img = HDR::new(cli.width, cli.height);
            let aspectratio = img.width as f32 / img.height as f32;

            // Let's apply the camera transformation (Rotation around the Z-axis * Translation)
            let angle_rad = angle_deg.to_radians();
            let camera_tr =
                ZRotation::new(angle_rad) * Translation::new(Vector::new(-1.0, 0.0, 1.0));

            // Renderer and randomizer setup
            let flat_renderer = FlatRenderer::new(BLACK);
            let onoff_renderer = OnOffRenderer::default();
            let path_tracer = PathTracer::new(BLACK, num_of_rays, max_depth, 2);
            let whitted = PointLightRenderer {
                background_color: BLACK,
            };

            // Let’s initialize the random number generator
            let mut pcg = PCG::default();

            let render_closure = |ray: Ray, world: &World| -> Result<Color> {
                if algorithm == "onoff" {
                    onoff_renderer.render(&ray, world, &mut pcg)
                } else if algorithm == "flat" {
                    flat_renderer.render(&ray, world, &mut pcg)
                } else if algorithm == "pathtracing" {
                    path_tracer.render(&ray, world, &mut pcg)
                } else if algorithm == "point-light" {
                    whitted.render(&ray, world, &mut pcg)
                } else {
                    panic!("Unknown algorithm: {}", algorithm);
                }
            };

            // Let's perform the tracking based on the camera type
            if cli.orthogonal {
                let mut o_cam = OrthogonalCamera::new(camera_tr);
                o_cam.set_aspect_ratio(aspectratio)?;
                let mut imagetracer = ImageTracer::new(img, o_cam, antialiasing);

                println!("Rendering in progress...");
                imagetracer.fire_all_rays(&world, render_closure)?;
                img = imagetracer.image; // Retrieve the calculated image
            } else {
                let mut p_cam = PerspectiveCamera::new(camera_tr);
                p_cam.set_aspect_ratio(aspectratio)?;
                let mut imagetracer = ImageTracer::new(img, p_cam, antialiasing);

                println!("Rendering in progress...");
                imagetracer.fire_all_rays(&world, render_closure)?;
                img = imagetracer.image;
            }
            // ==========================================
            // HDR SAVING and LDR CONVERSION
            // ==========================================
            std::fs::create_dir_all("outputs")?;

            let pfm_filename = format!("outputs/{}.pfm", file_name);
            let ldr_filename = format!("outputs/{}.{}", file_name, cli.format);

            let file = File::create(&pfm_filename)?;
            let disk_writer = BufWriter::new(&file);

            img.write_pfm(disk_writer, &Endianness::BigEndian)?;
            println!("HDR demo image written to {}", pfm_filename);

            pfm_to_ldr(pfm_filename, 0.18, 2.2, ldr_filename.clone())?;
            println!("LDR demo image written to {}", ldr_filename);

            let duration = now.elapsed();
            println!("Rendering completed in {:.1} s", duration.as_secs_f32());
            Ok(())
        }

        Commands::Pfm2Png {
            input_file,
            output_file,
            factor_a,
            gamma,
        } => {
            pfm_to_ldr(input_file, factor_a, gamma, output_file.clone())?;
            println!("File {} has been written to disk.", output_file);

            let duration = now.elapsed();
            println!("Program finished in {:?}", duration);
            Ok(())
        }

        Commands::Render {
            input_scene_name,
            algorithm,
            pfm_output,
            png_output,
            num_of_rays,
            max_depth,
            init_state,
            init_seq,
            samples_per_pixel,
            declare_float,
        } => {
            // 1. Check Anti-aliasing samples
            let samples_per_side = (samples_per_pixel as f64).sqrt() as usize;
            if samples_per_side * samples_per_side != samples_per_pixel {
                panic!(
                    "Error, the number of samples per pixel ({}) must be a perfect square",
                    samples_per_pixel
                );
            }

            // 2. Parse command line variables
            let variables = build_variable_table(&declare_float);

            // 3. Open and Parse the Scene File
            let file = File::open(&input_scene_name)
                .map_err(|e| anyhow!("Could not open scene file {}: {}", input_scene_name, e))?;
            let reader = BufReader::new(file);

            // Initialize the InputStream (assuming a tab size of 4)
            let mut stream = rstrace::lexer::InputStream::new(reader, 0, 4);

            println!("Parsing scene '{}'...", input_scene_name);
            let scene = rstrace::parser::parse_scene(&mut stream, variables)?;

            // 4. Setup Image and Camera
            let mut img = HDR::new(cli.width, cli.height);
            let mut camera = scene
                .camera
                .expect("Error: No camera defined in the scene file!");
            
            println!("Generating a {}x{} image", cli.width, cli.height);

            let mut imagetracer = ImageTracer::new(img, camera, samples_per_pixel.isqrt());

            // 5. Setup Renderer
            let flat_renderer = FlatRenderer::new(BLACK);
            let onoff_renderer = OnOffRenderer::default();

            let mut pcg = PCG::new(init_state, init_seq);
            let path_tracer = PathTracer::new(BLACK, num_of_rays, max_depth, 2);

            let render_closure = |ray: Ray, world: &World| -> Result<Color> {
                if algorithm == "onoff" {
                    onoff_renderer.render(&ray, world, &mut pcg)
                } else if algorithm == "flat" {
                    flat_renderer.render(&ray, world, &mut pcg)
                } else if algorithm == "pathtracing" {
                    path_tracer.render(&ray, world, &mut pcg)
                } else {
                    panic!("Unknown renderer: {}", algorithm);
                }
            };

            // 6. Execute Render
            println!("Rendering in progress...");
            imagetracer.fire_all_rays(&scene.world, render_closure)?;
            img = imagetracer.image;

            // 7. Save outputs
            std::fs::create_dir_all("outputs")?;

            let file = File::create(&pfm_output)?;
            let disk_writer = BufWriter::new(&file);
            img.write_pfm(disk_writer, &Endianness::BigEndian)?;
            println!("HDR demo image written to {}", pfm_output);

            pfm_to_ldr(pfm_output, 0.18, 2.2, png_output.clone())?;
            println!("PNG demo image written to {}", png_output);

            let elapsed_time = now.elapsed();
            println!("Rendering completed in {:.1} s", elapsed_time.as_secs_f32());

            Ok(())
        }
    }
}
