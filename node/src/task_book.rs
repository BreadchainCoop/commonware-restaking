//! TaskBook actor: resolves aggregation heights to the router's task directives.
//!
//! The aggregation engine calls `Automaton::propose(h)` for every height in its
//! window, but which task (if any) lives at `h` is decided solely by the router's
//! [`TaskDirective`] broadcasts on p2p channel 1. The TaskBook bridges the two: it
//! owns the directive log and hands out await-able per-height resolutions.
//!
//! # Skip rules (NORMATIVE)
//!
//! A pending height `h` resolves to [`Resolution::Skip`] only when:
//! 1. an explicit `TaskDirective::Skip { h }` arrives, OR
//! 2. a directive for some `h' > h` arrives while `h` has no directive — evidence
//!    the router moved past `h` (it serializes assignments, so a directive at `h'`
//!    means every unassigned lower height was abandoned).
//!
//! There are NO timer-based skips: propose futures for unassigned heights simply
//! stay parked until the router says otherwise. The engine proposes heights
//! `[tip, tip+window)` eagerly at startup, so parked waiters during idle periods
//! are normal — resolving them on a timer would burn heights and generate constant
//! skip-certificate churn.
//!
//! # Idempotency
//!
//! The router rebroadcasts directives until a height certifies, so duplicates are
//! expected: identical replays for an already-recorded height are ignored. The
//! router also has exactly one legitimate way to change its mind about a height:
//! after `round_timeout` elapses without a certificate, it latches from
//! `Announce` to broadcasting `Skip` for that same height, and never reverts. An
//! incoming `Skip` that replaces a recorded `Announce` for the same height is
//! therefore treated as that override, not a conflict: the recorded directive is
//! replaced and any still-parked waiters resolve to [`Resolution::Skip`]. Any
//! other mismatch (two different `Announce`s, or a later `Announce` for a height
//! already recorded as `Skip`) is a genuine router anomaly and is logged and
//! dropped, keeping the first-recorded directive.

use commonware_actor::mailbox;
use commonware_avs_core::bn254::PublicKey;
use commonware_avs_core::consensus::PRUNE_SLACK;
use commonware_avs_core::wire::{TaskData, TaskDirective};
use commonware_codec::{DecodeExt, Encode};
use commonware_p2p::{Receiver, Recipients, Sender};
use commonware_runtime::Metrics;
use commonware_utils::NZUsize;
use commonware_utils::channel::oneshot;
use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tracing::{debug, info, trace, warn};

/// How an aggregation height resolved.
#[derive(Debug, Clone)]
pub enum Resolution<T: TaskData> {
    /// The router announced `task` for the height: validate it and sign the
    /// expected task digest.
    Announce(T),
    /// The height carries no task: sign `skip_digest(height)` so the height still
    /// certifies and the pipeline advances.
    Skip,
}

/// Mailbox capacity before messages spill to the unbounded overflow queue.
///
/// Overflow never drops messages (directives and subscriptions must both be
/// reliable); the bound only limits the lock-free fast path.
const MAILBOX_CAPACITY: usize = 1024;

/// Messages processed by the [`TaskBook`] actor.
enum Message<T: TaskData + PartialEq> {
    /// A decoded directive from the router (channel 1).
    Directive(TaskDirective<T>),
    /// A request for the resolution of `height`; answered as soon as the skip
    /// rules allow, possibly immediately.
    Subscribe {
        height: u64,
        responder: oneshot::Sender<Resolution<T>>,
    },
    /// Drop resolved entries below `floor` (driven by the reporter's tip).
    PruneBelow(u64),
}

impl<T: TaskData + PartialEq> mailbox::Policy for Message<T> {
    type Overflow = VecDeque<Self>;

    fn handle(overflow: &mut VecDeque<Self>, message: Self) {
        // Never drop: losing a directive could wedge a height on this node and
        // losing a subscription would leave the engine's propose pending forever.
        overflow.push_back(message);
    }
}

/// Handle for feeding and querying the [`TaskBook`] actor. Cheap to clone.
#[derive(Clone)]
pub struct TaskBookMailbox<T: TaskData + PartialEq> {
    sender: mailbox::Sender<Message<T>>,
}

impl<T: TaskData + PartialEq> TaskBookMailbox<T> {
    /// Records a directive. Non-blocking; delivery failures (actor gone) are
    /// logged — the process is shutting down at that point anyway.
    pub fn deliver(&self, directive: TaskDirective<T>) {
        let height = directive.height();
        if !self
            .sender
            .enqueue(Message::Directive(directive))
            .accepted()
        {
            warn!(height, "task book closed; directive dropped");
        }
    }

