//! NodeAutomaton: supplies the digest the aggregation engine signs per height.
//!
//! The engine calls [`commonware_consensus::Automaton::propose`] for every height
//! in its window; the returned oneshot resolves once the TaskBook decides what
//! lives at that height:
//!
//! - `Announce(task)`: validate via the configured [`ValidatorTrait`] and resolve
//!   the expected task digest. Errors (transient or deterministic) are retried
//!   with backoff within a configured budget; once the budget is exhausted the
//!   height resolves to `skip_digest(namespace, h)` instead — by then quorum has
//!   typically abandoned the height and is signing the same skip digest.
//! - `Skip`: resolve `skip_digest(namespace, h)` so the height still certifies and
//!   the pipeline advances.
//!
//! `verify` is never called by the aggregation engine (propose-only contract); it
//! resolves `true` trivially to satisfy the trait.

use commonware_avs_core::consensus::trivial_verify;
use commonware_avs_core::validator::ValidatorTrait;
use commonware_avs_core::wire::{TaskData, skip_digest};
use commonware_consensus::Automaton;
use commonware_consensus::types::Height;
use commonware_cryptography::sha256::Digest;
use commonware_runtime::{Clock, Metrics, Spawner};
use commonware_utils::channel::oneshot;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::task_book::{Resolution, TaskBookMailbox};

/// First retry delay after a validation error; doubles per attempt.
const INITIAL_RETRY_BACKOFF: Duration = Duration::from_millis(500);

/// Ceiling for the exponential retry backoff.
const MAX_RETRY_BACKOFF: Duration = Duration::from_secs(5);

/// Automaton for the aggregation engine (`Context = Height`).
///
/// Cheap to clone (the engine requires `Clone`): all state lives behind an `Arc`.
pub struct NodeAutomaton<T, E>
where
    T: TaskData + PartialEq,
    E: Spawner + Clock + Metrics + Send + Sync + 'static,
{
    inner: Arc<Inner<T, E>>,
}

impl<T, E> Clone for NodeAutomaton<T, E>
where
    T: TaskData + PartialEq,
    E: Spawner + Clock + Metrics + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

struct Inner<T, E>
where
    T: TaskData + PartialEq,
    E: Spawner + Clock + Metrics + Send + Sync + 'static,
{
    /// Runtime handle used to spawn one resolution task per proposed height.
    /// Children of this context abort with the root task, never individually.
    context: E,
    /// Source of per-height directives (fed by the router over channel 1).
    task_book: TaskBookMailbox<T>,
    /// Application hook that recomputes the expected digest for an announced task.
    validator: Arc<dyn ValidatorTrait<T>>,
    /// Application namespace bound into [`skip_digest`], matching the namespace
    /// the router and every other node use for the same deployment.
    namespace: Vec<u8>,
    /// Total time budget for retrying validation errors.
    retry_budget: Duration,
}

impl<T, E> NodeAutomaton<T, E>
where
    T: TaskData + PartialEq,
    E: Spawner + Clock + Metrics + Send + Sync + 'static,
{
    /// Creates the automaton.
    ///
    /// `retry_budget` bounds how long an announced task is retried on validation
    /// errors before the height resolves to its skip digest; wire it to the
    /// router's round timeout so the node gives up in lockstep with the router.
    pub fn new(
        context: E,
        task_book: TaskBookMailbox<T>,
        validator: Arc<dyn ValidatorTrait<T>>,
        namespace: Vec<u8>,
        retry_budget: Duration,
    ) -> Self {
        Self {
            inner: Arc::new(Inner {
                context,
                task_book,
                validator,
                namespace,
                retry_budget,
            }),
        }
    }
}

impl<T, E> Automaton for NodeAutomaton<T, E>
where
    T: TaskData + PartialEq,
    E: Spawner + Clock + Metrics + Send + Sync + 'static,
{
    type Context = Height;
    type Digest = Digest;

    async fn propose(&mut self, height: Height) -> oneshot::Receiver<Digest> {
        let (sender, receiver) = oneshot::channel();
        let inner = Arc::clone(&self.inner);
        // Detached: dropping the Handle does not abort the task; it runs until the
        // height resolves (or the TaskBook dies). Parked subscriptions for
        // unassigned heights are normal and hold no resources beyond the oneshot.
        drop(
            self.inner
                .context
                .child("propose")
                .spawn(move |_| async move {
                    inner.resolve(height, sender).await;
                }),
        );
        receiver
    }

    async fn verify(&mut self, _context: Height, _payload: Digest) -> oneshot::Receiver<bool> {
        // The aggregation engine never calls verify (it only requests digests via
        // propose); resolve trivially to satisfy the trait.
        trivial_verify()
    }
}

