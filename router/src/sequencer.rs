//! Task sequencer: assigns aggregation heights to application-supplied tasks and
//! drives each one to resolution.
//!
//! The aggregation engine's tip only advances when every height certifies, so the
//! sequencer follows two hard rules:
//!
//! 1. **Exactly one outstanding height at a time.** `next_height` starts at the
//!    reporter tip observed after a short settle delay (journal replay) and only
//!    advances when the current height resolves — a certificate was observed (real
//!    digest or skip digest) AND, for real digests, on-chain execution finished
//!    (successfully or not).
//! 2. **Every assigned height MUST resolve to a certificate.** The sequencer
//!    rebroadcasts `TaskDirective::Announce` every `rebroadcast_interval`; after
//!    `round_timeout` without a certificate it switches to broadcasting
//!    `TaskDirective::Skip` (and keeps rebroadcasting until the height certifies —
//!    with either digest; a late real-digest certificate is still submitted while
//!    the assignment is cached).
//!
//! Pre-restart leftovers: a certificate for the current height whose digest is
//! neither the expected digest nor `skip_digest(height)` (a node resolved the
//! height with a directive from a previous router life) consumes the height — the
//! in-flight task is re-assigned to the next height. Heights certified without an
//! assignment simply advance `next_height` past them. Heights between the tip and
//! the first assignment that never certified (non-contiguous pre-restart
//! certificates) are resolved by the nodes' auto-skip rule once they see this
//! router's first directive for a higher height.

use crate::reporter::CertIndex;
use commonware_avs_core::wire::{TaskData, TaskDirective};
use commonware_codec::{DecodeExt, Encode};
use commonware_cryptography::PublicKey;
use commonware_cryptography::sha256::Digest;
use commonware_p2p::{Receiver as NetworkReceiver, Recipients, Sender as NetworkSender};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tracing::{debug, error, info, warn};

/// How long the sequencer waits after startup before reading the reporter tip.
///
/// Covers the engine's journal replay (which re-reports certified heights and the
/// tip into the [`crate::reporter::CertReporter`]) so `next_height` starts where the
/// previous router life stopped instead of re-assigning already-certified heights.
const SETTLE_DELAY: Duration = Duration::from_secs(2);

/// Per-height dispatch timestamps: the sequencer inserts `height -> Instant::now()`
/// when it assigns a task, and a consumer (typically the submitter) removes the
/// matching entry to compute the p2p round-trip duration.
///
/// Keying by height (rather than a single shared slot) means a height that fails
/// without being consumed cannot bleed its stale timestamp into the next height's
/// measurement. Heights advance monotonically and the sequencer runs one at a time,
/// so the sequencer evicts any entry older than the height it is dispatching,
/// keeping the map bounded even when heights repeatedly fail.
pub type DispatchTime = Arc<Mutex<HashMap<u64, Instant>>>;

/// Records `height`'s dispatch instant for later round-trip measurement, first
/// evicting any entry from an earlier height. Heights advance monotonically and the
/// sequencer runs one at a time, so an older entry belongs to a height that failed
/// without completing; dropping it here keeps the map bounded.
pub fn stamp_dispatch_time(times: &DispatchTime, height: u64) {
    if let Ok(mut times) = times.lock() {
        times.retain(|&h, _| h >= height);
        times.insert(height, Instant::now());
    }
}

/// Removes and returns `height`'s dispatch instant, if one was recorded. Consuming
/// the entry both yields the round-trip start and prevents the completed height from
/// lingering in the map.
pub fn take_dispatch_time(times: &DispatchTime, height: u64) -> Option<Instant> {
    times
        .lock()
        .ok()
        .and_then(|mut times| times.remove(&height))
}

/// A task ready for height assignment, produced by the application's [`TaskSource`].
pub struct SequencedTask<T: TaskData> {
    /// Full task retained in the assignment and handed to the submitter on
    /// execution.
    pub task: T,
    /// The view broadcast to nodes in `Announce` directives. Applications may strip
    /// fields nodes recompute independently (smaller frames survive unreliable p2p
    /// sends better); defaults to the full task if there is nothing to strip.
    pub announce: T,
    /// The digest a correct node signs for this task.
    pub digest: Digest,
}

