use bytes::{BufMut, Bytes, BytesMut};
use std::io;
use tokio_util::codec::{Decoder, Encoder};

pub struct ChunksCodec {
    bps: usize,
    duration: usize,
    current: usize,
}

impl ChunksCodec {
    pub fn new(bps: usize, duration: usize) -> Self {
        Self {
            bps,
            duration,
            current: 1usize,
        }
    }
}

impl Decoder for ChunksCodec {
    type Item = BytesMut;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if self.bps < buf.len() {
            self.current += 1;
            Ok(Some(buf.split_to(self.bps)))
        } else if self.current == self.duration && !buf.is_empty() {
            let remains = buf.split();
            Ok(Some(remains))
        } else {
            Ok(None)
        }
    }
}

impl Encoder<Bytes> for ChunksCodec {
    type Error = io::Error;

    fn encode(&mut self, data: Bytes, buf: &mut BytesMut) -> Result<(), io::Error> {
        buf.reserve(data.len());
        buf.put(data);
        Ok(())
    }
}
