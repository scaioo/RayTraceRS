use endianness::ByteOrder;

fn are_close(x: f32, y: f32) -> bool {
    let epsilon = 1e-5;
    (x - y).abs() < epsilon
}

//          ENDIANNESS --> +-1.0
pub fn endianness_number(endianness: &ByteOrder) -> f32 {
    match endianness {
        ByteOrder::LittleEndian => -1.0,
        ByteOrder::BigEndian => 1.0,
    }
}

// test are_close
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn are_close_test() {
        let x = 0.11111;
        let y = 0.11112;

        if !are_close(x, y) {
            panic!("are_close is not working");
        }
    }

    #[test]
    fn test_endianness_number() {
        assert_eq!(-1.0, endianness_number(&ByteOrder::LittleEndian));
        assert_eq!(1.0, endianness_number(&ByteOrder::BigEndian));
    }
}
