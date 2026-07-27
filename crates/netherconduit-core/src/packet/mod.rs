use bytes::BytesMut;

pub mod primitives;
pub mod stream;

#[derive(Debug, PartialEq, Eq)]
pub struct RawPacket {
    data: BytesMut,
}

impl RawPacket {
    pub fn new(data: BytesMut) -> RawPacket {
        RawPacket { data }
    }

    pub fn get_data(self) -> BytesMut {
        self.data
    }
}
