//! Minimal aggregation [`Reporter`]: certificate/tip accounting for the node.
//!
//! The engine reports [`Activity`] inline from its event loop (and re-reports the
//! whole journal on restart replay), so the reporter must be non-blocking and
//! replay-idempotent. This one is a mailbox actor: [`ReporterMailbox::report`]
//! only enqueues; the [`NodeReporter`] actor deduplicates certificates by height,
//! tracks the highest observed height for metrics, and drives [`TaskBook`]
//! pruning so directive state does not grow without bound.
//!
//! Applications whose schemes need post-certification work run their own actor
//! beside this one and opt in to a certificate tap
//! ([`NodeReporter::with_certificate_tap`]): the reporter then forwards one
//! [`CertificateObservation`] per certified height to that consumer, deduplicated
//! across journal replays exactly like its own counters.
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
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
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

/// A certificate observation forwarded to an application-side consumer (e.g. an
/// actor running scheme-specific post-certification work). Sent once per height,
/// deduplicated across journal replays exactly like the reporter's own counters.
#[derive(Clone, Debug)]
pub struct CertificateObservation<S: Scheme> {
    /// The aggregation height the certificate covers.
    pub height: u64,
    /// The certified digest — either a task's expected payload hash or
    /// `skip_digest(namespace, height)`. Skip heights are forwarded like any
    /// other; the consumer filters.
    pub digest: Digest,
    /// The scheme's certificate as assembled by the engine.
    pub certificate: S::Certificate,
}

pub type ObservationSender<S> = UnboundedSender<CertificateObservation<S>>;
pub type ObservationReceiver<S> = UnboundedReceiver<CertificateObservation<S>>;

/// Channel carrying certificate observations from the reporter to an
/// application-side consumer (see [`NodeReporter::with_certificate_tap`]).
pub fn observation_channel<S: Scheme>() -> (ObservationSender<S>, ObservationReceiver<S>) {
    tokio::sync::mpsc::unbounded_channel()
}

/// Actor that consumes reported activities.
pub struct NodeReporter<T: TaskData + PartialEq, S: Scheme> {
    mailbox: mailbox::Receiver<Report<S>>,
    /// Optional certificate tap: one [`CertificateObservation`] per certified
    /// height (first observation only — journal replays are deduplicated),
    /// skip-digest heights included. `None` unless the application opted in via
    /// [`Self::with_certificate_tap`].
    tap: Option<ObservationSender<S>>,
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
                tap: None,
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

