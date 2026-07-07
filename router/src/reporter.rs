//! CertReporter actor: the router's sink for aggregation-engine activity.
//!
//! The verifier-only engine reports [`Activity::Certified`] once per assembled
//! certificate and [`Activity::Tip`] on every tip fast-forward — and, on restart,
//! REPLAYS every journaled activity before the network starts. The reporter must
//! therefore be non-blocking (`report()` enqueues to a mailbox that never drops)
//! and replay-idempotent (certificates are deduplicated by height; tips are folded
//! with `max`).
//!
//! Certified heights are defensively re-verified against the verifier scheme and
//! forwarded to the height's consumer (typically a [`crate::submitter::Submitter`])
//! over an mpsc channel. Replayed certificates from a previous router life are
//! forwarded too — the consumer finds no assignment for them and resolves them
//! without an on-chain call, which is exactly how the sequencer learns about
//! pre-restart heights.

use commonware_actor::{Feedback, mailbox};
use commonware_avs_core::bn254::{Bn254Certificate, Bn254Scheme};
use commonware_avs_core::consensus::PRUNE_SLACK;
use commonware_consensus::Reporter;
use commonware_consensus::aggregation::types::{Activity, Certificate};
use commonware_consensus::types::Height;
use commonware_cryptography::sha256::Digest;
use commonware_parallel::Sequential;
use commonware_runtime::Metrics;
use commonware_utils::NZUsize;
use commonware_utils::channel::oneshot;
use rand_core::CryptoRngCore;
use std::collections::{BTreeMap, VecDeque};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tracing::{debug, error, info, trace, warn};

/// A certificate observation handed to the height's consumer.
#[derive(Clone, Debug)]
pub struct CertifiedHeight {
    /// The aggregation height the certificate covers.
    pub height: u64,
    /// The certified digest — either a task's expected payload hash or
    /// `skip_digest(height)`.
    pub digest: Digest,
    /// The BN254 certificate (signer bitmap + aggregated G1 signature) as
    /// assembled by the engine and re-verified by this reporter.
    pub certificate: Bn254Certificate,
}

pub type CertifiedSender = UnboundedSender<CertifiedHeight>;
pub type CertifiedReceiver = UnboundedReceiver<CertifiedHeight>;

/// Channel carrying verified certificates from the reporter to their consumer.
pub fn certified_channel() -> (CertifiedSender, CertifiedReceiver) {
    tokio::sync::mpsc::unbounded_channel()
}

/// Mailbox capacity before messages spill to the unbounded overflow queue.
///
/// Overflow never drops: `report()` is called inline in the engine loop (and
/// during journal replay before the network starts) and must not lose activity.
const MAILBOX_CAPACITY: usize = 1024;

/// Messages processed by the [`CertReporter`] actor.
enum Message {
    /// A newly assembled (or replayed) certificate from the engine.
    Certified(Certificate<Bn254Scheme, Digest>),
    /// A tip fast-forward from the engine.
    Tip(Height),
    /// Query: the next height the engine needs a certificate for.
    GetTip(oneshot::Sender<u64>),
    /// Query: the certified digest for a height, if observed.
    Get(u64, oneshot::Sender<Option<Digest>>),
}

impl mailbox::Policy for Message {
    type Overflow = VecDeque<Self>;

    fn handle(overflow: &mut VecDeque<Self>, message: Self) {
        // Never drop: losing a Certified would wedge the sequencer on its
        // in-flight height and losing a query would leave a caller pending.
        overflow.push_back(message);
    }
}

/// Handle for reporting to and querying the [`CertReporter`] actor. Cheap to
/// clone; the clone handed to the engine is its `Reporter`.
#[derive(Clone)]
pub struct CertReporterMailbox {
    sender: mailbox::Sender<Message>,
}

impl Reporter for CertReporterMailbox {
    type Activity = Activity<Bn254Scheme, Digest>;

    fn report(&mut self, activity: Self::Activity) -> Feedback {
        match activity {
            Activity::Certified(certificate) => {
                self.sender.enqueue(Message::Certified(certificate))
            }
            Activity::Tip(height) => self.sender.enqueue(Message::Tip(height)),
            // Own acks are journaled and re-reported on replay, but a
            // verifier-only scheme never signs — tolerate and ignore.
            Activity::Ack(_) => Feedback::Ok,
        }
    }
}

impl CertReporterMailbox {
    /// Returns the engine's observed tip: the next height needing a certificate
    /// (max of reported tips and `certified height + 1`). `0` until the engine
    /// reports anything, or when the actor is gone (shutdown).
    pub async fn get_tip(&self) -> u64 {
        let (responder, receiver) = oneshot::channel();
        if !self.sender.enqueue(Message::GetTip(responder)).accepted() {
            warn!("cert reporter closed; tip query unanswered");
            return 0;
        }
        receiver.await.unwrap_or(0)
    }

