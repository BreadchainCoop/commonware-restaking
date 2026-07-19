//! Shared pieces of the Solana counter example (peer of `counter-common`):
//! the task payload, the digest formula both sides recompute, and the key /
//! router-connection file formats.
//!
//! Runtime tuning knobs are re-exported from the EVM example's common crate —
//! they are chain-agnostic (journal directories, p2p quotas, engine windows).

pub mod keys;
pub mod types;
pub mod validator;

pub use counter_common::config::{
    ack_messages_per_second, agg_activity_timeout, agg_window, p2p_message_backlog,
    p2p_quota_period, rebroadcast_interval, round_timeout, storage_directory,
};
pub use keys::{RouterConnection, load_bn254_key};
pub use types::RoundTaskData;
pub use validator::{RoundValidator, expected_digest};

/// Application namespace for the Solana counter example: the p2p handshake
/// domain and the `skip_digest` binding. Distinct from the EVM counter
/// example's namespace so certificates and handshakes can never cross
/// deployments.
pub const APPLICATION_NAMESPACE: &[u8] = b"_COMMONWARE_AGGREGATION_SOLANA_";
