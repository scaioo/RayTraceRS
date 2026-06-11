// tests/ldr_to_hdr_integration.rs
use image::{Rgb, RgbImage};
use rstrace::hdr_image::HDR;
use tempfile::NamedTempFile;
use rstrace::color::Color;

#[test]
fn test_load_from_ldr() {
    let pixels: Vec<[u8; 3]> = vec![
        [1,   2,   3],   [4,   5,   6],   [7,   8,   9],
        [10, 255,  20],  [25,  30,  50],  [60,  70, 100],
        [11,  12, 255],  [13,  14,   0],  [15,  16,  17],
        [18,  19,  20],  [21,  22,  23],  [21,  22,  23],
        [0,    0, 255],  [255,  0,   0],  [0,    0, 255],
    ];

    // 1. Build the LDR image — outer loop is y (rows), inner is x (cols)
    let mut ldr = RgbImage::new(3, 5);
    let mut iter = pixels.iter();
    for y in 0..5 {
        for x in 0..3 {
            ldr.put_pixel(x, y, Rgb(*iter.next().unwrap()));
        }
    }

    // 2. Save to a temp file with an explicit .png extension
    let tmp = NamedTempFile::with_suffix(".png").unwrap();
    ldr.save(tmp.path()).unwrap();

    // 3. Known parameters
    let gamma = 2.2_f32;
    let a = 0.18_f32;
    let avr_lum = 0.15_f32;

    // 4. Load via your function
    let hdr = HDR::load_from_ldr(tmp.path().to_str().unwrap(), a, avr_lum, gamma).unwrap();
    assert_eq!(hdr.width, 3);
    assert_eq!(hdr.height, 5);
    assert_eq!(hdr.pixels.len(), 15);

    // 5. Verify every pixel by replicating the formula inline
    let expected: Vec<Color> = pixels
        .iter()
        .map(|[r, g, b]| {
            let decode = |x: u8| -> f32 {
                let t = (x as f32 / 255.0).powf(gamma);
                //let t = t.min(1.0 - f32::EPSILON); // guard against x=255
                (avr_lum / a) * (t / (1.0 - t))
            };
            Color::new(decode(*r), decode(*g), decode(*b))
        })
        .collect();

    for (i, (got, exp)) in hdr.pixels.iter().zip(expected.iter()).enumerate() {
        assert!(
            got.is_close(exp),
            "pixel {i}: expected {exp:?}, got {got:?}"
        );
    }
}