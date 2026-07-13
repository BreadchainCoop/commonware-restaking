//! Submitter: turns certified heights into on-chain ECDSA verification calls.
//!
//! Consumes certificate observations from the [`crate::reporter::CertReporter`] and
//! resolves each one:
//!
//! - `skip_digest(namespace, height)`: the quorum abandoned the height — log and
//!   notify the sequencer ([`ResolutionKind::Skipped`]); nothing goes on-chain.
//! - No assignment, or a digest that matches neither the assignment nor the skip
//!   digest: pre-restart leftovers — log and notify ([`ResolutionKind::Foreign`]);
//!   the sequencer re-assigns any in-flight task.
//! - Expected digest: pair the certificate's signer bitmap with its per-signer
//!   ECDSA signatures, sort by ascending operator address (the order an on-chain
//!   verifier typically enforces to reject duplicate signers), pin a reference
//!   block, and call [`EcdsaVerificationHandler::handle_verification`], with
//!   bounded retries.
//!
//! Unlike the BN254 flow ([`crate::submitter`]), no registry reads are needed to
//! resolve signers: [`EcdsaScheme::signer_addresses`] maps the certificate's bitmap
//! straight to Ethereum addresses locally. The only on-chain read before submission
//! is therefore a single L1 `eth_blockNumber` call — the certificate itself already
//! carries the operators' 65-byte `r || s || v` signatures.

use crate::executor::ExecutionResult;
use crate::reporter::{CertifiedHeight, CertifiedReceiver};
use crate::sequencer::{Resolution, ResolutionKind, ResolutionSender, SharedAssignments};
use alloy_primitives::{Address, Bytes, FixedBytes};
use alloy_provider::Provider;
use anyhow::Result;
use commonware_avs_bindings::ReadOnlyProvider;
use commonware_avs_core::ecdsa::{EcdsaCertificate, EcdsaScheme};
use commonware_avs_core::wire::{TaskData, skip_digest};
use commonware_cryptography::sha256::Digest;
use std::time::Duration;
use tracing::{error, info, warn};

/// Retries after the first failed submission attempt.
const MAX_RETRIES: u32 = 2;

/// Delay before the first retry; doubles per attempt.
const INITIAL_RETRY_BACKOFF: Duration = Duration::from_secs(2);

/// Contract-specific handler for a certified task's on-chain ECDSA signature check.
///
/// The submitter has already resolved the certificate's signer bitmap to operator
/// addresses (ascending order) and paired each with its raw 65-byte signature. This
/// trait is the seam where an application submits (or simulates) the actual
/// transaction against its own contract bindings.
///
/// `reference_block_number` is the L1 block number observed via a single
/// `eth_blockNumber` call immediately before this handler runs. It is passed
/// through unmodified — it is NOT decremented here. A handler that simulates the
/// transaction with `eth_estimateGas` (which runs against the *current* block) and
/// needs a strictly-past block for a registry checkpoint lookup should compute
/// `reference_block_number - 1` itself, mirroring the split used by the
/// implementation this trait was ported from.
#[async_trait::async_trait]
pub trait EcdsaVerificationHandler: Send + Sync {
    /// The application's task payload type, carried by [`commonware_avs_core::wire::TaskDirective`].
    type TaskData: TaskData;

    /// Submits (or otherwise executes) the verification for `height`.
    ///
    /// `task_data` is `Some` for a task the submitter announced and `None` for a
    /// certified height with no known assignment (pre-restart leftovers observed
    /// purely from the certificate stream).
    async fn handle_verification(
        &mut self,
        height: u64,
        msg_hash: FixedBytes<32>,
        reference_block_number: u64,
        operators: Vec<Address>,
        signatures: Vec<Bytes>,
        task_data: Option<&Self::TaskData>,
    ) -> Result<ExecutionResult>;
}

/// Consumes certified heights and drives the on-chain submission.
pub struct Submitter<T, H>
where
    T: TaskData,
    H: EcdsaVerificationHandler<TaskData = T>,
{
    /// Verifier scheme: maps certificate signer bitmaps to operator addresses.
    scheme: EcdsaScheme,
    /// L1 read-side provider: supplies the reference block passed to the handler
    /// (operator state for an EigenLayer-style registry lives on L1 only).
    view_only_provider: ReadOnlyProvider,
    /// Executes the final on-chain verification transaction.
    handler: H,
    /// Height → expected digest + task, written by the sequencer.
    assignments: SharedAssignments<T>,
    /// Verified certificates from the reporter.
    certified: CertifiedReceiver<EcdsaScheme>,
    /// Final dispositions back to the sequencer.
    resolutions: ResolutionSender,
    /// Application namespace bound into [`skip_digest`], matching the namespace
    /// the sequencer uses for the same deployment.
    namespace: Vec<u8>,
}

