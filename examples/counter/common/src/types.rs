use bytes::{Buf, BufMut};
use commonware_codec::varint::UInt;
use commonware_codec::{EncodeSize, Read, ReadExt, Write};

/// Task payload broadcast by the router: the `Counter.number()` round this task
/// certifies.
#[derive(Debug, Clone, PartialEq)]
pub struct CounterTaskData {
    pub round: u64,
}

impl Write for CounterTaskData {
    fn write(&self, buf: &mut impl BufMut) {
        UInt(self.round).write(buf);
    }
}

impl Read for CounterTaskData {
    type Cfg = ();

    fn read_cfg(buf: &mut impl Buf, _: &()) -> Result<Self, commonware_codec::Error> {
        let round: u64 = UInt::read(buf)?.into();
        Ok(Self { round })
    }
}

impl EncodeSize for CounterTaskData {
    fn encode_size(&self) -> usize {
        UInt(self.round).encode_size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use commonware_codec::{DecodeExt, Encode};

    #[test]
    fn roundtrip() {
        let original = CounterTaskData { round: 42 };
        let encoded = original.encode();
        assert_eq!(encoded.len(), original.encode_size());
        let decoded = CounterTaskData::decode(encoded).expect("decode failed");
        assert_eq!(decoded, original);
    }

    #[test]
    fn roundtrip_boundary_values() {
        for round in [0u64, 1, u32::MAX as u64, u64::MAX] {
            let original = CounterTaskData { round };
            let encoded = original.encode();
            assert_eq!(encoded.len(), original.encode_size());
            let decoded = CounterTaskData::decode(encoded).expect("decode failed");
            assert_eq!(decoded, original);
        }
    }

    #[test]
    fn truncated_encoding_rejected() {
        let encoded = CounterTaskData { round: 300 }.encode();
        let truncated = encoded.slice(0..encoded.len() - 1);
        assert!(CounterTaskData::decode(truncated).is_err());
    }
}