    /// Returns the certified digest for `height`, if a certificate was observed
    /// (and not yet pruned). `None` when the actor is gone.
    pub async fn get(&self, height: u64) -> Option<Digest> {
        let (responder, receiver) = oneshot::channel();
        if !self
            .sender
            .enqueue(Message::Get(height, responder))
            .accepted()
        {
            warn!(height, "cert reporter closed; digest query unanswered");
            return None;
        }
        receiver.await.unwrap_or(None)
    }
}

/// Actor owning the certificate log. The context doubles as the RNG for the
/// defensive certificate re-verification.
pub struct CertReporter<E>
where
    E: Metrics + CryptoRngCore + Send + 'static,
{
    context: E,
    mailbox: mailbox::Receiver<Message>,
    /// Verifier scheme (`me() == None`) used to re-verify certificates.
    scheme: Bn254Scheme,
    /// Certified digest per height: dedupe (replay-idempotency) + query source.
    certified: BTreeMap<u64, Digest>,
    /// Next height the engine needs a certificate for (see [`CertReporterMailbox::get_tip`]).
    tip: u64,
    /// Verified certificates bound for the height's consumer.
    submit: CertifiedSender,
}

impl<E> CertReporter<E>
where
    E: Metrics + CryptoRngCore + Send + 'static,
{
    /// Creates the actor and its mailbox handle.
    ///
    /// `context` labels the mailbox metrics and supplies the verification RNG;
    /// `scheme` must be the same verifier instance the engine runs with.
    pub fn new(
        context: E,
        scheme: Bn254Scheme,
        submit: CertifiedSender,
    ) -> (Self, CertReporterMailbox) {
        let (sender, receiver) = mailbox::new(context.child("mailbox"), NZUsize!(MAILBOX_CAPACITY));
        (
            Self {
                context,
                mailbox: receiver,
                scheme,
                certified: BTreeMap::new(),
                tip: 0,
                submit,
            },
            CertReporterMailbox { sender },
        )
    }

    /// Runs until every mailbox handle is dropped.
    pub async fn run(mut self) {
        while let Some(message) = self.mailbox.recv().await {
            match message {
                Message::Certified(certificate) => self.handle_certified(certificate),
                Message::Tip(height) => self.advance_tip(height.get()),
                Message::GetTip(responder) => {
                    let _ = responder.send(self.tip);
                }
                Message::Get(height, responder) => {
                    let _ = responder.send(self.certified.get(&height).cloned());
                }
            }
        }
        info!("cert reporter mailbox closed; exiting");
    }

    fn handle_certified(&mut self, certificate: Certificate<Bn254Scheme, Digest>) {
        let height = certificate.item.height.get();
        let digest = certificate.item.digest;

        // Replay-idempotency: the engine re-reports every journaled certificate
        // on restart, and a certificate is unique per height (same digest
        // guaranteed by quorum intersection) — first observation wins.
        if let Some(existing) = self.certified.get(&height) {
            if *existing != digest {
                // Quorum equivocation across restarts — impossible under the
                // fault assumptions; surface loudly but keep the first.
                error!(
                    height,
                    existing = %existing,
                    conflicting = %digest,
                    "conflicting certificate digests for one height (BUG)"
                );
            } else {
                trace!(height, "duplicate certificate ignored");
            }
            return;
        }

        // Defensive re-verification with the verifier scheme. The engine already
        // verified every contributing ack, so a failure here means journal
        // corruption or a scheme mismatch: refuse to submit (the on-chain check
        // would reject it anyway) and keep the height unresolved so the wedge is
        // operator-visible via the sequencer's rebroadcast warnings.
        if !certificate.verify(&mut self.context, &self.scheme, &Sequential) {
            error!(
                height,
                digest = %digest,
                "certificate failed defensive verification; dropping (BUG)"
            );
            return;
        }

        debug!(height, digest = %digest, "certificate observed");
        self.certified.insert(height, digest);
        self.advance_tip(height + 1);

        // Forward to the height's consumer. A closed channel means it died — the
        // router cannot make progress without it.
        let certified = CertifiedHeight {
            height,
            digest,
            certificate: certificate.certificate,
        };
        if self.submit.send(certified).is_err() {
            error!(
                height,
                "certificate consumer channel closed; certificate dropped"
            );
        }
    }

    /// Folds a tip observation (monotonic max) and prunes old dedupe entries.
    fn advance_tip(&mut self, tip: u64) {
        if tip <= self.tip {
            return;
        }
        self.tip = tip;
        let floor = tip.saturating_sub(PRUNE_SLACK);
        // `split_off` keeps entries >= floor.
        let kept = self.certified.split_off(&floor);
        self.certified = kept;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use commonware_avs_core::bn254::{Bn254, G1PublicKey, PublicKey};
    use commonware_consensus::aggregation::types::{Ack, Item};
    use commonware_consensus::types::Epoch;
    use commonware_cryptography::certificate::Scheme;
    use commonware_cryptography::{Hasher, Sha256, Signer};
    use commonware_runtime::{Runner, Spawner, Supervisor, deterministic};
    use commonware_utils::TryCollect;
    use commonware_utils::ordered::Set;
    use std::future::Future;

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

    fn certify(
        schemes: &[Bn254Scheme],
        verifier: &Bn254Scheme,
        height: u64,
        payload: &[u8],
    ) -> Certificate<Bn254Scheme, Digest> {
        let subject = item(height, payload);
        let acks: Vec<Ack<Bn254Scheme, Digest>> = schemes
            .iter()
            .map(|s| Ack::sign(s, Epoch::zero(), subject.clone()).expect("participant signs"))
            .collect();
        Certificate::from_acks(verifier, &acks, &Sequential).expect("quorum of acks")
    }

    /// Drives a fresh `CertReporter` inside the deterministic runtime, handing the
    /// mailbox and a raw `CertifiedReceiver` to `f`.
    fn with_reporter<F, Fut>(scheme: Bn254Scheme, f: F)
    where
        F: FnOnce(CertReporterMailbox, CertifiedReceiver) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let executor = deterministic::Runner::default();
        executor.start(|context| async move {
            let (submit, received) = certified_channel();
            let (reporter, mailbox) = CertReporter::new(context.child("reporter"), scheme, submit);
            context.child("actor").spawn(move |_| reporter.run());
            f(mailbox, received).await;
        });
    }

    #[test]
    fn duplicate_certificate_is_deduped() {
        let (schemes, verifier) = setup(4);
        let certificate = certify(&schemes, &verifier, 3, b"task payload");

        with_reporter(verifier, |mailbox, mut received| async move {
            let mut reporter_side = mailbox.clone();
            reporter_side.report(Activity::Certified(certificate.clone()));
            reporter_side.report(Activity::Certified(certificate.clone()));

            let first = received.recv().await.expect("first certificate forwarded");
            assert_eq!(first.height, 3);
            assert_eq!(first.digest, certificate.item.digest);

            // The duplicate is deduped by the reporter and never forwarded again;
            // the tip query below (which only answers after both reports are
            // processed, since the mailbox is FIFO per sender) confirms the actor
            // did not wedge on the duplicate.
            assert_eq!(mailbox.get_tip().await, 4);
            assert!(received.try_recv().is_err());
        });
    }

    #[test]
    fn tip_query_reflects_certified_height_plus_one() {
        let (schemes, verifier) = setup(4);
        let certificate = certify(&schemes, &verifier, 10, b"tip task");

        with_reporter(verifier, |mailbox, _received| async move {
            assert_eq!(mailbox.get_tip().await, 0);
            let mut reporter_side = mailbox.clone();
            reporter_side.report(Activity::Certified(certificate));
            assert_eq!(mailbox.get_tip().await, 11);
        });
    }

    #[test]
    fn get_returns_certified_digest_for_height() {
        let (schemes, verifier) = setup(4);
        let certificate = certify(&schemes, &verifier, 5, b"get task");
        let expected_digest = certificate.item.digest;

        with_reporter(verifier, move |mailbox, _received| async move {
            assert_eq!(mailbox.get(5).await, None);
            let mut reporter_side = mailbox.clone();
            reporter_side.report(Activity::Certified(certificate));
            assert_eq!(mailbox.get(5).await, Some(expected_digest));
            assert_eq!(mailbox.get(6).await, None);
        });
    }

    /// The defensive re-verification path: a certificate whose bitmap is tampered
    /// after assembly fails `Certificate::verify` and must be dropped rather than
    /// forwarded to the height's consumer.
    #[test]
    fn tampered_certificate_is_dropped_not_forwarded() {
        let (schemes, verifier) = setup(4);
        let mut certificate = certify(&schemes, &verifier, 8, b"tamper task");
        // Corrupt the aggregated signature so the pairing check fails, keeping the
        // signer bitmap (and therefore the decoded shape) intact.
        certificate.certificate.signature = schemes[0]
            .sign::<Digest>(&item(8, b"a different payload"))
            .expect("signer can sign")
            .signature
            .get()
            .cloned()
            .expect("attestation carries a decodable signature");

        with_reporter(verifier, |mailbox, mut received| async move {
            let mut reporter_side = mailbox.clone();
            reporter_side.report(Activity::Certified(certificate));

            // Nothing is forwarded to the consumer...
            assert_eq!(mailbox.get_tip().await, 0);
            assert!(received.try_recv().is_err());
            // ...and the height stays unresolved (no digest recorded).
            assert_eq!(mailbox.get(8).await, None);
        });
    }
}