    /// Subscribes to the resolution of `height`.
    ///
    /// The returned receiver yields the [`Resolution`] as soon as the skip rules
    /// permit (immediately if the height is already decided). It errors only if
    /// the TaskBook actor is gone.
    pub fn subscribe(&self, height: u64) -> oneshot::Receiver<Resolution<T>> {
        let (responder, receiver) = oneshot::channel();
        let message = Message::Subscribe { height, responder };
        if !self.sender.enqueue(message).accepted() {
            // The responder inside the rejected message is dropped, so the
            // receiver resolves with RecvError — callers observe the failure.
            warn!(height, "task book closed; subscription unresolvable");
        }
        receiver
    }

    /// Requests pruning of resolved entries below `floor`.
    pub fn prune_below(&self, floor: u64) {
        let _ = self.sender.enqueue(Message::PruneBelow(floor));
    }
}

/// Actor owning the directive log and the parked per-height waiters.
pub struct TaskBook<T: TaskData + PartialEq> {
    mailbox: mailbox::Receiver<Message<T>>,
    /// First directive recorded per height (see the idempotency rules above).
    directives: BTreeMap<u64, TaskDirective<T>>,
    /// Highest height with any directive; drives skip rule 2 and pruning.
    max_seen: Option<u64>,
    /// Subscribers awaiting a resolution. Invariant: every parked height has no
    /// directive and is `> max_seen` (anything else resolves immediately).
    waiters: BTreeMap<u64, Vec<oneshot::Sender<Resolution<T>>>>,
}

impl<T: TaskData + PartialEq> TaskBook<T> {
    /// Creates the actor and its mailbox handle. `metrics` labels the mailbox's
    /// backoff counter in the runtime registry.
    pub fn new(metrics: impl Metrics) -> (Self, TaskBookMailbox<T>) {
        let (sender, receiver) = mailbox::new(metrics.child("mailbox"), NZUsize!(MAILBOX_CAPACITY));
        (
            Self {
                mailbox: receiver,
                directives: BTreeMap::new(),
                max_seen: None,
                waiters: BTreeMap::new(),
            },
            TaskBookMailbox { sender },
        )
    }

    /// Runs until every mailbox handle is dropped.
    pub async fn run(mut self) {
        while let Some(message) = self.mailbox.recv().await {
            match message {
                Message::Directive(directive) => self.handle_directive(directive),
                Message::Subscribe { height, responder } => {
                    self.handle_subscribe(height, responder)
                }
                Message::PruneBelow(floor) => self.prune_below(floor),
            }
        }
        info!("task book mailbox closed; exiting");
    }

    fn handle_directive(&mut self, directive: TaskDirective<T>) {
        // TipReports are node → router traffic; `ingest` filters them out before
        // delivery, so one landing here is a caller bug — never record it as a
        // height directive.
        if matches!(directive, TaskDirective::TipReport { .. }) {
            warn!("tip report delivered to task book; ignored");
            return;
        }
        let height = directive.height();

        // Idempotency: identical replays are routine. A recorded `Announce`
        // overridden by a `Skip` for the same height is the router's one
        // legitimate change of mind (it latches after `round_timeout`; see the
        // module docs) — apply it and resolve any still-parked waiters. Any other
        // mismatch is a genuine anomaly: keep the first-recorded directive.
        if let Some(existing) = self.directives.get(&height) {
            if *existing == directive {
                trace!(height, "duplicate directive ignored");
            } else if matches!(existing, TaskDirective::Announce { .. })
                && matches!(directive, TaskDirective::Skip { .. })
            {
                debug!(
                    height,
                    "router abandoned previously announced height; switching to skip"
                );
                self.directives.insert(height, directive);
                if let Some(waiters) = self.waiters.remove(&height) {
                    for waiter in waiters {
                        let _ = waiter.send(Resolution::Skip);
                    }
                }
            } else {
                warn!(
                    height,
                    "conflicting directive for already-recorded height; keeping first"
                );
            }
            return;
        }

        // Directives below the prune horizon are stale (rule 2 already resolves
        // those heights as Skip); recording them would regrow pruned state.
        if let Some(max_seen) = self.max_seen
            && height < max_seen.saturating_sub(PRUNE_SLACK)
        {
            debug!(height, max_seen, "directive below prune horizon; ignored");
            return;
        }

        let resolution = resolution(&directive);
        debug!(
            height,
            announce = matches!(directive, TaskDirective::Announce { .. }),
            "recorded directive"
        );
        self.directives.insert(height, directive);

        // Resolve waiters parked at exactly this height.
        if let Some(waiters) = self.waiters.remove(&height) {
            for waiter in waiters {
                let _ = waiter.send(resolution.clone());
            }
        }

        // Skip rule 2: a directive at `height` is evidence the router moved past
        // every lower height that never got one — resolve those waiters as Skip.
        if self.max_seen.is_none_or(|max_seen| height > max_seen) {
            self.max_seen = Some(height);
            let passed_over: Vec<u64> = self.waiters.range(..height).map(|(h, _)| *h).collect();
            for h in passed_over {
                // Defensive: parked waiters never have a directive (invariant),
                // but re-check rather than mis-resolve if that ever regresses.
                if self.directives.contains_key(&h) {
                    continue;
                }
                if let Some(waiters) = self.waiters.remove(&h) {
                    debug!(
                        height = h,
                        directive_height = height,
                        "router moved past height without a directive; resolving as skip"
                    );
                    for waiter in waiters {
                        let _ = waiter.send(Resolution::Skip);
                    }
                }
            }
            self.prune_below(height.saturating_sub(PRUNE_SLACK));
        }
    }

