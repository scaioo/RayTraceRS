use endianness::ByteOrder;
use crate::hdr_image;

fn endianness_number(endianness: &ByteOrder) -> f32 {
    match endianness {
        ByteOrder::LittleEndian => -1.0,
        ByteOrder::BigEndian => 1.0,
    }
}




#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn test_endianness_number() {
        assert_eq!(-1.0,endianness_number(&ByteOrder::LittleEndian));
        assert_eq!(1.0,endianness_number(&ByteOrder::BigEndian));
    }
}