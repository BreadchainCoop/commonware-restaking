//! Task payload for the Solana counter example.

use bytes::{Buf, BufMut};
use commonware_codec::varint::UInt;
use commonware_codec::{EncodeSize, Read, ReadExt, Write};

/// Task payload broadcast by the router: the demo round this task certifies.
///
/// The consumer side is the NCN program's STATELESS `VerifyCertificate` — the
/// demo proves the full loop (task → engine quorum → on-chain pairing
/// verification); there is no consumer state to advance until the settlement
/// program (INTERFACES.md §4) plugs into the same handler seam.
#[derive(Debug, Clone, PartialEq)]
pub struct RoundTaskData {
    pub round: u64,
}

impl Write for RoundTaskData {
    fn write(&self, buf: &mut impl BufMut) {
        UInt(self.round).write(buf);
    }
}

impl Read for RoundTaskData {
    type Cfg = ();

    fn read_cfg(buf: &mut impl Buf, _: &()) -> Result<Self, commonware_codec::Error> {
        let round: u64 = UInt::read(buf)?.into();
        Ok(Self { round })
    }
}

impl EncodeSize for RoundTaskData {
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
        for round in [0u64, 1, 42, u32::MAX as u64, u64::MAX] {
            let original = RoundTaskData { round };
            let encoded = original.encode();
            assert_eq!(encoded.len(), original.encode_size());
            let decoded = RoundTaskData::decode(encoded).expect("decode failed");
            assert_eq!(decoded, original);
        }
    }

    #[test]
    fn truncated_encoding_rejected() {
        let encoded = RoundTaskData { round: 300 }.encode();
        let truncated = encoded.slice(0..encoded.len() - 1);
        assert!(RoundTaskData::decode(truncated).is_err());
    }
}