impl<T, H> Submitter<T, H>
where
    T: TaskData,
    H: EcdsaVerificationHandler<TaskData = T>,
{
    pub fn new(
        scheme: EcdsaScheme,
        view_only_provider: ReadOnlyProvider,
        handler: H,
        assignments: SharedAssignments<T>,
        certified: CertifiedReceiver<EcdsaScheme>,
        resolutions: ResolutionSender,
        namespace: Vec<u8>,
    ) -> Self {
        Self {
            scheme,
            view_only_provider,
            handler,
            assignments,
            certified,
            resolutions,
            namespace,
        }
    }

    /// Runs until the reporter side of the certified channel closes.
    pub async fn run(mut self) {
        while let Some(certified) = self.certified.recv().await {
            self.handle_certified(certified).await;
        }
        info!("certified channel closed; submitter exiting");
    }

    /// Notifies the sequencer of a height's final disposition.
    fn notify(&self, height: u64, kind: ResolutionKind) {
        if self.resolutions.send(Resolution { height, kind }).is_err() {
            // The sequencer is gone; the process is shutting down.
            warn!(height, "resolution channel closed");
        }
    }

    async fn handle_certified(&mut self, certified: CertifiedHeight<EcdsaScheme>) {
        let CertifiedHeight {
            height,
            digest,
            certificate,
        } = certified;

        // Skip certificate: the quorum abandoned the height.
        if digest == skip_digest(&self.namespace, height) {
            info!(height, "skip certificate observed; nothing to submit");
            self.notify(height, ResolutionKind::Skipped);
            return;
        }

        // Look up the sequencer's assignment for this height.
        let assignment = match self.assignments.read() {
            Ok(assignments) => assignments.get(&height).cloned(),
            Err(_) => {
                error!(height, "assignments lock poisoned; treating as unassigned");
                None
            }
        };
        let Some(assignment) = assignment else {
            // Unassigned heights certify during journal replay (pre-restart
            // leftovers) or when nodes resolved a height from a previous router
            // life. There is nothing to submit — the task cache is gone.
            info!(
                height,
                digest = %digest,
                "certificate for unassigned height; nothing to submit"
            );
            self.notify(height, ResolutionKind::Foreign);
            return;
        };
        if assignment.digest != digest {
            // A certificate formed for a digest we did not announce (a pre-restart
            // leftover): the height is consumed, the in-flight task is not.
            warn!(
                height,
                expected = %assignment.digest,
                certified = %digest,
                "certificate digest does not match assignment; height consumed"
            );
            self.notify(height, ResolutionKind::Foreign);
            return;
        }

        // Expected digest: submit on-chain with bounded retries. Failures here
        // are either transient RPC issues (retry helps) or deterministic
        // rejections (retries burn two extra round-trips, then the height is
        // released as a failed execution).
        let mut backoff = INITIAL_RETRY_BACKOFF;
        let mut attempt = 0u32;
        loop {
            attempt += 1;
            match self
                .submit(height, digest, &certificate, &assignment.task)
                .await
            {
                Ok(result) => {
                    info!(
                        height,
                        tx = %result.transaction_hash,
                        block = ?result.block_number,
                        status = ?result.status,
                        "verification submitted"
                    );
                    let success = result.status.unwrap_or(true);
                    self.notify(height, ResolutionKind::Executed { success });
                    return;
                }
                Err(error) if attempt <= MAX_RETRIES => {
                    warn!(
                        height,
                        attempt,
                        max_retries = MAX_RETRIES,
                        %error,
                        backoff_secs = backoff.as_secs_f64(),
                        "submission failed; retrying"
                    );
                    tokio::time::sleep(backoff).await;
                    backoff *= 2;
                }
                Err(error) => {
                    error!(
                        height,
                        attempts = attempt,
                        %error,
                        "submission failed after retries; releasing height"
                    );
                    self.notify(height, ResolutionKind::Executed { success: false });
                    return;
                }
            }
        }
    }

    /// One end-to-end submission attempt for a certified task digest.
    async fn submit(
        &mut self,
        height: u64,
        digest: Digest,
        certificate: &EcdsaCertificate,
        task: &T,
    ) -> Result<ExecutionResult> {
        // Bitmap → operator addresses, in participant order. The certificate was
        // verified by the reporter, so the bitmap length matches the set and the
        // signature list is index-aligned with the bitmap's ascending iteration.
        let (operators, signatures) = ordered_signatures(&self.scheme, certificate)?;

        let msg_hash = FixedBytes::<32>::from_slice(digest.as_ref());

        // The reference block handed to the handler. Fetched once, unmodified —
        // see the handler trait doc for who is responsible for any `- 1` split.
        let reference_block_number = self
            .view_only_provider
            .get_block_number()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get block number: {}", e))?;

        info!(
            height,
            signers = signatures.len(),
            reference_block = reference_block_number,
            "submitting ECDSA quorum certificate"
        );

        self.handler
            .handle_verification(
                height,
                msg_hash,
                reference_block_number,
                operators,
                signatures,
                Some(task),
            )
            .await
    }
}

