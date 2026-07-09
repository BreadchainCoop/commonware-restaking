//! Minimal aggregation [`Reporter`]: certificate/tip accounting for the node.
//!
//! The engine reports [`Activity`] inline from its event loop (and re-reports the
//! whole journal on restart replay), so the reporter must be non-blocking and
//! replay-idempotent. This one is a mailbox actor: [`ReporterMailbox::report`]
//! only enqueues; the [`NodeReporter`] actor deduplicates certificates by height,
//! tracks the highest observed height for metrics, and drives [`TaskBook`]
//! pruning so directive state does not grow without bound.
//!
//! [`TaskBook`]: crate::task_book::TaskBook

use commonware_actor::{Feedback, mailbox};
use commonware_avs_core::consensus::PRUNE_SLACK;
use commonware_avs_core::wire::{TaskData, skip_digest};
use commonware_consensus::Reporter;
use commonware_consensus::aggregation::types::Activity;
use commonware_cryptography::certificate::Scheme;
use commonware_cryptography::sha256::Digest;
use commonware_runtime::Metrics;
use commonware_runtime::telemetry::metrics::{Counter, Gauge, GaugeExt as _, raw};
use commonware_utils::NZUsize;
use std::collections::{BTreeSet, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::{debug, info, trace};

use crate::task_book::TaskBookMailbox;

/// Mailbox capacity before messages spill to the unbounded overflow queue.
const MAILBOX_CAPACITY: usize = 1024;

/// Activities reported by the engine, wrapped so the overflow policy can be
/// implemented locally (never drop: a lost `Certified` would undercount and a
/// lost `Tip` could stall TaskBook pruning until the next fast-forward).
struct Report<S: Scheme>(Activity<S, Digest>);

impl<S: Scheme> mailbox::Policy for Report<S> {
    type Overflow = VecDeque<Self>;

    fn handle(overflow: &mut VecDeque<Self>, message: Self) {
        overflow.push_back(message);
    }
}

/// Handle given to the engine (`Config::reporter`). Cheap to clone.
#[derive(Clone)]
pub struct ReporterMailbox<S: Scheme> {
    sender: mailbox::Sender<Report<S>>,
}

impl<S: Scheme> Reporter for ReporterMailbox<S> {
    type Activity = Activity<S, Digest>;

    /// Enqueues the activity for the actor; never blocks (the engine calls this
    /// inline from its event loop and during journal replay).
    fn report(&mut self, activity: Self::Activity) -> Feedback {
        self.sender.enqueue(Report(activity))
    }
}

/// Actor that consumes reported activities.
pub struct NodeReporter<T: TaskData + PartialEq, S: Scheme> {
    mailbox: mailbox::Receiver<Report<S>>,
    /// Prunes resolved directives as the engine's tip advances.
    task_book: TaskBookMailbox<T>,
    /// Application namespace bound into [`skip_digest`], matching the namespace
    /// used by the automaton and the router for the same deployment.
    namespace: Vec<u8>,
    /// Highest height observed via `Certified` or `Tip`, shared with the directive
    /// ingest loop (which reports it to a router assigning heights below this node's
    /// tip). This actor is the sole writer, so it doubles as the monotonicity source
    /// for [`Self::advance`]. Monotonic; replayed/stale tips are ignored.
    tip_handle: Arc<AtomicU64>,
    /// Heights already counted as certified — journal replay re-reports every
    /// certificate, so counting must dedupe. Pruned below `tip - PRUNE_SLACK`
    /// alongside the TaskBook (replay never reaches below the engine's own
    /// `activity_timeout`, which is well inside that slack).
    certified: BTreeSet<u64>,
    /// Highest certified-or-tip height (metric; handle must stay alive).
    height_gauge: Gauge,
    /// Total certificates observed (deduped by height).
    certified_counter: Counter,
    /// Subset of certificates carrying `skip_digest(namespace, h)` — heights
    /// abandoned by quorum instead of carrying a task.
    skipped_counter: Counter,
}

impl<T: TaskData + PartialEq, S: Scheme> NodeReporter<T, S> {
    /// Creates the actor and the mailbox handle to wire into the engine config.
    ///
    /// `context` labels the mailbox and metrics in the runtime registry;
    /// `tip_handle` mirrors the highest observed height for the directive ingest
    /// loop's stale-directive tip reports; `namespace` must match the namespace
    /// used to compute skip digests elsewhere in the deployment.
    pub fn new(
        context: impl Metrics,
        task_book: TaskBookMailbox<T>,
        tip_handle: Arc<AtomicU64>,
        namespace: Vec<u8>,
    ) -> (Self, ReporterMailbox<S>) {
        let height_gauge = context.register(
            "height",
            "highest aggregation height observed via certificate or tip",
            raw::Gauge::default(),
        );
        let certified_counter = context.register(
            "certified",
            "certificates observed (deduplicated by height)",
            raw::Counter::default(),
        );
        let skipped_counter = context.register(
            "skipped",
            "certificates carrying the skip digest (heights abandoned by quorum)",
            raw::Counter::default(),
        );
        let (sender, receiver) = mailbox::new(context.child("mailbox"), NZUsize!(MAILBOX_CAPACITY));
        (
            Self {
                mailbox: receiver,
                task_book,
                namespace,
                tip_handle,
                certified: BTreeSet::new(),
                height_gauge,
                certified_counter,
                skipped_counter,
            },
            ReporterMailbox { sender },
        )
    }

    /// Runs until the engine (all mailbox handles) is gone.
    pub async fn run(mut self) {
        while let Some(Report(activity)) = self.mailbox.recv().await {
            match activity {
                Activity::Certified(certificate) => {
                    let height = certificate.item.height.get();
                    if !self.certified.insert(height) {
                        // Journal replay after restart; live duplicates are
                        // already deduped by the engine.
                        trace!(height, "replayed certificate ignored");
                        continue;
                    }
                    self.certified_counter.inc();
                    let skipped = certificate.item.digest == skip_digest(&self.namespace, height);
                    if skipped {
                        self.skipped_counter.inc();
                        debug!(height, "height certified as skipped");
                    } else {
                        info!(
                            height,
                            digest = ?certificate.item.digest,
                            "height certified"
                        );
                    }
                    self.advance(height);
                }
                Activity::Tip(tip) => {
                    trace!(tip = tip.get(), "tip reported");
                    self.advance(tip.get());
                }
                Activity::Ack(ack) => {
                    // Own acks are journaled and re-reported on replay only;
                    // nothing to track beyond a trace.
                    trace!(height = ack.item.height.get(), "own ack replayed");
                }
            }
        }
        info!("reporter mailbox closed; exiting");
    }

    /// Raises the highest observed height, updates the gauge, and prunes both the
    /// TaskBook and the local dedupe set below the retention horizon.
    fn advance(&mut self, height: u64) {
        // Sole writer, so a plain load/store is race-free. `height <= current`
        // ignores stale/replayed tips; the one edge it also skips — the very first
        // activity being height 0 while `tip_handle` is still its initial 0 — is a
        // harmless no-op (gauge already 0, prune floor 0).
        if height <= self.tip_handle.load(Ordering::Relaxed) {
            return;
        }
        self.tip_handle.store(height, Ordering::Relaxed);
        let _ = self.height_gauge.try_set(height);
        let floor = height.saturating_sub(PRUNE_SLACK);
        if floor > 0 {
            self.task_book.prune_below(floor);
            self.certified = self.certified.split_off(&floor);
        }
    }
}
