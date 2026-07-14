//! Shared glue for the commonware-consensus aggregation engine used by both the
//! node (signing participant) and the router (verifier-only).

use commonware_consensus::Monitor;
use commonware_consensus::types::Epoch;
use commonware_utils::channel::{mpsc, oneshot};
use commonware_utils::sync::Mutex;
use std::sync::Arc;

/// Heights this far below the engine tip are pruned from the node's directive log
/// and dedupe sets, and drive the router's certificate bookkeeping.
///
/// Must comfortably exceed the engine's `activity_timeout` (default 256): heights
/// the engine can still re-request after a restart replay must remain answerable.
/// Shared here so the node's `TaskBook`/reporter and the router's reporter cannot
/// drift to different values.
pub const PRUNE_SLACK: u64 = 1024;

/// Resolves a [`commonware_consensus::Automaton::verify`] call trivially.
///
/// The aggregation engine never calls `verify` (it only requests digests via
/// `propose`), but the trait requires an implementation. Both the node and router
/// automatons delegate here.
pub fn trivial_verify() -> oneshot::Receiver<bool> {
    let (sender, receiver) = oneshot::channel();
    let _ = sender.send(true);
    receiver
}

/// [`Monitor`] pinned to a single static epoch: `subscribe` returns `Epoch::zero()`
/// and a channel that never fires (deployments using this monitor have no epoch
/// transitions — the operator set is fixed at process startup).
///
/// The engine exits ("epoch subscription failed") if the sender side of the
/// subscription drops, so every sender handed out is retained for the process
/// lifetime. A `Vec` of senders (rather than a single take-once `Option`) tolerates
/// the engine subscribing more than once without panicking.
#[derive(Clone, Default)]
pub struct StaticEpochMonitor {
    subscribers: Arc<Mutex<Vec<mpsc::Sender<Epoch>>>>,
}

impl StaticEpochMonitor {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Monitor for StaticEpochMonitor {
    type Index = Epoch;

    async fn subscribe(&mut self) -> (Epoch, mpsc::Receiver<Epoch>) {
        let (sender, receiver) = mpsc::channel(1);
        self.subscribers.lock().push(sender);
        (Epoch::zero(), receiver)
    }
}
