use anyhow::{Result, anyhow};
use clap::Parser;
use rstrace::camera::Camera;
use rstrace::color::{BLACK, Color};
use rstrace::hdr_image::HDR;
use rstrace::image_tracer::ImageTracer;
use rstrace::pcg::PCG;
use rstrace::pfm_func::{Endianness, pfm_to_ldr};
use rstrace::ray::Ray;
use rstrace::renderer::{FlatRenderer, OnOffRenderer, PathTracer, PointLightRenderer, Renderer};
use rstrace::world::World;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::time::Instant;

#[derive(Parser)]
struct Cli {

    #[arg(long, default_value = "png")]
    format: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser, Clone)]
enum Commands {
    Pfm2Png {
        input_file: String,
        output_file: String,
        factor_a: f32,
        gamma: f32,
    },

    Render {
        input_scene_name: String,

        #[arg(long, default_value_t = 1000)]
        width: usize,

        #[arg(long, default_value_t = 750)]
        height: usize,

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

        #[arg(long, default_value_t = 5)]
        antialiasing: usize,

        /// Declare a variable. Syntax: VAR:VALUE. Example: --declare-float=clock:150
        #[arg(short = 'd', long = "declare-float")]
        declare_float: Vec<String>,
    },
}

// ====================================================================
// CONSTRUCTION OF THE SCENE
// ====================================================================

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
// FILENAMES HELPER FUNCTION
// ====================================================================
/// Ensures a filename has the given extension, adding it only if missing.
/// E.g. "output" -> "output.pfm", "output.pfm" -> "output.pfm"
fn ensure_extension(name: &str, ext: &str) -> String {
    let suffix = format!(".{}", ext);
    if name.ends_with(&suffix) {
        name.to_string()
    } else {
        format!("{}.{}", name, ext)
    }
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
            width,
            height,
            input_scene_name,
            algorithm,
            pfm_output,
            png_output,
            num_of_rays,
            max_depth,
            init_state,
            init_seq,
            antialiasing,
            declare_float,
        } => {

            // 1. Parse command line variables
            let variables = build_variable_table(&declare_float);

            // 2. Open and Parse the Scene File
            let file = File::open(&input_scene_name)
                .map_err(|e| anyhow!("Could not open scene file {}: {}", input_scene_name, e))?;
            let reader = BufReader::new(file);

            // Initialize the InputStream (assuming a tab size of 4)
            let mut stream = rstrace::lexer::InputStream::new(reader, 0, 4);

            println!("Parsing scene '{}'...", input_scene_name);
            let scene = rstrace::parser::parse_scene(&mut stream, variables)?;

            // 3. Setup Image and Camera
            let mut img = HDR::new(width, height);
            let mut camera = scene
                .camera
                .expect("Error: No camera defined in the scene file!");
            let aspect_ratio = width as f32 / height as f32;
            camera.set_aspect_ratio(aspect_ratio)?;

            println!("Generating a {}x{} image", width, height);

            let mut imagetracer = ImageTracer::new(img, camera, antialiasing);

            // 4. Setup Renderer
            let flat_renderer = FlatRenderer::new(BLACK);
            let onoff_renderer = OnOffRenderer::default();

            let mut pcg = PCG::new(init_state, init_seq);
            let path_tracer = PathTracer::new(BLACK, num_of_rays, max_depth, 2);
            let whitted = PointLightRenderer {
                background_color: BLACK,
            };

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

            // 5. Execute Render
            println!("Rendering in progress...");
            imagetracer.fire_all_rays(&scene.world, render_closure)?;
            img = imagetracer.image;

            // 9. Save outputs
            std::fs::create_dir_all("outputs")?;

            let pfm_filename = format!("outputs/{}", ensure_extension(&pfm_output, "pfm"));
            let ldr_filename = format!("outputs/{}", ensure_extension(&png_output, &cli.format));

            let file = File::create(&pfm_filename)?;
            let disk_writer = BufWriter::new(&file);
            img.write_pfm(disk_writer, &Endianness::BigEndian)?;
            println!("HDR mage written to {}", pfm_filename);

            pfm_to_ldr(pfm_filename, 0.18, 2.2, ldr_filename.clone())?;
            println!("PNG image written to {}", ldr_filename);

            let elapsed_time = now.elapsed();
            println!("Rendering completed in {:.1} s", elapsed_time.as_secs_f32());

            Ok(())
        }
    }
}
