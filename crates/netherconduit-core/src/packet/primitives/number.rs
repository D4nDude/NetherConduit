use byteorder::ByteOrder;
use bytes::BufMut;

use crate::packet::stream::{DecodeError, EncodeError, RawPacketDecodable, RawPacketEncodable};

// -----------------------------------------------------------------------------
// Decoding
// -----------------------------------------------------------------------------

impl RawPacketDecodable for bool {
    fn decode(buffer: &[u8]) -> Result<(Self, usize), DecodeError> {
        if buffer.is_empty() {
            return Err(DecodeError::Incomplete);
        }
        Ok((
            match buffer[0] {
                0x00 => false,
                0x01 => true,
                value => {
                    return Err(DecodeError::Invalid(format!(
                        "Bool is not valid value: {value}"
                    )));
                }
            },
            1,
        ))
    }
}

impl RawPacketDecodable for i8 {
    fn decode(buffer: &[u8]) -> Result<(Self, usize), DecodeError> {
        if buffer.is_empty() {
            return Err(DecodeError::Incomplete);
        }
        Ok((buffer[0] as i8, 1))
    }
}

impl RawPacketDecodable for u8 {
    fn decode(buffer: &[u8]) -> Result<(Self, usize), DecodeError> {
        if buffer.is_empty() {
            return Err(DecodeError::Incomplete);
        }
        Ok((buffer[0], 1))
    }
}

impl RawPacketDecodable for i16 {
    fn decode(buffer: &[u8]) -> Result<(Self, usize), DecodeError> {
        if buffer.len() < 2 {
            return Err(DecodeError::Incomplete);
        }
        Ok((byteorder::BigEndian::read_i16(buffer), 2))
    }
}

impl RawPacketDecodable for u16 {
    fn decode(buffer: &[u8]) -> Result<(Self, usize), DecodeError> {
        if buffer.len() < 2 {
            return Err(DecodeError::Incomplete);
        }
        Ok((byteorder::BigEndian::read_u16(buffer), 2))
    }
}

impl RawPacketDecodable for i32 {
    fn decode(buffer: &[u8]) -> Result<(Self, usize), DecodeError> {
        if buffer.len() < 4 {
            return Err(DecodeError::Incomplete);
        }
        Ok((byteorder::BigEndian::read_i32(buffer), 4))
    }
}

impl RawPacketDecodable for i64 {
    fn decode(buffer: &[u8]) -> Result<(Self, usize), DecodeError> {
        if buffer.len() < 8 {
            return Err(DecodeError::Incomplete);
        }
        Ok((byteorder::BigEndian::read_i64(buffer), 8))
    }
}

impl RawPacketDecodable for f32 {
    fn decode(buffer: &[u8]) -> Result<(Self, usize), DecodeError> {
        if buffer.len() < 4 {
            return Err(DecodeError::Incomplete);
        }
        Ok((byteorder::BigEndian::read_f32(buffer), 4))
    }
}

impl RawPacketDecodable for f64 {
    fn decode(buffer: &[u8]) -> Result<(Self, usize), DecodeError> {
        if buffer.len() < 8 {
            return Err(DecodeError::Incomplete);
        }
        Ok((byteorder::BigEndian::read_f64(buffer), 8))
    }
}

// -----------------------------------------------------------------------------
// Encoding
// -----------------------------------------------------------------------------

impl RawPacketEncodable for bool {
    fn encode(&self, buffer: &mut bytes::BytesMut) -> Result<usize, EncodeError> {
        match self {
            false => buffer.put_u8(0x00),
            true => buffer.put_u8(0x01),
        }
        Ok(1)
    }

    fn get_encoded_length(&self) -> usize {
        1
    }
}

impl RawPacketEncodable for i8 {
    fn encode(&self, buffer: &mut bytes::BytesMut) -> Result<usize, EncodeError> {
        buffer.put_u8(*self as u8);
        Ok(1)
    }

    fn get_encoded_length(&self) -> usize {
        1
    }
}

