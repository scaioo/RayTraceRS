mod cli;

use anyhow::{Result, anyhow};
use clap::Parser;
use cli::{build_variable_table, ensure_extension, ensure_image_extension};
use rstrace::camera::Camera;
use rstrace::color::{BLACK, Color};
use rstrace::hdr_image::HDR;
use rstrace::image_tracer::ImageTracer;
use rstrace::pcg::PCG;
use rstrace::pfm_func::{Endianness, pfm_to_ldr};
use rstrace::ray::Ray;
use rstrace::renderer::{FlatRenderer, OnOffRenderer, PathTracer, PointLightRenderer, Renderer};
use rstrace::world::World;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::time::Instant;

/// Command-line interface for the rstrace ray tracer.
///
/// Supports two subcommands: converting a PFM file into a viewable LDR image
/// (`pfm2png`), and rendering a scene description file (`render`).
#[derive(Parser)]
struct Cli {
    /// Output raster format for the final rendered image.
    /// Accepted values: `png`, `jpg`, `jpeg`.
    #[arg(long, default_value = "png")]
    format: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser, Clone)]
enum Commands {
    /// Convert a PFM (HDR, floating-point) image into a viewable LDR image,
    /// applying tone mapping (average luminosity normalization) and gamma
    /// correction.
    Pfm2Png {
        /// Path to the input PFM file.
        input_file: String,

        /// Path to the output LDR image file (the extension determines the format).
        output_file: String,

        /// Normalization factor `a` used during tone mapping: scales pixel
        /// luminosity before compression. Higher values brighten the image.
        factor_a: f32,

        /// Gamma correction exponent applied after tone mapping
        /// (typically in the 1.0-2.2 range).
        gamma: f32,
    },

    /// Parse a scene description file and render it, producing both a PFM
    /// (raw HDR) file and a tone-mapped raster image.
    Render {
        /// Path to the scene description file to render.
        input_scene_name: String,

        /// Width of the rendered image, in pixels.
        #[arg(long, default_value_t = 1000)]
        width: usize,

        /// Height of the rendered image, in pixels.
        #[arg(long, default_value_t = 750)]
        height: usize,

        /// Rendering algorithm to use.
        /// Accepted values: `pathtracing`, `flat`, `onoff`, `point-light`.
        #[arg(long, default_value = "pathtracing")]
        algorithm: String,

        /// Name of the output PFM (raw HDR) file. The `.pfm` extension is
        /// added automatically if not already present.
        #[arg(long, default_value = "output.pfm")]
        pfm_output: String,

        /// Name of the output raster image. The extension is derived
        /// automatically from `--format` (png, jpg, jpeg) if not already present.
        #[arg(long, default_value = "output")]
        image_output: String,

        /// Number of rays sampled per pixel (used by the path tracer).
        #[arg(long, default_value_t = 10)]
        num_of_rays: usize,

        /// Maximum recursion depth for ray bounces (used by the path tracer).
        #[arg(long, default_value_t = 3)]
        max_depth: usize,

        /// Initial state of the PCG pseudo-random number generator.
        #[arg(long, default_value_t = 45)]
        init_state: u64,

        /// Initial sequence (stream) identifier of the PCG pseudo-random number generator.
        #[arg(long, default_value_t = 54)]
        init_seq: u64,

        /// Number of samples per pixel side used for antialiasing
        /// (total samples per pixel = antialiasing²).
        #[arg(long, default_value_t = 5)]
        antialiasing: usize,

        /// Number of spaces a tab character (`\t`) counts for when the parser
        /// computes column numbers in error messages.
        #[arg(long, default_value_t = 4)]
        tab_size: usize,

        /// Declare a variable. Syntax: VAR:VALUE. Example: --declare-float=clock:150
        #[arg(short = 'd', long = "declare-float")]
        declare_float: Vec<String>,
    },
}

// ====================================================================
// MAIN PROGRAM
// ====================================================================
fn main() -> Result<()> {
    let now = Instant::now();
    println!("\n------------------------------------------------------\n");

    let cli = Cli::parse();

    if !["png", "jpeg", "jpg"].contains(&cli.format.as_str()) {
        return Err(anyhow!(
            "Invalid value '{}' for --format: expected one of png, jpg, jpeg",
            cli.format
        ));
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
            image_output,
            num_of_rays,
            max_depth,
            init_state,
            init_seq,
            antialiasing,
            tab_size,
            declare_float,
        } => {
            // 1. Parse command line variables
            let variables = build_variable_table(&declare_float);

            // 2. Open and Parse the Scene File
            let file = File::open(&input_scene_name)
                .map_err(|e| anyhow!("Could not open scene file {}: {}", input_scene_name, e))?;
            let reader = BufReader::new(file);

            // Initialize the InputStream with the user-configured tab size
            let mut stream = rstrace::lexer::InputStream::new(reader, 0, tab_size);

            println!("Parsing scene '{}'...", input_scene_name);
            let scene = rstrace::parser::parse_scene(&mut stream, variables)?;

            // 3. Setup Image and Camera
            let mut img = HDR::new(width, height);
            let mut camera = scene.camera.ok_or_else(|| {
                anyhow!(
                    "No camera defined in scene file '{}': add a `camera(...)` statement.",
                    input_scene_name
                )
            })?;
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

            // 6. Save outputs
            std::fs::create_dir_all("outputs")?;

            let pfm_filename = format!("outputs/{}", ensure_extension(&pfm_output, "pfm"));
            let ldr_filename = format!(
                "outputs/{}",
                ensure_image_extension(&image_output, &cli.format)
            );

            let file = File::create(&pfm_filename)?;
            let disk_writer = BufWriter::new(&file);
            img.write_pfm(disk_writer, &Endianness::BigEndian)?;
            println!("HDR image written to {}", pfm_filename);

            pfm_to_ldr(pfm_filename, 0.18, 2.2, ldr_filename.clone())?;
            println!(
                "{} image written to {}",
                cli.format.to_uppercase(),
                ldr_filename
            );

            let elapsed_time = now.elapsed();
            println!("Rendering completed in {:.1} s", elapsed_time.as_secs_f32());

            Ok(())
        }
    }
}