    fn handle_subscribe(&mut self, height: u64, responder: oneshot::Sender<Resolution<T>>) {
        // Already decided by an explicit directive.
        if let Some(directive) = self.directives.get(&height) {
            let _ = responder.send(resolution(directive));
            return;
        }
        // Skip rule 2 for late subscribers: the router already moved past this
        // height without assigning it (covers pruned heights too).
        if let Some(max_seen) = self.max_seen
            && height < max_seen
        {
            debug!(
                height,
                max_seen, "subscription below highest directive; resolving as skip"
            );
            let _ = responder.send(Resolution::Skip);
            return;
        }
        // Undecided: park until a directive arrives (no timer — see module docs).
        self.waiters.entry(height).or_default().push(responder);
    }

    fn prune_below(&mut self, floor: u64) {
        // `split_off` keeps entries >= floor.
        let kept = self.directives.split_off(&floor);
        let pruned = self.directives.len();
        self.directives = kept;
        if pruned > 0 {
            trace!(floor, pruned, "pruned resolved directives");
        }
    }
}

/// Maps a recorded directive to the resolution handed to subscribers.
fn resolution<T: TaskData + PartialEq>(directive: &TaskDirective<T>) -> Resolution<T> {
    match directive {
        TaskDirective::Announce { task, .. } => Resolution::Announce(task.clone()),
        // TipReports are rejected before recording (`handle_directive`); the arm
        // exists only for exhaustiveness and mirrors Skip's no-task semantics.
        TaskDirective::Skip { .. } | TaskDirective::TipReport { .. } => Resolution::Skip,
    }
}

/// Returns whether `height` falls at or beyond the engine's proposal window,
/// i.e. `height >= tip + window` (saturating, so an overflowing `tip + window`
/// is treated as unreachable rather than wrapping).
///
/// A height out here can never be proposed by this node's engine — it only
/// calls `Automaton::propose` for `[tip, tip + window)` — so a directive for it
/// is a signal the node's tip is stuck, not routine router noise.
fn beyond_window(height: u64, tip: u64, window: u64) -> bool {
    height >= tip.saturating_add(window)
}

