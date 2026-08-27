use bytes::{Buf, BytesMut};

pub use error::DecodeError;

pub mod decoder;
pub mod encoder;
mod error;

pub trait Encode {
    // Encodes Self into the Buffer
    fn encode(&self, buffer: &mut BytesMut) -> usize;
    fn get_encoded_length(&self) -> usize;
}

pub trait Decode: Sized {
    fn decode(buffer: &[u8]) -> Result<(Self, usize), DecodeError>;

    fn decode_consuming_buffer(buffer: &mut BytesMut) -> Result<(Self, usize), DecodeError> {
        let (value, decoded_length) = Self::decode(buffer.as_ref())?;
        buffer.advance(decoded_length);
        Ok((value, decoded_length))
    }
}

#[cfg(test)]
mod test {
    use super::decoder::MinecraftPacketDecoder;
    use super::encoder::MinecraftPacketEncoder;
    use bytes::{BufMut, BytesMut};
    use tokio_util::codec::Decoder;
    use tokio_util::codec::Encoder;

    #[test]
    fn zero_length_packet() {
        // Setup codec
        let mut decoder = MinecraftPacketDecoder::new();
        let mut encoder = MinecraftPacketEncoder::new();

        // setup test data
        let test_data: BytesMut = BytesMut::from(vec![0x01, 0x00].as_slice());
        let mut test_input = test_data.clone();

        // decode data into packet
        let output = decoder.decode(&mut test_input).unwrap().unwrap();

        log::info!("Intermediate Packet: {:?}", output);

        // Reencode back into stream
        let mut test_output = BytesMut::new();
        encoder.encode(output, &mut test_output).unwrap();

        assert_eq!(test_data, test_output);
    }

    #[test]
    fn well_sized_packet() {
        // Setup codec
        let mut decoder = MinecraftPacketDecoder::new();
        let mut encoder = MinecraftPacketEncoder::new();

        // setup test data
        let test_data: BytesMut = BytesMut::from(vec![0x10; 17].as_slice());
        let mut test_input = test_data.clone();

        // decode data into packet
        let output = decoder.decode(&mut test_input).unwrap().unwrap();

        log::info!("Intermediate Packet: {:?}", output);

        // Reencode back into stream
        let mut test_output = BytesMut::new();
        encoder.encode(output, &mut test_output).unwrap();

        assert_eq!(test_data, test_output);
    }

    #[test]
    fn repeated_well_sized_packets() {
        // Setup codec
        let mut decoder = MinecraftPacketDecoder::new();
        let mut encoder = MinecraftPacketEncoder::new();

        // setup first test data
        let mut test_data: BytesMut = BytesMut::from(vec![0x10; 17].as_slice());
        let mut test_input = test_data.clone();
        let mut test_output = BytesMut::new();

        // decode data into packet
        let first_output = decoder.decode(&mut test_input).unwrap().unwrap();
        encoder.encode(first_output, &mut test_output).unwrap();

        // Add next packet
        test_data.put_bytes(0x11, 18);
        test_input.put_bytes(0x11, 18);

        // decode second
        let second_output = decoder.decode(&mut test_input).unwrap().unwrap();
        encoder.encode(second_output, &mut test_output).unwrap();

        assert_eq!(test_data, test_output);
    }

    // #[test]
    // fn double_well_sized_packets() {
    //     let mut decoder = MinecraftPacketDecoder::new();

    //     let mut test_data: BytesMut = BytesMut::from(&[0x10; 17][..]);
    //     test_data.put_bytes(0x11, 18);

    //     let output_packet = decoder.decode(&mut test_data).unwrap().unwrap();
    //     assert_eq!(
    //         output_packet,
    //         RawPacket::new(VarInt::new(16), BytesMut::from(&[0x10; 15][..]))
    //     );
    //     let output_packet = decoder.decode(&mut test_data).unwrap().unwrap();
    //     assert_eq!(
    //         output_packet,
    //         RawPacket::new(VarInt::new(17), BytesMut::from(&[0x11; 16][..]))
    //     );
    // }

    #[test]
    fn double_well_sized_packets() {
        // Setup codec
        let mut decoder = MinecraftPacketDecoder::new();
        let mut encoder = MinecraftPacketEncoder::new();

        // setup first test data
        let mut test_data: BytesMut = BytesMut::from(vec![0x10; 17].as_slice());
        test_data.put_bytes(0x11, 18);
        let test_data = test_data;
        let mut test_input = test_data.clone();
        let mut test_output = BytesMut::new();

        // decode both
        let first_output = decoder.decode(&mut test_input).unwrap().unwrap();
        let second_output = decoder.decode(&mut test_input).unwrap().unwrap();

        // Reencode both
        encoder.encode(first_output, &mut test_output).unwrap();
        encoder.encode(second_output, &mut test_output).unwrap();

        assert_eq!(test_data, test_output);
    }

    // vec![0x008606096c6f63616c686f73741ec601]
}
