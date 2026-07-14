//! Router automaton: answers the verifier-only engine's digest requests.
//!
//! The aggregation engine calls `propose(h)` for every height in its window even
//! though the router never signs. Resolving the oneshot with the sequencer's
//! expected digest lets the engine mark the height `Pending::Verified` and filter
//! buffered acks against it; for unassigned heights the sender is DROPPED — the
//! engine records `AppProposeCanceled` (a warn, nothing more) and still assembles
//! certificates for that height purely from network acks. Dropping (rather than
//! parking) the sender keeps the engine's futures pool from accumulating
//! never-resolving proposals.
//!
//! Note the inherent race: the engine proposes `[tip, tip+window)` eagerly at
//! startup, usually before the sequencer assigns anything, so most proposals
//! resolve as dropped. That is expected and harmless — certificates form from
//! acks regardless.

use commonware_avs_core::consensus::trivial_verify;
use commonware_avs_core::wire::TaskData;
use commonware_consensus::Automaton;
use commonware_consensus::types::Height;
use commonware_cryptography::sha256::Digest;
use commonware_utils::channel::oneshot;
use tracing::{debug, error};

use crate::sequencer::SharedAssignments;

/// Automaton for the router's verifier-only aggregation engine.
///
/// Cheap to clone (the engine requires `Clone`): the only state is the shared
/// assignment map written by the sequencer.
#[derive(Clone)]
pub struct RouterAutomaton<T: TaskData> {
    assignments: SharedAssignments<T>,
}

impl<T: TaskData> RouterAutomaton<T> {
    pub fn new(assignments: SharedAssignments<T>) -> Self {
        Self { assignments }
    }
}

impl<T: TaskData> Automaton for RouterAutomaton<T> {
    type Context = Height;
    type Digest = Digest;

    async fn propose(&mut self, height: Height) -> oneshot::Receiver<Digest> {
        let (sender, receiver) = oneshot::channel();
        let digest = match self.assignments.read() {
            Ok(assignments) => assignments.get(&height.get()).map(|a| a.digest),
            Err(_) => {
                // Poisoned lock: a writer panicked. Treat as unassigned rather
                // than propagating the panic into the engine.
                error!(height = height.get(), "assignments lock poisoned");
                None
            }
        };
        match digest {
            Some(digest) => {
                debug!(height = height.get(), %digest, "resolving propose with expected digest");
                let _ = sender.send(digest);
            }
            None => {
                // Unassigned: drop the sender so the engine observes a canceled
                // proposal immediately instead of holding a pending future.
                drop(sender);
            }
        }
        receiver
    }

    async fn verify(&mut self, _context: Height, _payload: Digest) -> oneshot::Receiver<bool> {
        // The aggregation engine never calls verify (propose-only contract);
        // resolve trivially to satisfy the trait.
        trivial_verify()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sequencer::{Assignment, shared_assignments};
    use bytes::{Buf, BufMut};
    use commonware_codec::varint::UInt;
    use commonware_codec::{Encode, EncodeSize, Error, Read, ReadExt, Write};
    use commonware_cryptography::{Hasher, Sha256};

    /// Minimal task payload used to exercise the automaton without any
    /// application-specific dependencies.
    #[derive(Debug, Clone, PartialEq)]
    struct TestTask {
        id: u64,
    }

    impl Write for TestTask {
        fn write(&self, buf: &mut impl BufMut) {
            UInt(self.id).write(buf);
        }
    }

    impl Read for TestTask {
        type Cfg = ();

        fn read_cfg(buf: &mut impl Buf, _: &()) -> Result<Self, Error> {
            let id: u64 = UInt::read(buf)?.into();
            Ok(Self { id })
        }
    }

    impl EncodeSize for TestTask {
        fn encode_size(&self) -> usize {
            UInt(self.id).encode_size()
        }
    }

    fn digest_for(task: &TestTask) -> Digest {
        let mut hasher = Sha256::new();
        hasher.update(&task.encode());
        hasher.finalize()
    }

    #[tokio::test]
    async fn propose_resolves_assigned_height_and_drops_unassigned() {
        let assignments = shared_assignments();
        let task = TestTask { id: 1 };
        let digest = digest_for(&task);
        assignments
            .write()
            .unwrap()
            .insert(7, Assignment { digest, task });

        let mut automaton = RouterAutomaton::new(assignments);

        // Assigned: resolves with the expected digest.
        let receiver = automaton.propose(Height::new(7)).await;
        assert_eq!(receiver.await.unwrap(), digest);

        // Unassigned: the sender is dropped, surfacing a RecvError.
        let receiver = automaton.propose(Height::new(8)).await;
        assert!(receiver.await.is_err());
    }

    #[tokio::test]
    async fn verify_resolves_true() {
        let mut automaton = RouterAutomaton::<TestTask>::new(shared_assignments());
        let receiver = automaton
            .verify(Height::new(1), Digest::from([0u8; 32]))
            .await;
        assert!(receiver.await.unwrap());
    }
}
