use std::{fmt::Display, num::TryFromIntError};

use bytes::{BufMut, BytesMut};

use crate::packet::stream::{Decode, Encode, DecodeError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct VarInt(i32);

impl VarInt {
    pub fn new(value: i32) -> VarInt {
        VarInt(value)
    }

    pub fn value(self) -> i32 {
        self.0
    }
}

impl Encode for VarInt {
    fn encode(&self, buffer: &mut BytesMut) -> usize {
        let mut value: u32 = self.value() as u32;
        let mut count = 1;
        while (value & !0x7f) != 0 {
            buffer.put_u8(((value & 0x7f) as u8) | 0x80);
            value >>= 7;
            count += 1;
            debug_assert!(count <= 5);
        }
        buffer.put_u8((value & 0x7f) as u8);
        count
    }

    fn get_encoded_length(&self) -> usize {
        let value: u32 = self.value() as u32;
        if value < 0x80 {
            1
        } else if value < 0x4000 {
            2
        } else if value < 0x20_0000 {
            3
        } else if value < 0x1000_0000 {
            4
        } else {
            5
        }
    }
}

impl Decode for VarInt {
    fn decode(buffer: &[u8]) -> Result<(VarInt, usize), DecodeError> {
        let mut value: u32 = 0;

        let mut position: u32 = 0;
        for cursor in 0..5 {
            let current_byte: &u8 = match buffer.get(cursor) {
                Some(value) => value,
                None => return Err(DecodeError::Incomplete),
            };
            value |= ((current_byte & 0x7F) as u32) << position;

            if (current_byte & 0x80) == 0 {
                return Ok((VarInt(value.cast_signed()), cursor + 1));
            }

            position += 7;
        }
        Err(DecodeError::Invalid)
    }
}

impl Display for VarInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<i32> for VarInt {
    fn from(value: i32) -> Self {
        VarInt(value)
    }
}

impl From<VarInt> for i32 {
    fn from(value: VarInt) -> Self {
        value.0
    }
}

impl TryFrom<VarInt> for usize {
    type Error = TryFromIntError;
    fn try_from(value: VarInt) -> Result<Self, Self::Error> {
        usize::try_from(value.0)
    }
}

#[cfg(test)]
mod test {

    mod encode {
        use bytes::BytesMut;

        use super::super::VarInt;
        use crate::packet::stream::Encode;

        #[test]
        fn zero() {
            let mut test_buffer: BytesMut = BytesMut::with_capacity(1);
            let test_number = VarInt(0);
            let length = test_number.encode(&mut test_buffer);
            assert_eq!(1, length);
            assert_eq!([0x00], test_buffer.as_ref());
        }

        #[test]
        fn one() {
            let mut test_buffer: BytesMut = BytesMut::with_capacity(1);
            let test_number = VarInt(1);
            let length = test_number.encode(&mut test_buffer);
            assert_eq!(1, length);
            assert_eq!(1, test_number.get_encoded_length());
            assert_eq!([0x01], test_buffer.as_ref());
        }

        #[test]
        fn max_byte() {
            let mut test_buffer: BytesMut = BytesMut::with_capacity(1);
            let test_number = VarInt(127);
            let length = test_number.encode(&mut test_buffer);
            assert_eq!(1, length);
            assert_eq!(1, test_number.get_encoded_length());
            assert_eq!([0x7f], test_buffer.as_ref());
        }

        #[test]
        fn carry() {
            let mut test_buffer: BytesMut = BytesMut::with_capacity(2);
            let test_number = VarInt(128);
            let length = test_number.encode(&mut test_buffer);
            assert_eq!(2, length);
            assert_eq!(2, test_number.get_encoded_length());
            assert_eq!([0x80, 0x01], test_buffer.as_ref());
        }

        #[test]
        fn max_with_carry() {
            let mut test_buffer: BytesMut = BytesMut::with_capacity(2);
            let test_number = VarInt(255);
            let length = test_number.encode(&mut test_buffer);
            assert_eq!(2, length);
            assert_eq!(2, test_number.get_encoded_length());
            assert_eq!([0xff, 0x01], test_buffer.as_ref());
        }

        #[test]
        fn mc_port_3_bytes() {
            let mut test_buffer: BytesMut = BytesMut::with_capacity(3);
            let test_number = VarInt(25565);
            let length = test_number.encode(&mut test_buffer);
            assert_eq!(3, length);
            assert_eq!(3, test_number.get_encoded_length());
            assert_eq!([0xdd, 0xc7, 0x01], test_buffer.as_ref());
        }

        #[test]
        fn max_3_bytes() {
            let mut test_buffer: BytesMut = BytesMut::with_capacity(3);
            let test_number = VarInt(2097151);
            let length = test_number.encode(&mut test_buffer);
            assert_eq!(3, length);
            assert_eq!(3, test_number.get_encoded_length());
            assert_eq!([0xff, 0xff, 0x7f], test_buffer.as_ref());
        }

        #[test]
        fn max_int() {
            let mut test_buffer: BytesMut = BytesMut::with_capacity(5);
            let test_number = VarInt(2147483647);
            let length = test_number.encode(&mut test_buffer);
            assert_eq!(5, length);
            assert_eq!(5, test_number.get_encoded_length());
            assert_eq!([0xff, 0xff, 0xff, 0xff, 0x07], test_buffer.as_ref());
        }

        #[test]
        fn negative_one() {
            let mut test_buffer: BytesMut = BytesMut::with_capacity(5);
            let test_number = VarInt(-1);
            let length = test_number.encode(&mut test_buffer);
            assert_eq!(5, length);
            assert_eq!(5, test_number.get_encoded_length());
            assert_eq!([0xff, 0xff, 0xff, 0xff, 0x0f], test_buffer.as_ref());
        }

        #[test]
        fn min_int() {
            let mut test_buffer: BytesMut = BytesMut::with_capacity(5);
            let test_number = VarInt(-2147483648);
            let length = test_number.encode(&mut test_buffer);
            assert_eq!(5, length);
            assert_eq!(5, test_number.get_encoded_length());
            assert_eq!([0x80, 0x80, 0x80, 0x80, 0x08], test_buffer.as_ref());
        }
    }
    mod decode {
        use std::assert_matches;

        use super::super::VarInt;
        use crate::packet::stream::{Decode, DecodeError};

        #[test]
        fn zero() {
            let test_buffer: Vec<u8> = vec![0x00];
            let result = VarInt::decode(&test_buffer).unwrap();
            assert_eq!(result, (VarInt(0), 1));
        }

        #[test]
        fn one() {
            let test_buffer: Vec<u8> = vec![0x01];
            let result = VarInt::decode(&test_buffer).unwrap();
            assert_eq!(result, (VarInt(1), 1));
        }

        #[test]
        fn max_byte() {
            let test_buffer: Vec<u8> = vec![0x7f];
            let result = VarInt::decode(&test_buffer).unwrap();
            assert_eq!(result, (VarInt(127), 1));
        }

        #[test]
        fn carry() {
            let test_buffer: Vec<u8> = vec![0x80, 0x01];
            let result = VarInt::decode(&test_buffer).unwrap();
            assert_eq!(result, (VarInt(128), 2));
        }

        #[test]
        fn max_with_carry() {
            let test_buffer: Vec<u8> = vec![0xff, 0x01];
            let result = VarInt::decode(&test_buffer).unwrap();
            assert_eq!(result, (VarInt(255), 2));
        }

        #[test]
        fn mc_port_3_bytes() {
            let test_buffer: Vec<u8> = vec![0xdd, 0xc7, 0x01];
            let result = VarInt::decode(&test_buffer).unwrap();
            assert_eq!(result, (VarInt(25565), 3));
        }

        #[test]
        fn max_3_bytes() {
            let test_buffer: Vec<u8> = vec![0xff, 0xff, 0x7f];
            let result = VarInt::decode(&test_buffer).unwrap();
            assert_eq!(result, (VarInt(2097151), 3));
        }

        #[test]
        fn max_int() {
            let test_buffer: Vec<u8> = vec![0xff, 0xff, 0xff, 0xff, 0x07];
            let result = VarInt::decode(&test_buffer).unwrap();
            assert_eq!(result, (VarInt(2147483647), 5));
        }

        #[test]
        fn negative_one() {
            let test_buffer: Vec<u8> = vec![0xff, 0xff, 0xff, 0xff, 0x0f];
            let result = VarInt::decode(&test_buffer).unwrap();
            assert_eq!(result, (VarInt(-1), 5));
        }

        #[test]
        fn min_int() {
            let test_buffer: Vec<u8> = vec![0x80, 0x80, 0x80, 0x80, 0x08];
            let result = VarInt::decode(&test_buffer).unwrap();
            assert_eq!(result, (VarInt(-2147483648), 5));
        }

        #[test]
        fn too_short_buffer() {
            let test_buffer: Vec<u8> = vec![0xff, 0xff, 0xff];
            let result = VarInt::decode(&test_buffer);
            assert_matches!(result, Err(DecodeError::Incomplete));
        }

        #[test]
        fn too_long() {
            let test_buffer: Vec<u8> = vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f];
            let result = VarInt::decode(&test_buffer);
            assert_matches!(result, Err(DecodeError::Invalid));
        }
    }
}