/// Produces the tasks a [`Sequencer`] assigns to aggregation heights.
///
/// Implementations own whatever ingress mechanism (a queue, an HTTP endpoint, a
/// poll loop) feeds the application's tasks, plus any per-task preparation (e.g.
/// computing the fields the digest is built from) that needs a fresh view of the
/// world at assignment time.
#[async_trait::async_trait]
pub trait TaskSource<T: TaskData>: Send {
    /// Blocks until the next task is ready to assign. The sequencer calls this only
    /// when it is ready to drive a new height, so implementations can defer
    /// expensive preparation (fresh-state enrichment) until this moment. Returning
    /// `None` shuts the sequencer down.
    async fn next_task(&mut self) -> Option<SequencedTask<T>>;
}

/// The expected outcome for an assigned height, shared between the sequencer (which
/// writes it), the automaton (which resolves `propose(h)` with `digest`), and the
/// submitter (which needs `task` for the on-chain submission).
#[derive(Clone)]
pub struct Assignment<T: TaskData> {
    /// The digest a correct node signs for this height.
    pub digest: Digest,
    /// The task data announced (and, on certification, submitted) for this height.
    pub task: T,
}

/// Height → assignment map. Entries live from assignment until the height resolves;
/// the sequencer removes them, so the map holds at most the single in-flight
/// height.
pub type SharedAssignments<T> = Arc<RwLock<BTreeMap<u64, Assignment<T>>>>;

pub fn shared_assignments<T: TaskData>() -> SharedAssignments<T> {
    Arc::new(RwLock::new(BTreeMap::new()))
}

/// How a certified height was consumed, reported by the submitter to the
/// sequencer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionKind {
    /// The certificate carried the assignment's expected digest and the on-chain
    /// submission finished (`success` reflects the on-chain outcome).
    Executed { success: bool },
    /// The certificate carried `skip_digest(height)`: the quorum abandoned the
    /// height and the task (if any) was dropped.
    Skipped,
    /// The certificate carried a digest that is neither the expected digest nor the
    /// skip digest, or the height had no assignment (pre-restart leftovers). The
    /// height is consumed; an in-flight task must be re-assigned.
    Foreign,
}

/// A certified height's final disposition (submitter → sequencer).
#[derive(Debug, Clone, Copy)]
pub struct Resolution {
    pub height: u64,
    pub kind: ResolutionKind,
}

pub type ResolutionSender = UnboundedSender<Resolution>;
pub type ResolutionReceiver = UnboundedReceiver<Resolution>;

pub fn resolution_channel() -> (ResolutionSender, ResolutionReceiver) {
    mpsc::unbounded_channel()
}

/// Why [`Sequencer::drive_height`] returned.
enum HeightOutcome {
    /// The height resolved (certificate + execution); see [`ResolutionKind`].
    Resolved(ResolutionKind),
    /// Node tip reports prove the quorum is already past this height (the router
    /// lost its journal and assigned a height the nodes will never propose) — the
    /// task must be re-assigned at the reported tip.
    Superseded,
    /// The resolution channel closed (submitter died) — shut down.
    Closed,
}

/// Node tip reports (the directive p2p channel, node → router), keyed by operator
/// identity.
///
/// A node that receives a directive for a height below its own engine tip replies
/// with [`TaskDirective::TipReport`] — evidence this router lost its journal and is
/// driving heights the nodes will never propose (their engines only propose
/// `[tip, tip+window)`). [`TipReports::safe_tip`] applies the same trust rule as the
/// engine's safe-tip: the `(f+1)`-th highest report is reachable only if at least
/// one honest node reached it (`f = (n-1)/3`, the N3f1 fault bound).
///
/// Callers must only [`TipReports::record`] reports from authenticated quorum
/// participants (see [`ingest_tip_reports`]).
#[derive(Clone)]
pub struct TipReports<P: PublicKey> {
    /// Highest tip reported per participant (monotonic).
    reports: Arc<RwLock<BTreeMap<P, u64>>>,
    /// Faults tolerated by the engine's N3f1 quorum rule.
    max_faults: usize,
}

impl<P: PublicKey> TipReports<P> {
    pub fn new(participant_count: usize) -> Self {
        Self {
            reports: Arc::new(RwLock::new(BTreeMap::new())),
            max_faults: participant_count.saturating_sub(1) / 3,
        }
    }

    /// Records `tip` for `peer`, keeping the per-peer maximum.
    pub fn record(&self, peer: P, tip: u64) {
        if let Ok(mut reports) = self.reports.write() {
            let entry = reports.entry(peer).or_insert(0);
            *entry = (*entry).max(tip);
        }
    }

