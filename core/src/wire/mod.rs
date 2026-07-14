//! Task directives broadcast by the router to the nodes (the directive p2p channel).
//!
//! The router assigns each task a monotonically increasing aggregation height and
//! announces it with [`TaskDirective::Announce`]. Because the aggregation engine's
//! tip only advances when every height certifies, a height the router abandons MUST
//! still resolve — that is what [`TaskDirective::Skip`] and [`skip_digest`] are for:
//! nodes sign `skip_digest(namespace, h)` so the height certifies with a sentinel
//! digest instead of stalling the pipeline.

use bytes::{Buf, BufMut};
use commonware_codec::varint::UInt;
use commonware_codec::{EncodeSize, Error, Read, ReadExt, Write};
use commonware_cryptography::sha256::Digest;
use commonware_cryptography::{Hasher, Sha256};

/// Wire tag for [`TaskDirective::Announce`].
const TAG_ANNOUNCE: u8 = 0;
/// Wire tag for [`TaskDirective::Skip`].
const TAG_SKIP: u8 = 1;
/// Wire tag for [`TaskDirective::TipReport`].
const TAG_TIP_REPORT: u8 = 2;

/// Domain suffix for [`skip_digest`], appended to the application namespace.
///
/// Versioned so a future format change cannot collide with old skip certificates.
const SKIP_DIGEST_SUFFIX: &[u8] = b"_SKIP_V1";

/// Bounds every task payload type must satisfy to ride a [`TaskDirective`]:
/// commonware-codec encodable with no read configuration.
pub trait TaskData:
    Clone + Send + Sync + Write + Read<Cfg = ()> + EncodeSize + std::fmt::Debug + 'static
{
}

impl<T> TaskData for T where
    T: Clone + Send + Sync + Write + Read<Cfg = ()> + EncodeSize + std::fmt::Debug + 'static
{
}

/// A router → node broadcast assigning (or abandoning) an aggregation height.
///
/// Wire format: tag `u8` + varint `UInt(height)` + (`Announce` only) the task
/// payload. The nested task read is bounded by the payload type's own codec and the
/// p2p channel's `max_message_size` caps the overall size.
#[derive(Debug, Clone, PartialEq)]
pub enum TaskDirective<T: TaskData> {
    /// Assigns `task` to aggregation height `height`; nodes validate the task and
    /// sign its expected digest.
    Announce { height: u64, task: T },
    /// Abandons `height`; nodes sign [`skip_digest`]`(namespace, height)` so the
    /// height still certifies and the pipeline advances.
    Skip { height: u64 },
    /// Node → router: "my engine tip is `height`, your directive was below it".
    ///
    /// Sent (rate-limited) by a node that receives a directive for a height below
    /// its own aggregation tip — evidence the router lost its journal and is
    /// assigning heights the nodes will never propose. The router takes the
    /// `(f+1)`-th highest report (at least one honest node reached it, same trust
    /// rule as the engine's own safe-tip) and fast-forwards its next assignment.
    TipReport { height: u64 },
}

impl<T: TaskData> TaskDirective<T> {
    /// The height this directive addresses (for [`TaskDirective::TipReport`], the
    /// reporting node's tip).
    pub fn height(&self) -> u64 {
        match self {
            TaskDirective::Announce { height, .. } => *height,
            TaskDirective::Skip { height } => *height,
            TaskDirective::TipReport { height } => *height,
        }
    }
}

impl<T: TaskData> Write for TaskDirective<T> {
    fn write(&self, buf: &mut impl BufMut) {
        match self {
            TaskDirective::Announce { height, task } => {
                TAG_ANNOUNCE.write(buf);
                UInt(*height).write(buf);
                task.write(buf);
            }
            TaskDirective::Skip { height } => {
                TAG_SKIP.write(buf);
                UInt(*height).write(buf);
            }
            TaskDirective::TipReport { height } => {
                TAG_TIP_REPORT.write(buf);
                UInt(*height).write(buf);
            }
        }
    }
}

impl<T: TaskData> Read for TaskDirective<T> {
    type Cfg = ();

    fn read_cfg(buf: &mut impl Buf, _: &()) -> Result<Self, Error> {
        let tag = u8::read(buf)?;
        let height: u64 = UInt::read(buf)?.into();
        match tag {
            TAG_ANNOUNCE => {
                let task = T::read(buf)?;
                Ok(TaskDirective::Announce { height, task })
            }
            TAG_SKIP => Ok(TaskDirective::Skip { height }),
            TAG_TIP_REPORT => Ok(TaskDirective::TipReport { height }),
            other => Err(Error::InvalidEnum(other)),
        }
    }
}