impl RawPacketEncodable for u8 {
    fn encode(&self, buffer: &mut bytes::BytesMut) -> Result<usize, EncodeError> {
        buffer.put_u8(*self);
        Ok(1)
    }

    fn get_encoded_length(&self) -> usize {
        1
    }
}

impl RawPacketEncodable for i16 {
    fn encode(&self, buffer: &mut bytes::BytesMut) -> Result<usize, EncodeError> {
        let buf = &mut [0; 2];
        byteorder::BigEndian::write_i16(buf, *self);
        buffer.put(&buf[..]);
        Ok(2)
    }

    fn get_encoded_length(&self) -> usize {
        2
    }
}

impl RawPacketEncodable for u16 {
    fn encode(&self, buffer: &mut bytes::BytesMut) -> Result<usize, EncodeError> {
        let buf = &mut [0; 2];
        byteorder::BigEndian::write_u16(buf, *self);
        buffer.put(&buf[..]);
        Ok(2)
    }

    fn get_encoded_length(&self) -> usize {
        2
    }
}

impl RawPacketEncodable for i32 {
    fn encode(&self, buffer: &mut bytes::BytesMut) -> Result<usize, EncodeError> {
        let buf = &mut [0; 4];
        byteorder::BigEndian::write_i32(buf, *self);
        buffer.put(&buf[..]);
        Ok(4)
    }

    fn get_encoded_length(&self) -> usize {
        4
    }
}

impl RawPacketEncodable for i64 {
    fn encode(&self, buffer: &mut bytes::BytesMut) -> Result<usize, EncodeError> {
        let buf = &mut [0; 8];
        byteorder::BigEndian::write_i64(buf, *self);
        buffer.put(&buf[..]);
        Ok(8)
    }

    fn get_encoded_length(&self) -> usize {
        8
    }
}

impl RawPacketEncodable for f32 {
    fn encode(&self, buffer: &mut bytes::BytesMut) -> Result<usize, EncodeError> {
        let buf = &mut [0; 4];
        byteorder::BigEndian::write_f32(buf, *self);
        buffer.put(&buf[..]);
        Ok(4)
    }

    fn get_encoded_length(&self) -> usize {
        4
    }
}

impl RawPacketEncodable for f64 {
    fn encode(&self, buffer: &mut bytes::BytesMut) -> Result<usize, EncodeError> {
        let buf = &mut [0; 8];
        byteorder::BigEndian::write_f64(buf, *self);
        buffer.put(&buf[..]);
        Ok(8)
    }

    fn get_encoded_length(&self) -> usize {
        8
    }
}

#[cfg(test)]
mod test {
    use bytes::BytesMut;

    use crate::packet::stream::{RawPacketDecodable, RawPacketEncodable};

    fn assert_round_trip<T>(value: T)
    where
        T: RawPacketEncodable + RawPacketDecodable + PartialEq + std::fmt::Debug,
    {
        let mut buffer = BytesMut::new();

        let encoded_length = value.get_encoded_length();
        let written = match value.encode(&mut buffer) {
            Ok(value) => value,
            Err(error) => panic!("Could not encode {:#?}! Reason: {:?}", value, error),
        };

        assert_eq!(
            written, encoded_length,
            "encode() length differs from get_encoded_length()"
        );

        assert_eq!(
            buffer.len(),
            encoded_length,
            "buffer length differs from get_encoded_length()"
        );

        let (decoded, decoded_length) = match T::decode(&buffer) {
            Ok(value) => value,
            Err(error) => panic!(
                "Could not decode {:#?}! Buffer: (0x{:x}) Reason: {:?}",
                value, buffer, error
            ),
        };

        assert_eq!(decoded, value);
        assert_eq!(decoded_length, encoded_length);

        // Test consuming-buffer decoding.
        let mut consuming_buffer = buffer.clone();

        let (decoded, consumed_length) = T::decode_consuming_buffer(&mut consuming_buffer).unwrap();

        assert_eq!(decoded, value);
        assert_eq!(consumed_length, encoded_length);
        assert!(consuming_buffer.is_empty());
    }

