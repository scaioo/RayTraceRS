mod cli;

use anyhow::{Result, anyhow};
use clap::Parser;
use cli::{
    CliReflectancePolicy, build_variable_table, create_parent_dir, ensure_extension,
    ensure_image_extension,
};
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
/// (`pfm-ldr`), and rendering a scene description file (`render`).
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser, Clone)]
enum Commands {
    /// Convert a PFM (HDR, floating-point) image into a viewable LDR image,
    /// applying tone mapping (average luminosity normalization) and gamma
    /// correction.
    PfmLdr {
        /// Path to the input PFM file.
        input_file: String,

        /// Path to the output LDR image file (the extension determines the format).
        output_file: String,

        /// Normalization factor `a` used during tone mapping: scales pixel
        /// luminosity before compression. Higher values brighten the image.
        /// The default (0.18) is the conventional middle-grey reference.
        #[arg(long, default_value_t = 0.18)]
        factor_a: f32,

        /// Gamma correction exponent applied after tone mapping
        /// (typically in the 1.0-2.2 range). The default (2.2) matches the
        /// standard sRGB display response.
        #[arg(long, default_value_t = 2.2)]
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

        /// Output raster format for the final rendered image.
        /// Accepted values: `png`, `jpg`, `jpeg`.
        #[arg(long, default_value = "png")]
        format: String,

        /// Path to the output PFM (raw HDR) file. Absolute paths and
        /// subdirectories are honored; any missing parent directories are
        /// created automatically. The `.pfm` extension is added if not already
        /// present.
        #[arg(long, default_value = "outputs/output.pfm")]
        pfm_output: String,

        /// Path to the output raster image. Absolute paths and subdirectories
        /// are honored; any missing parent directories are created
        /// automatically. The extension is derived from `--format`
        /// (png, jpg, jpeg) if not already present.
        #[arg(long, default_value = "outputs/output")]
        image_output: String,

        /// Number of rays sampled per pixel (used by the path tracer).
        #[arg(long, default_value_t = 5)]
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

        /// Policy for pigments with reflectance channels outside [0,1].
        /// Accepted values: `reject`, `rescale`, `ignore`.
        #[arg(long, default_value = "reject")]
        reflectance_policy: CliReflectancePolicy,

        /// Declare a variable. Syntax: VAR:VALUE. Example: --declare-float=clock:150
        #[arg(short = 'd', long = "declare-float")]
        declare_float: Vec<String>,

        /// Number of threads used for rendering.
        /// Use 1 to disable multi-threading; 0 (the default) uses all CPU cores.
        #[arg(long, default_value_t = 0)]
        threads: usize,
    },
}

// ====================================================================
// MAIN PROGRAM
// ====================================================================
fn main() -> Result<()> {
    let now = Instant::now();
    println!("\n------------------------------------------------------\n");

    let cli = Cli::parse();

    match cli.command {
        Commands::PfmLdr {
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
            format,
            tab_size,
            reflectance_policy,
            declare_float,
            threads,
        } => {
            // 0.1 Validate format and configure the rendering thread pool
            if !["png", "jpeg", "jpg"].contains(&format.as_str()) {
                return Err(anyhow!(
                    "Invalid value '{}' for --format: expected one of png, jpg, jpeg",
                    format
                ));
            }

            if threads > 0 {
                rayon::ThreadPoolBuilder::new()
                    .num_threads(threads)
                    .build_global()
                    .map_err(|e| anyhow!("Could not configure the thread pool: {}", e))?;
            }

            // 1. Parse command line variables
            let variables = build_variable_table(&declare_float);

            // 2. Open and Parse the Scene File
            let file = File::open(&input_scene_name)
                .map_err(|e| anyhow!("Could not open scene file {}: {}", input_scene_name, e))?;
            let reader = BufReader::new(file);

            // Initialize the InputStream with the user-configured tab size
            let mut stream = rstrace::lexer::InputStream::new(reader, 0, tab_size);

            println!("Parsing scene '{}'...", input_scene_name);

            let scene = rstrace::parser::parse_scene_with_policy(
                &mut stream,
                variables,
                reflectance_policy.into(),
            )?;

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

            let pcg = PCG::new(init_state, init_seq);
            let path_tracer = PathTracer::new(BLACK, num_of_rays, max_depth, 2);
            let whitted = PointLightRenderer {
                background_color: BLACK,
            };

            let render_closure = |ray: Ray, world: &World, pcg: &mut PCG| -> Result<Color> {
                if algorithm == "onoff" {
                    onoff_renderer.render(&ray, world, pcg)
                } else if algorithm == "flat" {
                    flat_renderer.render(&ray, world, pcg)
                } else if algorithm == "pathtracing" {
                    path_tracer.render(&ray, world, pcg)
                } else if algorithm == "point-light" {
                    whitted.render(&ray, world, pcg)
                } else {
                    panic!("Unknown algorithm: {}", algorithm);
                }
            };

            // 5. Execute Render
            println!("Rendering in progress...");
            imagetracer.fire_all_rays(&scene.world, pcg, render_closure)?;
            img = imagetracer.image;

            // 6. Save outputs (respecting the user-provided paths)
            let pfm_filename = ensure_extension(&pfm_output, "pfm");
            let ldr_filename = ensure_image_extension(&image_output, &format);

            create_parent_dir(&pfm_filename)?;
            let file = File::create(&pfm_filename)
                .map_err(|e| anyhow!("Could not create output file {}: {}", pfm_filename, e))?;
            let disk_writer = BufWriter::new(&file);
            img.write_pfm(disk_writer, &Endianness::BigEndian)?;
            println!("HDR image written to {}", pfm_filename);

            create_parent_dir(&ldr_filename)?;
            pfm_to_ldr(pfm_filename, 0.18, 2.2, ldr_filename.clone()).map_err(|e| {
                anyhow!(
                    "Could not write {} image {}: {}",
                    format.to_uppercase(),
                    ldr_filename,
                    e
                )
            })?;
            println!(
                "{} image written to {}",
                format.to_uppercase(),
                ldr_filename
            );

            let elapsed_time = now.elapsed();
            println!("Rendering completed in {:.1} s", elapsed_time.as_secs_f32());

            Ok(())
        }
    }
}