/// Consumes p2p channel 1, decoding [`TaskDirective`]s and feeding the TaskBook.
///
/// Only the router assigns heights, so directives from any other (authorized but
/// non-router) peer are rejected — otherwise a single faulty operator could inject
/// `Skip`s and split signers across digests. Malformed payloads are logged and
/// ignored (the p2p layer already authenticated the sender; garbage here is a
/// peer bug, not grounds to kill the node).
///
/// Stale-directive recovery: a directive for a height below this node's engine tip
/// is evidence the router lost its journal and is assigning heights this node will
/// never propose (its engine only proposes `[tip, tip+window)`). Without feedback
/// the router would rebroadcast a dead height forever, wedging the pipeline — so
/// the node replies with a rate-limited [`TaskDirective::TipReport`] carrying its
/// tip, and the router fast-forwards its next assignment.
///
/// `window` is the same value passed as the engine config's `window`: the number
/// of heights above `tip` the engine works on concurrently. A directive at or
/// past `tip + window` can never be proposed locally, so this node can never sign
/// it — evidence its tip has stopped advancing (typically because it cannot reach
/// its peer operators and so never assembles the certificates that move the
/// tip), even though it can still reach the router. That condition produces no
/// TipReport (the directive isn't below tip) and no error anywhere else, so it is
/// logged here via a rate-limited `warn!` — observability only; the directive is
/// still delivered to the TaskBook exactly as any other.
pub async fn ingest<T, R, S>(
    mut receiver: R,
    mut sender: S,
    router: PublicKey,
    task_book: TaskBookMailbox<T>,
    engine_tip: Arc<AtomicU64>,
    window: u64,
    min_report_interval: Duration,
) where
    T: TaskData + PartialEq,
    R: Receiver<PublicKey = PublicKey>,
    S: Sender<PublicKey = PublicKey>,
{
    let mut last_report: Option<Instant> = None;
    let mut last_beyond_window_warning: Option<Instant> = None;
    loop {
        match receiver.recv().await {
            Ok((peer, bytes)) => {
                if peer != router {
                    warn!(peer = %peer, "task directive from non-router peer; ignored");
                    continue;
                }
                match TaskDirective::<T>::decode(bytes) {
                    Ok(TaskDirective::TipReport { height }) => {
                        // Node → router only; the router never sends these.
                        debug!(height, "unexpected tip report from router; ignored");
                    }
                    Ok(directive) => {
                        trace!(height = directive.height(), "received task directive");
                        let tip = engine_tip.load(Ordering::Relaxed);
                        let directive_height = directive.height();
                        if directive_height < tip
                            && last_report.is_none_or(|at| at.elapsed() >= min_report_interval)
                        {
                            last_report = Some(Instant::now());
                            debug!(
                                directive_height,
                                tip, "directive below engine tip; reporting tip to router"
                            );
                            let report = TaskDirective::<T>::TipReport { height: tip }.encode();
                            let _ = sender.send(Recipients::One(router.clone()), report, true);
                        }
                        if beyond_window(directive_height, tip, window)
                            && last_beyond_window_warning
                                .is_none_or(|at| at.elapsed() >= min_report_interval)
                        {
                            last_beyond_window_warning = Some(Instant::now());
                            warn!(
                                directive_height,
                                tip,
                                window,
                                "directive beyond the engine's proposal window; this node cannot \
                                 sign it — engine tip is not advancing (check operator-to-operator \
                                 connectivity)"
                            );
                        }
                        task_book.deliver(directive);
                    }
                    Err(error) => {
                        warn!(%error, "malformed task directive; ignored");
                    }
                }
            }
            Err(error) => {
                // The network is shutting down; nothing to recover here.
                info!(?error, "task directive channel closed; exiting");
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{Buf, BufMut};
    use commonware_codec::varint::UInt;
    use commonware_codec::{EncodeSize, Error, Read, ReadExt, Write};
    use commonware_runtime::{Runner, Spawner, Supervisor, deterministic};

    /// Minimal task payload used to exercise the TaskBook without any
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

    fn announce(height: u64) -> TaskDirective<TestTask> {
        TaskDirective::Announce {
            height,
            task: TestTask { id: height },
        }
    }

    /// Runs `f` against a live TaskBook actor inside the deterministic runtime.
    fn with_task_book<F, Fut>(f: F)
    where
        F: FnOnce(TaskBookMailbox<TestTask>) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let executor = deterministic::Runner::default();
        executor.start(|context| async move {
            let (task_book, mailbox) = TaskBook::new(context.child("task_book"));
            context.child("actor").spawn(move |_| task_book.run());
            f(mailbox).await;
        });
    }

    #[test]
    fn announce_resolves_existing_and_future_subscribers() {
        with_task_book(|mailbox| async move {
            // Parked before the directive arrives.
            let early = mailbox.subscribe(5);
            mailbox.deliver(announce(5));
            assert!(matches!(early.await.unwrap(), Resolution::Announce(t) if t.id == 5));
            // Resolves immediately after.
            let late = mailbox.subscribe(5);
            assert!(matches!(late.await.unwrap(), Resolution::Announce(_)));
        });
    }

    #[test]
    fn explicit_skip_resolves_height() {
        with_task_book(|mailbox| async move {
            let waiter = mailbox.subscribe(3);
            mailbox.deliver(TaskDirective::Skip { height: 3 });
            assert!(matches!(waiter.await.unwrap(), Resolution::Skip));
        });
    }

    #[test]
    fn higher_directive_skips_passed_over_heights() {
        with_task_book(|mailbox| async move {
            let low = mailbox.subscribe(1);
            let mid = mailbox.subscribe(2);
            let high = mailbox.subscribe(3);
            mailbox.deliver(announce(3));
            // Heights 1 and 2 had no directive when 3 arrived: skip.
            assert!(matches!(low.await.unwrap(), Resolution::Skip));
            assert!(matches!(mid.await.unwrap(), Resolution::Skip));
            assert!(matches!(high.await.unwrap(), Resolution::Announce(_)));
            // Late subscriber below max_seen also skips.
            assert!(matches!(
                mailbox.subscribe(0).await.unwrap(),
                Resolution::Skip
            ));
        });
    }

    #[test]
    fn recorded_directive_survives_later_higher_directive() {
        with_task_book(|mailbox| async move {
            mailbox.deliver(announce(1));
            mailbox.deliver(announce(2));
            // Height 1 HAD a directive when 2 arrived: it must not become a skip.
            assert!(matches!(
                mailbox.subscribe(1).await.unwrap(),
                Resolution::Announce(t) if t.id == 1
            ));
        });
    }

    #[test]
    fn announce_then_skip_overrides_to_skip() {
        with_task_book(|mailbox| async move {
            // Already resolved from the first directive: unaffected by the
            // override, since a resolved oneshot cannot be un-resolved.
            let early = mailbox.subscribe(4);
            mailbox.deliver(announce(4));
            assert!(matches!(early.await.unwrap(), Resolution::Announce(_)));
            // The router latched to skip after its round timeout: this replaces
            // the recorded directive, not a conflict.
            mailbox.deliver(TaskDirective::Skip { height: 4 });
            // A late subscriber now sees the override.
            assert!(matches!(
                mailbox.subscribe(4).await.unwrap(),
                Resolution::Skip
            ));
        });
    }

    #[test]
    fn skip_override_is_sticky_against_announce_replay() {
        with_task_book(|mailbox| async move {
            mailbox.deliver(announce(4));
            mailbox.deliver(TaskDirective::Skip { height: 4 });
            // The router rebroadcasting its old Announce (a race with, or after,
            // its own switch to skip) must not flip the height back: Skip is
            // sticky, since the router itself never reverts Skip to Announce.
            mailbox.deliver(announce(4));
            assert!(matches!(
                mailbox.subscribe(4).await.unwrap(),
                Resolution::Skip
            ));
        });
    }

    #[test]
    fn differing_announce_conflict_keeps_first() {
        with_task_book(|mailbox| async move {
            mailbox.deliver(announce(4));
            // A second, differing Announce for the same height is not the
            // Announce-to-Skip override and remains a genuine conflict.
            mailbox.deliver(TaskDirective::Announce {
                height: 4,
                task: TestTask { id: 999 },
            });
            assert!(matches!(
                mailbox.subscribe(4).await.unwrap(),
                Resolution::Announce(t) if t.id == 4
            ));
        });
    }

    #[test]
    fn unassigned_future_heights_stay_pending() {
        with_task_book(|mailbox| async move {
            mailbox.deliver(announce(1));
            let mut pending = mailbox.subscribe(2);
            // No directive for 2 and nothing above it: must NOT resolve (no
            // timer-based skips). try_recv is empty rather than closed/ready.
            assert!(matches!(
                pending.try_recv(),
                Err(oneshot::error::TryRecvError::Empty)
            ));
        });
    }

    // `ingest` takes p2p `Receiver`/`Sender` trait objects with no lightweight
    // mock available in this workspace (unlike the deterministic actor harness
    // above, which only needs the TaskBook mailbox). The threshold check is
    // factored into `beyond_window` so it can be unit tested directly instead.
    #[test]
    fn beyond_window_boundaries() {
        // Last height still inside the window: tip + window - 1.
        assert!(!beyond_window(9, 0, 10));
        assert!(!beyond_window(19, 10, 10));
        // First height outside the window: tip + window.
        assert!(beyond_window(10, 0, 10));
        assert!(beyond_window(20, 10, 10));
        // Well past the window.
        assert!(beyond_window(100, 10, 10));
        // Overflowing tip + window saturates instead of wrapping, so a huge
        // height is (correctly) still reported as beyond the window rather than
        // spuriously appearing "within" it after wraparound.
        assert!(beyond_window(u64::MAX, u64::MAX - 1, 10));
        assert!(!beyond_window(u64::MAX - 1, u64::MAX - 1, 10));
    }
}
