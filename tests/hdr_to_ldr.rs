use image::GenericImageView;
use rstrace::hdr_image::hdr_to_ldr;
use rstrace::pfm_func::{_parse_img_size, Parameter};
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use tempfile::tempdir;
use rstrace::hdr_image::hdr_to_ldr;
use rstrace::pfm_func::Parameter;
use std::fs;
use tempfile::tempdir;
use rstrace::hdr_image::{hdr_to_ldr};
use rstrace::pfm_func::{Parameter, _parse_img_size};

#[test]
fn hdr_to_ldr_with_reference_be_pfm() -> anyhow::Result<()> {
    // 1. Create a temporary directory
    let dir = tempdir()?;

    // 2. Define input and output paths inside the temp directory
    let input_path = dir.path().join("input.pfm");
    let output_path = dir.path().join("output.png");

    // 3. Copy the real PFM file into the temp directory
    fs::copy("tests/assets/reference_be.pfm", &input_path)?;

    // 4. Build parameters
    let mut params = Parameter {
        input_pfm_file_name: input_path.to_string_lossy().to_string(),
        factor_a: 0.18,
        gamma: 2.2,
        output_file_name: output_path.to_string_lossy().to_string(),
    };

    // 5. Run the function
    hdr_to_ldr(&mut params)?;

    // 6. Check that output file exists
    assert!(output_path.exists(), "Output file was not created");

    // 7. Open the output image
    let img = image::open(&output_path)?;

    // 8. Dimension checks
    let file = File::open(&params.input_pfm_file_name);
    let mut reader = BufReader::new(file?);
    let mut line: String = String::new();
    reader.read_line(&mut line)?;
    line.clear();
    reader.read_line(&mut line)?;
    let (original_width, original_height) = _parse_img_size(&mut line)?;

    let (width, height) = img.dimensions();
    assert!(width > 0 && height > 0, "Image has invalid dimensions");
    assert_eq!(original_width, width as usize);
    assert_eq!(original_height, height as usize);

    println!("Output image size: {}x{}", width, height);

    Ok(())
}
