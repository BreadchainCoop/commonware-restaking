//! On-chain execution seam: the types and trait an application implements to turn a
//! [`crate::submitter::Submitter`]'s verified BLS submission into an actual
//! transaction.
//!
//! The submitter resolves a certified height into everything an EigenLayer
//! `BLSSignatureChecker`-style contract call needs (the message hash, quorum
//! numbers, the reference block, and the non-signer stakes/signature struct) but
//! has no idea which contract or ABI to call — that is application-specific. It
//! hands the assembled arguments to a [`BlsSignatureVerificationHandler`], which the
//! application implements against its own contract bindings.

use alloy_primitives::{Bytes, FixedBytes};
use anyhow::Result;
use commonware_avs_bindings::bls_sig_check_operator_state_retriever::{
    BN254::{G1Point, G2Point},
    IBLSSignatureCheckerTypes,
};
use commonware_avs_core::wire::TaskData;

/// Outcome of an on-chain verification transaction, reported back to the router so
/// it can decide whether the height executed successfully.
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub transaction_hash: String,
    pub block_number: Option<u64>,
    pub gas_used: Option<u64>,
    pub status: Option<bool>,
    pub contract_address: Option<String>,
}

/// Contract-specific handler for a certified task's on-chain BLS signature check.
///
/// The submitter has already resolved the certificate's signer bitmap to registered
/// operator keys, converted the aggregated signature to its on-chain point
/// representation, and fetched a `NonSignerStakesAndSignature` reading pinned to the
/// same block used for operator resolution. This trait is the seam where an
/// application submits (or simulates) the actual transaction against its own
/// contract bindings.
#[async_trait::async_trait]
pub trait BlsSignatureVerificationHandler: Send + Sync {
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
        quorum_numbers: Bytes,
        reference_block_number: u32,
        non_signer_data: IBLSSignatureCheckerTypes::NonSignerStakesAndSignature,
        task_data: Option<&Self::TaskData>,
    ) -> Result<ExecutionResult>;
}

/// Rebuilds the operator state retriever's non-signer stakes and signature struct
/// into the canonical BLS signature checker type.
///
/// Solidity's ABI code generation mints a nominally distinct Rust struct for every
/// binding module that references a shared interface type, even when the shape is
/// identical, so a value crossing from this crate's retriever binding into an
/// application's own contract binding (a separate `sol!` invocation entirely) needs
/// a field-by-field rebuild rather than a plain cast. Applications call this from
/// their [`BlsSignatureVerificationHandler::handle_verification`] implementation
/// before doing their own rebuild into their contract's binding type.
pub fn convert_non_signer_data(
    non_signer_data: IBLSSignatureCheckerTypes::NonSignerStakesAndSignature,
) -> IBLSSignatureCheckerTypes::NonSignerStakesAndSignature {
    IBLSSignatureCheckerTypes::NonSignerStakesAndSignature {
        nonSignerQuorumBitmapIndices: non_signer_data.nonSignerQuorumBitmapIndices,
        nonSignerPubkeys: non_signer_data
            .nonSignerPubkeys
            .into_iter()
            .map(|p| G1Point { X: p.X, Y: p.Y })
            .collect(),
        quorumApks: non_signer_data
            .quorumApks
            .into_iter()
            .map(|p| G1Point { X: p.X, Y: p.Y })
            .collect(),
        apkG2: G2Point {
            X: non_signer_data.apkG2.X,
            Y: non_signer_data.apkG2.Y,
        },
        sigma: G1Point {
            X: non_signer_data.sigma.X,
            Y: non_signer_data.sigma.Y,
        },
        quorumApkIndices: non_signer_data.quorumApkIndices,
        totalStakeIndices: non_signer_data.totalStakeIndices,
        nonSignerStakeIndices: non_signer_data.nonSignerStakeIndices,
    }
}