/// Pairs a verified certificate's signer bitmap with its signatures and returns
/// the raw 65-byte signatures sorted by ascending signer address — the order an
/// on-chain verifier typically enforces to reject duplicate signers.
///
/// Participant order (the certificate's signature order) is already ascending by
/// address bytes, so this sort is a no-op in practice; it stays as a defensive
/// guarantee of the on-chain calling convention rather than an assumption about
/// the participant set's ordering.
fn ordered_signatures(
    scheme: &EcdsaScheme,
    certificate: &EcdsaCertificate,
) -> Result<(Vec<Address>, Vec<Bytes>)> {
    let addresses = scheme.signer_addresses(&certificate.signers);
    if addresses.is_empty() || addresses.len() != certificate.signatures.len() {
        return Err(anyhow::anyhow!(
            "signer bitmap resolved to an inconsistent signer set ({} addresses / {} signatures)",
            addresses.len(),
            certificate.signatures.len()
        ));
    }

    let mut pairs: Vec<_> = addresses
        .into_iter()
        .zip(certificate.signatures.iter())
        .collect();
    pairs.sort_by_key(|(address, _)| *address);

    Ok(pairs
        .into_iter()
        .map(|(address, signature)| (address, Bytes::copy_from_slice(signature.as_ref())))
        .unzip())
}

#[cfg(test)]
mod tests {
    use super::*;
    use commonware_avs_core::ecdsa::{Ecdsa, PublicKey};
    use commonware_consensus::aggregation::types::Item;
    use commonware_consensus::types::Height;
    use commonware_cryptography::certificate::Scheme as _;
    use commonware_cryptography::{Hasher as _, Sha256, Signer as _};
    use commonware_parallel::Sequential;
    use commonware_utils::ordered::Set;
    use commonware_utils::{N3f1, TryCollect};

    /// Builds a 4-participant scheme set plus a verifier, participant-ordered.
    fn setup() -> (Vec<Ecdsa>, EcdsaScheme) {
        let keys: Vec<Ecdsa> = (0..4).map(Ecdsa::from_seed).collect();
        let participants: Set<PublicKey> = keys
            .iter()
            .map(|k| k.public_key())
            .try_collect()
            .expect("distinct keys");
        let verifier = EcdsaScheme::verifier(participants);
        (keys, verifier)
    }

    fn certificate_for(
        keys: &[Ecdsa],
        verifier: &EcdsaScheme,
        payload: &[u8],
    ) -> (EcdsaCertificate, [u8; 32]) {
        let mut hasher = Sha256::new();
        hasher.update(payload);
        let digest = hasher.finalize();
        let digest_bytes: [u8; 32] = digest.as_ref().try_into().unwrap();
        let subject = Item {
            height: Height::new(1),
            digest,
        };

        let schemes: Vec<EcdsaScheme> = keys
            .iter()
            .map(|k| EcdsaScheme::signer(verifier.participants().clone(), k.private_key()).unwrap())
            .collect();
        let attestations: Vec<_> = schemes
            .iter()
            .take(3)
            .map(|s| {
                s.sign::<commonware_cryptography::sha256::Digest>(&subject)
                    .unwrap()
            })
            .collect();
        let certificate = verifier
            .assemble::<_, N3f1>(attestations, &Sequential)
            .unwrap();
        (certificate, digest_bytes)
    }

    #[test]
    fn ordered_signatures_are_ascending_and_recover_to_operators() {
        let (keys, verifier) = setup();
        let (certificate, digest) = certificate_for(&keys, &verifier, b"submitter ordering");

        let (signers, signatures) = ordered_signatures(&verifier, &certificate).unwrap();
        assert_eq!(signers.len(), 3);
        assert_eq!(signatures.len(), 3);

        // Recovered signer addresses match the operator list positionally, are
        // strictly ascending, and every signer is one of the operators.
        let operators: Vec<Address> = verifier
            .participants()
            .iter()
            .map(|k| k.address())
            .collect();
        let mut last = Address::ZERO;
        for (operator, raw) in signers.iter().zip(&signatures) {
            let sig = commonware_avs_core::ecdsa::Signature::try_from(raw.as_ref()).unwrap();
            let recovered = sig.recover(&digest).unwrap();
            assert_eq!(recovered, *operator, "signature must be index-aligned");
            assert!(recovered > last, "signers must be strictly ascending");
            assert!(operators.contains(&recovered), "signer must be an operator");
            last = recovered;
        }
    }

    #[test]
    fn ordered_signatures_rejects_count_mismatch() {
        let (keys, verifier) = setup();
        let (mut certificate, _) = certificate_for(&keys, &verifier, b"count mismatch");
        certificate.signatures.pop();

        assert!(ordered_signatures(&verifier, &certificate).is_err());
    }
}