    /// The `(f+1)`-th highest reported tip (0 until that many nodes reported).
    pub fn safe_tip(&self) -> u64 {
        let Ok(reports) = self.reports.read() else {
            return 0;
        };
        let mut tips: Vec<u64> = reports.values().copied().collect();
        tips.sort_unstable_by(|a, b| b.cmp(a));
        tips.get(self.max_faults).copied().unwrap_or(0)
    }
}

/// Consumes the router's side of the directive p2p channel, recording node tip
/// reports.
///
/// Only quorum participants may influence the sequencer's height choice; other
/// directive variants (the router's own broadcasts echoed back by a buggy peer)
/// are ignored. Malformed payloads are logged and dropped.
pub async fn ingest_tip_reports<T, P, R>(
    mut receiver: R,
    participants: HashSet<P>,
    reports: TipReports<P>,
) where
    T: TaskData,
    P: PublicKey,
    R: NetworkReceiver<PublicKey = P>,
{
    loop {
        match receiver.recv().await {
            Ok((peer, bytes)) => {
                if !participants.contains(&peer) {
                    warn!(peer = %peer, "tip report from non-participant; ignored");
                    continue;
                }
                match TaskDirective::<T>::decode(bytes) {
                    Ok(TaskDirective::TipReport { height }) => {
                        debug!(peer = %peer, height, "node tip report");
                        reports.record(peer, height);
                    }
                    Ok(directive) => {
                        debug!(
                            height = directive.height(),
                            "non-report directive on router side; ignored"
                        );
                    }
                    Err(error) => {
                        warn!(%error, "malformed directive on router side; ignored");
                    }
                }
            }
            Err(error) => {
                info!(?error, "directive channel closed; exiting");
                return;
            }
        }
    }
}

/// Pulls tasks from the application and drives one aggregation height at a time.
pub struct Sequencer<T, P, S, R, TS>
where
    T: TaskData,
    P: PublicKey,
    S: NetworkSender<PublicKey = P>,
    R: CertIndex,
    TS: TaskSource<T>,
{
    source: TS,
    /// Shared with the height's consumer to measure p2p round-trip duration.
    dispatch_time: DispatchTime,
    /// Shared with the automaton and submitter.
    assignments: SharedAssignments<T>,
    /// Certificate observations (tip + per-height digests) from the engine.
    reporter: R,
    /// Final dispositions from the submitter.
    resolutions: ResolutionReceiver,
    /// Broadcasts [`TaskDirective`]s to the nodes on the directive p2p channel.
    directive_sender: S,
    /// Explicit directive recipients (the participant/operator keys).
    ///
    /// Directives are sent to these keys via `Recipients::Some` rather than
    /// `Recipients::All`: the p2p `LimitedSender` resolves `Recipients::All` from a
    /// lazily-populated connected-peer snapshot that stays empty on the router's
    /// send side (its inbound-heavy directive-channel usage never warms it),
    /// silently dropping every `Announce`. Addressing the known operator set
    /// directly bypasses that snapshot.
    recipients: Vec<P>,
    /// Node tip reports; a safe tip above the driven height supersedes it.
    tip_reports: TipReports<P>,
    /// Next height to assign; only advances when the current height resolves.
    next_height: u64,
    round_timeout: Duration,
    rebroadcast_interval: Duration,
}