impl<T, E> Inner<T, E>
where
    T: TaskData + PartialEq,
    E: Spawner + Clock + Metrics + Send + Sync + 'static,
{
    /// Waits for the TaskBook's resolution of `height` and resolves the engine's
    /// digest request accordingly.
    async fn resolve(&self, height: Height, sender: oneshot::Sender<Digest>) {
        let h = height.get();
        let resolution = match self.task_book.subscribe(h).await {
            Ok(resolution) => resolution,
            Err(_) => {
                // TaskBook actor is gone (process shutting down). Drop the sender
                // so the engine records AppProposeCanceled instead of leaking a
                // forever-pending future.
                warn!(height = h, "task book unavailable; abandoning propose");
                return;
            }
        };

        let digest = match resolution {
            Resolution::Skip => {
                info!(height = h, "height skipped; signing skip digest");
                skip_digest(&self.namespace, h)
            }
            Resolution::Announce(task) => self.digest_for_announce(h, &task).await,
        };

        if sender.send(digest).is_err() {
            // The engine dropped the request (e.g. shutdown mid-resolution).
            debug!(height = h, "engine dropped digest request");
        }
    }

    /// Computes the expected digest for an announced task, retrying errors with
    /// backoff until `retry_budget` is spent, then falling back to the skip digest.
    ///
    /// The validator returns untyped (anyhow) errors, so transient failures (e.g.
    /// RPC hiccups) and deterministic validation failures are indistinguishable
    /// here; both are retried within the budget and both end in
    /// `skip_digest(namespace, h)`. That resolves the height without ever wedging
    /// on a flaky dependency, at the cost of waiting out the full budget on a
    /// deterministic failure.
    async fn digest_for_announce(&self, height: u64, task: &T) -> Digest {
        let deadline = self.context.current() + self.retry_budget;
        let mut backoff = INITIAL_RETRY_BACKOFF;
        loop {
            match self.validator.expected_digest(task).await {
                Ok(digest) => {
                    debug!(height, ?digest, "validated announced task");
                    return digest;
                }
                Err(error) if self.context.current() + backoff < deadline => {
                    debug!(
                        height,
                        %error,
                        backoff_ms = backoff.as_millis() as u64,
                        "task validation failed; retrying"
                    );
                    self.context.sleep(backoff).await;
                    backoff = (backoff * 2).min(MAX_RETRY_BACKOFF);
                }
                Err(error) => {
                    // Budget exhausted: treat as failed validation and sign the
                    // skip digest so the height can still certify.
                    warn!(
                        height,
                        %error,
                        budget_secs = self.retry_budget.as_secs_f64(),
                        "task validation budget exhausted; signing skip digest"
                    );
                    return skip_digest(&self.namespace, height);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{Buf, BufMut};
    use commonware_avs_core::validator::MockValidator;
    use commonware_avs_core::wire::TaskDirective;
    use commonware_codec::varint::UInt;
    use commonware_codec::{EncodeSize, Error, Read, ReadExt, Write};
    use commonware_runtime::{Runner, Supervisor, deterministic};

    use crate::task_book::TaskBook;

    const NAMESPACE: &[u8] = b"test-namespace";

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

    #[test]
    fn announce_resolves_to_validator_digest() {
        let executor = deterministic::Runner::default();
        executor.start(|context| async move {
            let (task_book, mailbox) = TaskBook::<TestTask>::new(context.child("task_book"));
            context.child("actor").spawn(move |_| task_book.run());

            let validator: Arc<dyn ValidatorTrait<TestTask>> =
                Arc::new(MockValidator::new_success());
            let mut automaton = NodeAutomaton::new(
                context.child("automaton"),
                mailbox.clone(),
                validator,
                NAMESPACE.to_vec(),
                Duration::from_secs(1),
            );

            let task = TestTask { id: 42 };
            mailbox.deliver(TaskDirective::Announce {
                height: 1,
                task: task.clone(),
            });

            let digest = automaton
                .propose(Height::new(1))
                .await
                .await
                .expect("propose resolved");
            assert_eq!(digest, MockValidator::digest_for(&task));
        });
    }

    #[test]
    fn skip_resolves_to_skip_digest() {
        let executor = deterministic::Runner::default();
        executor.start(|context| async move {
            let (task_book, mailbox) = TaskBook::<TestTask>::new(context.child("task_book"));
            context.child("actor").spawn(move |_| task_book.run());

            let validator = Arc::new(MockValidator::new_success());
            let mut automaton = NodeAutomaton::new(
                context.child("automaton"),
                mailbox.clone(),
                validator,
                NAMESPACE.to_vec(),
                Duration::from_secs(1),
            );

            mailbox.deliver(TaskDirective::<TestTask>::Skip { height: 7 });

            let digest = automaton
                .propose(Height::new(7))
                .await
                .await
                .expect("propose resolved");
            assert_eq!(digest, skip_digest(NAMESPACE, 7));
        });
    }

    #[test]
    fn exhausted_retry_budget_resolves_to_skip_digest() {
        let executor = deterministic::Runner::default();
        executor.start(|context| async move {
            let (task_book, mailbox) = TaskBook::<TestTask>::new(context.child("task_book"));
            context.child("actor").spawn(move |_| task_book.run());

            let validator = Arc::new(MockValidator::new_failure("always fails".to_string()));
            let mut automaton = NodeAutomaton::new(
                context.child("automaton"),
                mailbox.clone(),
                validator,
                NAMESPACE.to_vec(),
                // Tiny budget: the first retry check should already be past the
                // deadline, so the height resolves to skip quickly.
                Duration::from_millis(1),
            );

            let task = TestTask { id: 9 };
            mailbox.deliver(TaskDirective::Announce { height: 3, task });

            let digest = automaton
                .propose(Height::new(3))
                .await
                .await
                .expect("propose resolved");
            assert_eq!(digest, skip_digest(NAMESPACE, 3));
        });
    }
}
