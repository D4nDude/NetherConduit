#[allow(dead_code)]
pub fn peak_varint(buffer: &[u8]) -> Option<(i32, usize)> {
    log::trace!("Reading Var Int");
    let mut cursor: usize = 0;
    let mut value: u32 = 0;

    let mut position: u32 = 0;
    while position < 32 {
        let current_byte: &u8 = buffer.get(cursor)?;
        value |= ((current_byte & 0x7F) as u32) << position;

        if (current_byte & 0x80) == 0 {
            return Some((value.cast_signed(), cursor + 1));
        }

        position += 7;
        cursor += 1;
    }
    None
}

#[cfg(test)]
mod test {

    mod peak_var_int {
        use std::assert_matches;

        use crate::packet::primitives::peak_varint;

        #[test]
        fn zero() {
            let test_buffer: Vec<u8> = vec![0x00];
            let result = peak_varint(&test_buffer).unwrap();
            assert_eq!(result, (0, 1));
        }

        #[test]
        fn one() {
            let test_buffer: Vec<u8> = vec![0x01];
            let result = peak_varint(&test_buffer).unwrap();
            assert_eq!(result, (1, 1));
        }

        #[test]
        fn max_byte() {
            let test_buffer: Vec<u8> = vec![0x7f];
            let result = peak_varint(&test_buffer).unwrap();
            assert_eq!(result, (127, 1));
        }

        #[test]
        fn carry() {
            let test_buffer: Vec<u8> = vec![0x80, 0x01];
            let result = peak_varint(&test_buffer).unwrap();
            assert_eq!(result, (128, 2));
        }

        #[test]
        fn max_with_carry() {
            let test_buffer: Vec<u8> = vec![0xff, 0x01];
            let result = peak_varint(&test_buffer).unwrap();
            assert_eq!(result, (255, 2));
        }

        #[test]
        fn mc_port_3_bytes() {
            let test_buffer: Vec<u8> = vec![0xdd, 0xc7, 0x01];
            let result = peak_varint(&test_buffer).unwrap();
            assert_eq!(result, (25565, 3));
        }

        #[test]
        fn max_3_bytes() {
            let test_buffer: Vec<u8> = vec![0xff, 0xff, 0x7f];
            let result = peak_varint(&test_buffer).unwrap();
            assert_eq!(result, (2097151, 3));
        }

        #[test]
        fn max_int() {
            let test_buffer: Vec<u8> = vec![0xff, 0xff, 0xff, 0xff, 0x07];
            let result = peak_varint(&test_buffer).unwrap();
            assert_eq!(result, (2147483647, 5));
        }

        #[test]
        fn negative_one() {
            let test_buffer: Vec<u8> = vec![0xff, 0xff, 0xff, 0xff, 0x0f];
            let result = peak_varint(&test_buffer).unwrap();
            assert_eq!(result, (-1, 5));
        }

        #[test]
        fn min_int() {
            let test_buffer: Vec<u8> = vec![0x80, 0x80, 0x80, 0x80, 0x08];
            let result = peak_varint(&test_buffer).unwrap();
            assert_eq!(result, (-2147483648, 5));
        }

        #[test]
        fn too_short_buffer() {
            let test_buffer: Vec<u8> = vec![0xff, 0xff, 0xff];
            let result = peak_varint(&test_buffer);
            assert_matches!(result, None);
        }

        #[test]
        fn too_long() {
            let test_buffer: Vec<u8> = vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f];
            let result = peak_varint(&test_buffer);
            assert_matches!(result, None);
        }
    }
}
