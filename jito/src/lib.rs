//! Jito/Solana backend for the commonware-restaking chassis — the additive
//! peer of the `eigenlayer` crate (INTERFACES.md §5; zero changes to the EVM
//! path).
//!
//! - [`bn254`]: key/signature wrappers in the NCN program's signature domain
//!   (`ncn-program-core` `Sha256Normalized` hash-to-curve, Solana compressed
//!   point wire formats).
//! - [`scheme`]: [`scheme::JitoBn254Scheme`], a
//!   `commonware_cryptography::certificate::Scheme` whose certificates carry
//!   exactly the `VerifyCertificate` wire triple.
//! - [`config`]: the NCN deployment JSON (analog of `avs_deploy.json`).
//! - [`network`]: quorum discovery from `NCNOperatorAccount` +
//!   `Snapshot` PDAs at `confirmed` commitment.
//! - [`quorum`]: startup reconciliation between the engine's count-based N3f1
//!   quorum and the program's stake-weighted threshold.
//! - [`instruction`]: manual `VerifyCertificate` instruction construction
//!   (frozen §2 shape; discriminator pinned byte-for-byte against the Phase 1
//!   `NCNProgramInstruction` enum on `main`).
//! - [`submitter`]: [`submitter::JitoSubmitter`] + the
//!   [`submitter::SolanaCertificateHandler`] seam (peer of the EVM
//!   `BlsSignatureVerificationHandler`), with finalized-only resolutions and
//!   blockhash-expiry rebuild.

pub mod bn254;
pub mod config;
pub mod instruction;
pub mod network;
pub mod quorum;
pub mod scheme;
pub mod settle;
pub mod submitter;

pub use config::NcnDeployment;
pub use network::{JitoOperatorInfo, JitoQuorum, JitoStakingClient};
pub use scheme::{JitoBn254Scheme, JitoCertificate};
pub use settle::SettleCertificateHandler;
pub use submitter::{
    CertificateSubmission, JitoSubmitter, SolanaCertificateHandler, SolanaExecutionResult,
    VerifyCertificateHandler,
};
