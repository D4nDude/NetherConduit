use crate::packet::{
    primitives::VarInt,
    stream::{DecodeError, EncodeError, RawPacketDecodable, RawPacketEncodable},
};

impl<T: RawPacketDecodable> RawPacketDecodable for Vec<T> {
    fn decode(buffer: &[u8]) -> Result<(Self, usize), DecodeError> {
        let (length, length_size) = VarInt::decode(buffer)?;

        let mut data: Vec<T> = Vec::with_capacity(length.value_u32()?.try_into()?);
        let mut data_size: usize = length_size;

        for _ in 0..length.value_u32()? {
            let (element, element_size) = T::decode(buffer)?;
            data.push(element);
            data_size += element_size;
        }

        Ok((data, data_size))
    }
}

impl<T: RawPacketEncodable> RawPacketEncodable for Vec<T> {
    fn encode(&self, buffer: &mut bytes::BytesMut) -> Result<usize, EncodeError> {
        let mut data_size = VarInt::new(self.len().try_into()?).encode(buffer)?;

        for element in self {
            data_size += element.encode(buffer)?;
        }
        Ok(data_size)
    }

    fn get_encoded_length(&self) -> usize {
        let data_size = VarInt::get_var_int_encoded_length(
            self.len()
                .try_into()
                .expect("Length should not be this big?!"),
        );
        data_size + self.iter().map(|x| x.get_encoded_length()).sum::<usize>()
    }
}
