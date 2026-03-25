pub mod color;
mod functions;
pub mod hdr_image;

fn are_close(x: f32, y: f32) -> bool {
    let epsilon = 1e-5;
    (x - y).abs() < epsilon
}

// test are_close
mod tests {
    use super::*;
    #[test]
    fn are_close_test() {
        let x = 0.11111;
        let y = 0.11112;

        if (!are_close(x, y)) {
            panic!("are_close is not working");
        }
    }
}