impl<T: TaskData> EncodeSize for TaskDirective<T> {
    fn encode_size(&self) -> usize {
        let tag_and_height = 1 + UInt(self.height()).encode_size();
        match self {
            TaskDirective::Announce { task, .. } => tag_and_height + task.encode_size(),
            TaskDirective::Skip { .. } | TaskDirective::TipReport { .. } => tag_and_height,
        }
    }
}

/// The sentinel digest a quorum signs to certify that height `height` carries no
/// task: `sha256(namespace || "_SKIP_V1" || height.to_be_bytes())`.
///
/// The router treats a certificate whose digest equals `skip_digest(namespace, h)`
/// as "task at `h` abandoned by quorum" and never submits it on-chain. The height is
/// bound into the preimage so skip certificates are not replayable across heights;
/// the application namespace is bound so they are not replayable across applications
/// that share an operator set. Task digests carry no such binding — they are
/// expected to be protected by the consuming contract's own replay protection.
///
/// `namespace` must be the same application namespace on the router and every node
/// (conventionally the p2p application namespace).
pub fn skip_digest(namespace: &[u8], height: u64) -> Digest {
    let mut hasher = Sha256::new();
    hasher.update(namespace);
    hasher.update(SKIP_DIGEST_SUFFIX);
    hasher.update(&height.to_be_bytes());
    hasher.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;
    use commonware_codec::{DecodeExt, Encode};

    /// Minimal task payload exercising a variable-size codec.
    #[derive(Debug, Clone, PartialEq)]
    struct TestTask {
        id: u64,
        blob: Vec<u8>,
    }

    impl Write for TestTask {
        fn write(&self, buf: &mut impl BufMut) {
            UInt(self.id).write(buf);
            self.blob.write(buf);
        }
    }

    impl Read for TestTask {
        type Cfg = ();

        fn read_cfg(buf: &mut impl Buf, _: &()) -> Result<Self, Error> {
            let id: u64 = UInt::read(buf)?.into();
            let blob = Vec::<u8>::read_cfg(buf, &(((..=1024usize).into()), ()))?;
            Ok(Self { id, blob })
        }
    }

    impl EncodeSize for TestTask {
        fn encode_size(&self) -> usize {
            UInt(self.id).encode_size() + self.blob.encode_size()
        }
    }

    fn sample_task() -> TestTask {
        TestTask {
            id: 7,
            blob: vec![0xaa, 0xbb, 0xcc],
        }
    }

    #[test]
    fn announce_roundtrip() {
        let original = TaskDirective::Announce {
            height: u64::MAX,
            task: sample_task(),
        };
        let encoded = original.encode();
        assert_eq!(encoded.len(), original.encode_size());
        let decoded = TaskDirective::<TestTask>::decode(encoded).expect("decode failed");
        assert_eq!(decoded, original);
    }

    #[test]
    fn skip_roundtrip() {
        let original = TaskDirective::<TestTask>::Skip { height: 0 };
        let encoded = original.encode();
        assert_eq!(encoded.len(), original.encode_size());
        let decoded = TaskDirective::<TestTask>::decode(encoded).expect("decode failed");
        assert_eq!(decoded, original);
    }

    #[test]
    fn tip_report_roundtrip() {
        let original = TaskDirective::<TestTask>::TipReport { height: 12_345 };
        let encoded = original.encode();
        assert_eq!(encoded.len(), original.encode_size());
        let decoded = TaskDirective::<TestTask>::decode(encoded).expect("decode failed");
        assert_eq!(decoded, original);
    }

    #[test]
    fn unknown_tag_rejected() {
        let mut bytes = TaskDirective::<TestTask>::Skip { height: 5 }.encode_mut();
        bytes[0] = 3;
        assert!(matches!(
            TaskDirective::<TestTask>::decode(bytes.freeze()),
            Err(Error::InvalidEnum(3))
        ));
    }

    #[test]
    fn truncated_announce_rejected() {
        let encoded = TaskDirective::Announce {
            height: 9,
            task: sample_task(),
        }
        .encode();
        let truncated = encoded.slice(0..encoded.len() - 1);
        assert!(TaskDirective::<TestTask>::decode(truncated).is_err());
    }

    #[test]
    fn skip_digest_is_deterministic() {
        assert_eq!(skip_digest(b"ns", 42), skip_digest(b"ns", 42));
    }

    #[test]
    fn skip_digest_distinct_per_height() {
        assert_ne!(skip_digest(b"ns", 0), skip_digest(b"ns", 1));
        assert_ne!(skip_digest(b"ns", 1), skip_digest(b"ns", 256));
        // Big-endian height bytes: byte-shifted heights must not collide.
        assert_ne!(skip_digest(b"ns", 1 << 8), skip_digest(b"ns", 1 << 16));
    }

    #[test]
    fn skip_digest_distinct_per_namespace() {
        assert_ne!(skip_digest(b"app-a", 7), skip_digest(b"app-b", 7));
    }
}