    // -------------------------------------------------------------------------
    // bool
    // -------------------------------------------------------------------------

    #[test]
    fn bool_round_trip() {
        assert_round_trip(false);
        assert_round_trip(true);
    }

    // -------------------------------------------------------------------------
    // byte / ubyte
    // -------------------------------------------------------------------------

    #[test]
    fn byte_round_trip() {
        assert_round_trip(i8::MIN);
        assert_round_trip(-1i8);
        assert_round_trip(0i8);
        assert_round_trip(1i8);
        assert_round_trip(i8::MAX);
    }

    #[test]
    fn ubyte_round_trip() {
        assert_round_trip(0u8);
        assert_round_trip(1u8);
        assert_round_trip(u8::MAX);
    }

    // -------------------------------------------------------------------------
    // short / ushort
    // -------------------------------------------------------------------------

    #[test]
    fn short_round_trip() {
        assert_round_trip(i16::MIN);
        assert_round_trip(-1i16);
        assert_round_trip(0i16);
        assert_round_trip(1i16);
        assert_round_trip(i16::MAX);
    }

    #[test]
    fn ushort_round_trip() {
        assert_round_trip(0u16);
        assert_round_trip(1u16);
        assert_round_trip(u16::MAX);
    }

    // -------------------------------------------------------------------------
    // int / long
    // -------------------------------------------------------------------------

    #[test]
    fn int_round_trip() {
        assert_round_trip(i32::MIN);
        assert_round_trip(-1i32);
        assert_round_trip(0i32);
        assert_round_trip(1i32);
        assert_round_trip(i32::MAX);
    }

    #[test]
    fn long_round_trip() {
        assert_round_trip(i64::MIN);
        assert_round_trip(-1i64);
        assert_round_trip(0i64);
        assert_round_trip(1i64);
        assert_round_trip(i64::MAX);
    }

    // -------------------------------------------------------------------------
    // float
    // -------------------------------------------------------------------------

    #[test]
    fn float_round_trip() {
        assert_round_trip(0.0f32);
        assert_round_trip(1.0f32);
        assert_round_trip(-1.0f32);
        assert_round_trip(f32::MIN);
        assert_round_trip(f32::MAX);
    }

    // -------------------------------------------------------------------------
    // string
    // -------------------------------------------------------------------------

    #[test]
    fn string_round_trip() {
        assert_round_trip(String::new());
        assert_round_trip("hello".to_string());
        assert_round_trip("hello world".to_string());
        assert_round_trip("こんにちは".to_string());
        assert_round_trip("😀".to_string());
    }

    // -------------------------------------------------------------------------
    // varint
    // -------------------------------------------------------------------------

    #[test]
    fn varint_round_trip() {
        assert_round_trip(0i32);
        assert_round_trip(1i32);
        assert_round_trip(127i32);
        assert_round_trip(128i32);
        assert_round_trip(255i32);
        assert_round_trip(256i32);
        assert_round_trip(16_383i32);
        assert_round_trip(16_384i32);
        assert_round_trip(i32::MAX);
    }

    // -------------------------------------------------------------------------
    // Buffer consumption
    // -------------------------------------------------------------------------

    #[test]
    fn decode_consuming_buffer_leaves_remaining_bytes() {
        let first = 123i32;
        let second = 456i32;

        let mut buffer = BytesMut::new();

        first.encode(&mut buffer).unwrap();
        second.encode(&mut buffer).unwrap();

        let first_length = first.get_encoded_length();

        let (decoded, consumed) = i32::decode_consuming_buffer(&mut buffer).unwrap();

        assert_eq!(decoded, first);
        assert_eq!(consumed, first_length);

        // The second value should still be in the buffer.
        let (decoded, consumed) = i32::decode_consuming_buffer(&mut buffer).unwrap();

        assert_eq!(decoded, second);
        assert_eq!(consumed, second.get_encoded_length());

        assert!(buffer.is_empty());
    }
}