    /// Opts in to certificate observation: every certified height — including
    /// skip-digest heights, which the consumer filters — is forwarded to `tap`
    /// exactly once (journal replays are deduplicated by height). A dropped
    /// receiver is logged at debug and never affects the reporter's own
    /// accounting.
    #[must_use]
    pub fn with_certificate_tap(mut self, tap: ObservationSender<S>) -> Self {
        self.tap = Some(tap);
        self
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
                    let digest = certificate.item.digest;
                    let skipped = digest == skip_digest(&self.namespace, height);
                    if skipped {
                        self.skipped_counter.inc();
                        debug!(height, "height certified as skipped");
                    } else {
                        info!(height, digest = ?digest, "height certified");
                    }
                    if let Some(tap) = &self.tap {
                        // Forwarded for every certified height, skip digests
                        // included — the consumer filters. A gone receiver only
                        // costs the observation, never the accounting above.
                        let observation = CertificateObservation {
                            height,
                            digest,
                            certificate: certificate.certificate,
                        };
                        if tap.send(observation).is_err() {
                            debug!(height, "certificate tap receiver gone; observation dropped");
                        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task_book::TaskBook;
    use bytes::{Buf, BufMut};
    use commonware_avs_core::bn254::{Bn254, Bn254Scheme, G1PublicKey, PublicKey};
    use commonware_codec::{EncodeSize, Error, Read, Write};
    use commonware_consensus::aggregation::types::{Ack, Certificate, Item};
    use commonware_consensus::types::{Epoch, Height};
    use commonware_cryptography::{Hasher, Sha256, Signer};
    use commonware_parallel::Sequential;
    use commonware_runtime::{Clock, Runner, Spawner, Supervisor, deterministic};
    use commonware_utils::TryCollect;
    use commonware_utils::ordered::Set;
    use std::time::Duration;

    /// Namespace bound into skip digests for every reporter under test.
    const NAMESPACE: &[u8] = b"reporter-test";

    /// Minimal task payload satisfying [`TaskData`] without any
    /// application-specific dependencies (the reporter never inspects it; it only
    /// types the TaskBook mailbox).
    #[derive(Debug, Clone, PartialEq)]
    struct TestTask;

    impl Write for TestTask {
        fn write(&self, _: &mut impl BufMut) {}
    }

    impl Read for TestTask {
        type Cfg = ();

        fn read_cfg(_: &mut impl Buf, _: &()) -> Result<Self, Error> {
            Ok(Self)
        }
    }

    impl EncodeSize for TestTask {
        fn encode_size(&self) -> usize {
            0
        }
    }

    /// Deterministic signer set with participant-ordered schemes, mirroring
    /// `core::bn254::scheme::tests::setup`.
    fn setup(n: usize) -> (Vec<Bn254Scheme>, Bn254Scheme) {
        let keys: Vec<Bn254> = (0..n).map(|i| Bn254::from_seed(i as u64)).collect();
        let participants: Set<PublicKey> = keys
            .iter()
            .map(|k| k.public_key())
            .try_collect()
            .expect("no duplicate keys");
        let g1_keys: Vec<G1PublicKey> = participants
            .iter()
            .map(|pk| {
                keys.iter()
                    .find(|k| &k.public_key() == pk)
                    .expect("participant derives from keys")
                    .public_g1()
            })
            .collect();

        let mut schemes: Vec<Bn254Scheme> = keys
            .iter()
            .map(|k| {
                Bn254Scheme::signer(participants.clone(), g1_keys.clone(), k.private_key())
                    .expect("key is in participant set")
            })
            .collect();
        schemes.sort_by_key(|s| s.me().expect("signer scheme has an index"));
        let verifier = Bn254Scheme::verifier(participants, g1_keys);
        (schemes, verifier)
    }

    fn item(height: u64, payload: &[u8]) -> Item<Digest> {
        let mut hasher = Sha256::new();
        hasher.update(payload);
        Item {
            height: Height::new(height),
            digest: hasher.finalize(),
        }
    }

    fn certify_item(
        schemes: &[Bn254Scheme],
        verifier: &Bn254Scheme,
        subject: Item<Digest>,
    ) -> Certificate<Bn254Scheme, Digest> {
        let acks: Vec<Ack<Bn254Scheme, Digest>> = schemes
            .iter()
            .map(|s| Ack::sign(s, Epoch::zero(), subject.clone()).expect("participant signs"))
            .collect();
        Certificate::from_acks(verifier, &acks, &Sequential).expect("quorum of acks")
    }

    fn certify(
        schemes: &[Bn254Scheme],
        verifier: &Bn254Scheme,
        height: u64,
        payload: &[u8],
    ) -> Certificate<Bn254Scheme, Digest> {
        certify_item(schemes, verifier, item(height, payload))
    }

    /// Drives a fresh `NodeReporter` (tap installed) and its TaskBook inside the
    /// deterministic runtime, handing `f` the context, the engine-side mailbox,
    /// the tap receiver, and the shared tip handle.
    fn with_reporter<F, Fut>(f: F)
    where
        F: FnOnce(
                deterministic::Context,
                ReporterMailbox<Bn254Scheme>,
                ObservationReceiver<Bn254Scheme>,
                Arc<AtomicU64>,
            ) -> Fut
            + Send
            + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let executor = deterministic::Runner::default();
        executor.start(|context| async move {
            let (task_book, task_book_mailbox) =
                TaskBook::<TestTask>::new(context.child("task_book"));
            context
                .child("task_book_actor")
                .spawn(move |_| task_book.run());
            let tip_handle = Arc::new(AtomicU64::new(0));
            let (tap, received) = observation_channel();
            let (reporter, mailbox) = NodeReporter::<TestTask, Bn254Scheme>::new(
                context.child("reporter"),
                task_book_mailbox,
                Arc::clone(&tip_handle),
                NAMESPACE.to_vec(),
            );
            let reporter = reporter.with_certificate_tap(tap);
            context
                .child("reporter_actor")
                .spawn(move |_| reporter.run());
            f(context, mailbox, received, tip_handle).await;
        });
    }

    #[test]
    fn certified_activity_reaches_tap() {
        let (schemes, verifier) = setup(4);
        let certificate = certify(&schemes, &verifier, 2, b"task payload");
        let skip_item = Item {
            height: Height::new(3),
            digest: skip_digest(NAMESPACE, 3),
        };
        let skip_certificate = certify_item(&schemes, &verifier, skip_item.clone());

        with_reporter(
            move |_context, mut mailbox, mut received, _tip| async move {
                mailbox.report(Activity::Certified(certificate.clone()));
                mailbox.report(Activity::Certified(skip_certificate.clone()));

                let observation = received.recv().await.expect("observation forwarded");
                assert_eq!(observation.height, 2);
                assert_eq!(observation.digest, certificate.item.digest);
                assert_eq!(observation.certificate, certificate.certificate);

                // Skip-digest heights are forwarded like any other; the consumer
                // filters.
                let skipped = received.recv().await.expect("skip observation forwarded");
                assert_eq!(skipped.height, 3);
                assert_eq!(skipped.digest, skip_item.digest);
                assert_eq!(skipped.certificate, skip_certificate.certificate);
            },
        );
    }

    #[test]
    fn replayed_certificate_not_resent() {
        let (schemes, verifier) = setup(4);
        let certificate = certify(&schemes, &verifier, 5, b"replayed task");
        let later = certify(&schemes, &verifier, 6, b"later task");

        with_reporter(
            move |_context, mut mailbox, mut received, _tip| async move {
                mailbox.report(Activity::Certified(certificate.clone()));
                // Journal replay after restart re-reports the same height.
                mailbox.report(Activity::Certified(certificate.clone()));
                mailbox.report(Activity::Certified(later.clone()));

                let first = received.recv().await.expect("first observation forwarded");
                assert_eq!(first.height, 5);
                // The mailbox and the tap are both FIFO, so the observation after
                // height 5 being height 6 proves the replay was not re-sent.
                let second = received.recv().await.expect("later observation forwarded");
                assert_eq!(second.height, 6);
                assert!(received.try_recv().is_err());
            },
        );
    }

    #[test]
    fn dropped_receiver_does_not_break_reporter() {
        let (schemes, verifier) = setup(4);
        let first = certify(&schemes, &verifier, 1, b"first task");
        let second = certify(&schemes, &verifier, 2, b"second task");

        with_reporter(
            move |context, mut mailbox, received, tip_handle| async move {
                drop(received);
                mailbox.report(Activity::Certified(first));
                mailbox.report(Activity::Certified(second));

                // No tap to synchronize on: poll the shared tip handle, which the
                // reporter raises after each certificate it accounts for.
                for _ in 0..100 {
                    if tip_handle.load(Ordering::Relaxed) >= 2 {
                        break;
                    }
                    context.sleep(Duration::from_millis(10)).await;
                }
                assert_eq!(tip_handle.load(Ordering::Relaxed), 2);
                // The counter advanced past the dead tap for both heights.
                let metrics = context.encode();
                assert!(
                    metrics.contains("certified_total 2"),
                    "expected certified_total 2 in:\n{metrics}"
                );
            },
        );
    }
}