impl<T, P, S, R, TS> Sequencer<T, P, S, R, TS>
where
    T: TaskData,
    P: PublicKey,
    S: NetworkSender<PublicKey = P>,
    R: CertIndex,
    TS: TaskSource<T>,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        source: TS,
        dispatch_time: DispatchTime,
        assignments: SharedAssignments<T>,
        reporter: R,
        resolutions: ResolutionReceiver,
        directive_sender: S,
        recipients: Vec<P>,
        tip_reports: TipReports<P>,
        round_timeout: Duration,
        rebroadcast_interval: Duration,
    ) -> Self {
        Self {
            source,
            dispatch_time,
            assignments,
            reporter,
            resolutions,
            directive_sender,
            recipients,
            tip_reports,
            next_height: 0,
            round_timeout,
            rebroadcast_interval,
        }
    }

    /// Main loop: pull the next task from the application → assign height → drive
    /// to resolution → repeat.
    pub async fn run(mut self) {
        // Let the engine replay its journal into the reporter before choosing the
        // first height, so we resume where the previous router life stopped.
        tokio::time::sleep(SETTLE_DELAY).await;
        self.next_height = self
            .reporter
            .get_tip()
            .await
            .max(self.tip_reports.safe_tip());
        info!(
            next_height = self.next_height,
            "sequencer starting at reporter tip"
        );

        loop {
            // Consume any resolutions that arrived while idle (heights certified
            // without an assignment — pre-restart leftovers); never re-assign them.
            self.drain_resolutions();

            let Some(sequenced) = self.source.next_task().await else {
                info!("task source exhausted, sequencer shutting down");
                return;
            };

            // Assign heights until the task is consumed. A foreign-digest
            // certificate consumes the height but not the task, so re-assign it to
            // the next height (liveness rule 2).
            let mut pending = Some(sequenced);
            while let Some(sequenced) = pending.take() {
                self.drain_resolutions();
                // Never assign below the engine tip (e.g. tip fast-forwarded past
                // heights certified elsewhere while we were executing) or below
                // the nodes' reported tips (journal loss on this router).
                self.next_height = self
                    .next_height
                    .max(self.reporter.get_tip().await)
                    .max(self.tip_reports.safe_tip());
                let height = self.next_height;
                self.next_height += 1;

                if let Ok(mut assignments) = self.assignments.write() {
                    assignments.insert(
                        height,
                        Assignment {
                            digest: sequenced.digest,
                            task: sequenced.task.clone(),
                        },
                    );
                } else {
                    error!("assignments lock poisoned, dropping task");
                    break;
                }
                stamp_dispatch_time(&self.dispatch_time, height);
                info!(
                    height,
                    digest = %sequenced.digest,
                    task = ?sequenced.task,
                    "assigned task to height"
                );

                let outcome = self.drive_height(height, &sequenced).await;
                if let Ok(mut assignments) = self.assignments.write() {
                    assignments.remove(&height);
                }
                match outcome {
                    HeightOutcome::Resolved(ResolutionKind::Executed { success }) => {
                        info!(height, success, "height executed, releasing sequencer");
                    }
                    HeightOutcome::Resolved(ResolutionKind::Skipped) => {
                        // Liveness rule: a skip certificate for our height means the
                        // quorum abandoned the task — drop it (the client observes
                        // no on-chain effect) and move on.
                        warn!(height, "quorum skipped our height, dropping task");
                    }
                    HeightOutcome::Resolved(ResolutionKind::Foreign) => {
                        // Pre-restart leftover consumed the height; the task is
                        // still ours to deliver — re-assign it to the next height.
                        warn!(
                            height,
                            "height certified with a foreign digest, re-assigning task"
                        );
                        pending = Some(sequenced);
                    }
                    HeightOutcome::Superseded => {
                        // Node tip reports prove the quorum is past this height —
                        // it certified (or was skipped) in a previous router life
                        // and can never certify again. Re-assign at the safe tip.
                        warn!(
                            height,
                            safe_tip = self.tip_reports.safe_tip(),
                            "height superseded by node tip reports, re-assigning task"
                        );
                        pending = Some(sequenced);
                    }
                    HeightOutcome::Closed => {
                        error!("resolution channel closed, sequencer shutting down");
                        return;
                    }
                }
            }
        }
    }

    /// Applies buffered resolutions to `next_height` without blocking.
    fn drain_resolutions(&mut self) {
        while let Ok(resolution) = self.resolutions.try_recv() {
            debug!(
                height = resolution.height,
                kind = ?resolution.kind,
                "observed resolution for unassigned height"
            );
            self.next_height = self.next_height.max(resolution.height + 1);
        }
    }

    /// Broadcasts (and rebroadcasts) the directive for `height` until it resolves.
    ///
    /// Announces `sequenced.announce`; after `round_timeout` without a certificate,
    /// switches to `Skip{height}` so the height still certifies and the pipeline
    /// advances. Timeout granularity is the rebroadcast interval. Rebroadcasting
    /// stops once the reporter has a certificate for the height (waiting only on
    /// execution).
    async fn drive_height(&mut self, height: u64, sequenced: &SequencedTask<T>) -> HeightOutcome {
        let announce = TaskDirective::Announce {
            height,
            task: sequenced.announce.clone(),
        }
        .encode();
        let skip = TaskDirective::<T>::Skip { height }.encode();
        let deadline = Instant::now() + self.round_timeout;
        let mut skipping = false;
        // Latches once the height certifies, so the post-certification wait for the
        // height's consumer (execution may take minutes) stops re-polling the
        // reporter actor every tick.
        let mut certified = false;

        self.broadcast(announce.clone());
        loop {
            tokio::select! {
                resolution = self.resolutions.recv() => {
                    let Some(resolution) = resolution else {
                        return HeightOutcome::Closed;
                    };
                    if resolution.height == height {
                        return HeightOutcome::Resolved(resolution.kind);
                    }
                    // A different height resolved (pre-restart leftover certified
                    // while we drive ours) — never assign it again.
                    debug!(
                        height = resolution.height,
                        kind = ?resolution.kind,
                        "observed resolution for another height while driving"
                    );
                    self.next_height = self.next_height.max(resolution.height + 1);
                }
                _ = tokio::time::sleep(self.rebroadcast_interval) => {
                    // Once certified, the directive is moot — the resolution is all
                    // that remains (execution may take minutes; don't broadcast
                    // Skip for an already-certified height). Poll the reporter only
                    // until it certifies, then latch and idle.
                    if certified || self.reporter.get(height).await.is_some() {
                        certified = true;
                        continue;
                    }
                    // Nodes past this height will never sign it (their engines
                    // only propose at-or-above their tips) — stop driving it.
                    if self.tip_reports.safe_tip() > height {
                        return HeightOutcome::Superseded;
                    }
                    if !skipping && Instant::now() >= deadline {
                        warn!(
                            height,
                            timeout_secs = self.round_timeout.as_secs_f64(),
                            "no certificate before round timeout, broadcasting skip"
                        );
                        skipping = true;
                    }
                    let directive = if skipping { skip.clone() } else { announce.clone() };
                    self.broadcast(directive);
                }
            }
        }
    }

    /// Broadcasts an encoded [`TaskDirective`] to the operator set on the directive
    /// p2p channel.
    ///
    /// Uses `Recipients::Some(self.recipients)` — the explicit participant keys —
    /// rather than `Recipients::All`, whose connected-peer snapshot stays empty on
    /// the router's send side (see the `recipients` field). An empty result
    /// (everyone rate-limited or momentarily disconnected) is expected transiently;
    /// the rebroadcast loop retries.
    fn broadcast(&mut self, message: bytes::Bytes) {
        let sent =
            self.directive_sender
                .send(Recipients::Some(self.recipients.clone()), message, true);
        if sent.is_empty() {
            debug!("directive broadcast reached no peers (will rebroadcast)");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{Buf, BufMut};
    use commonware_codec::varint::UInt;
    use commonware_codec::{EncodeSize, Error, Read, ReadExt, Write};
    use commonware_cryptography::{Hasher, Sha256};

    /// Minimal task payload used to exercise the sequencer without any
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

    #[test]
    fn test_dispatch_time_evicts_failed_heights_and_isolates_measurements() {
        let times: DispatchTime = Arc::new(Mutex::new(HashMap::new()));

        // Height 0 is dispatched but never certifies/executes, so it is never
        // consumed.
        stamp_dispatch_time(&times, 0);
        assert_eq!(times.lock().unwrap().len(), 1);

        // Dispatching height 1 evicts the stale height-0 entry rather than letting
        // it accumulate or bleed into height 1's measurement.
        stamp_dispatch_time(&times, 1);
        {
            let map = times.lock().unwrap();
            assert_eq!(map.len(), 1, "stale failed-height entry should be evicted");
            assert!(map.contains_key(&1));
            assert!(!map.contains_key(&0));
        }

        // Height 1's own timestamp is consumed exactly once; the failed height 0
        // is gone.
        assert!(take_dispatch_time(&times, 0).is_none());
        assert!(take_dispatch_time(&times, 1).is_some());
        assert!(take_dispatch_time(&times, 1).is_none());
        assert!(times.lock().unwrap().is_empty());
    }

    #[test]
    fn test_assignment_map_shares_digest_and_task() {
        let assignments = shared_assignments::<TestTask>();
        let task = TestTask { id: 3 };
        let mut hasher = Sha256::new();
        hasher.update(&task.encode());
        let digest = hasher.finalize();

        assignments.write().unwrap().insert(
            5,
            Assignment {
                digest,
                task: task.clone(),
            },
        );

        let read = assignments.read().unwrap();
        let assignment = read.get(&5).expect("assignment present");
        assert_eq!(assignment.digest, digest);
        assert_eq!(assignment.task.id, 3);
        assert!(read.get(&6).is_none());
    }
}
