use alloy_primitives::Address;
use commonware_avs_eigenlayer::AvsDeployment;
use std::env;
use std::fs;

pub struct CounterDeployment {
    inner: AvsDeployment,
}

impl CounterDeployment {
    pub fn load() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let inner = AvsDeployment::load()?;
        Ok(Self { inner })
    }

    pub fn counter_address(&self) -> Result<Address, Box<dyn std::error::Error + Send + Sync>> {
        self.inner.custom_address("counter")
    }

    pub fn registry_coordinator_address(
        &self,
    ) -> Result<Address, Box<dyn std::error::Error + Send + Sync>> {
        self.inner.registry_coordinator_address()
    }

    pub fn bls_apk_registry_address(
        &self,
    ) -> Result<Address, Box<dyn std::error::Error + Send + Sync>> {
        self.inner.bls_apk_registry_address()
    }

    pub fn bls_sig_check_operator_state_retriever_address(
        &self,
    ) -> Result<Address, Box<dyn std::error::Error + Send + Sync>> {
        self.inner.bls_sig_check_operator_state_retriever_address()
    }
}

/// Default P2P channel message backlog depth.
///
/// The backlog bounds how many queued messages the channel will hold before the sender
/// blocks or drops new messages. Configurable at runtime via `P2P_MESSAGE_BACKLOG`.
pub const DEFAULT_P2P_MESSAGE_BACKLOG: usize = 256;

/// Default P2P channel rate limit in messages per second.
///
/// Configurable at runtime via `P2P_MESSAGES_PER_SECOND`. Accepts fractional values
/// (e.g. `0.5` for one message every two seconds).
pub const DEFAULT_P2P_MESSAGES_PER_SECOND: f64 = 1.0;

/// Reads the P2P channel backlog depth from `P2P_MESSAGE_BACKLOG`, defaulting to
/// [`DEFAULT_P2P_MESSAGE_BACKLOG`].
pub fn p2p_message_backlog() -> usize {
    env::var("P2P_MESSAGE_BACKLOG")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|&v: &usize| v > 0)
        .unwrap_or(DEFAULT_P2P_MESSAGE_BACKLOG)
}

/// Reads the P2P channel rate limit from `P2P_MESSAGES_PER_SECOND` and returns the
/// per-message quota period (`1 / rate`), defaulting to
/// [`DEFAULT_P2P_MESSAGES_PER_SECOND`] when unset or invalid.
///
/// The quota is a smooth rate with no burst allowance: a rate of `5.0` permits one
/// message every 200 ms, not bursts of five. Values whose reciprocal would overflow
/// a `Duration` (e.g. `1e-20`) or round below its 1 ns resolution (e.g. `3e9`) are
/// treated as invalid and fall back to the default.
pub fn p2p_quota_period() -> std::time::Duration {
    parse_p2p_quota_period(env::var("P2P_MESSAGES_PER_SECOND").ok().as_deref())
}

/// Parses a `P2P_MESSAGES_PER_SECOND` value into a quota period, falling back to the
/// default rate on malformed, non-positive, non-finite, or non-representable input
/// (including `Duration` overflow and sub-nanosecond reciprocals that round to zero).
fn parse_p2p_quota_period(value: Option<&str>) -> std::time::Duration {
    value
        .and_then(|v| v.trim().parse::<f64>().ok())
        .filter(|&v| v > 0.0 && v.is_finite())
        .and_then(|v| std::time::Duration::try_from_secs_f64(1.0 / v).ok())
        .filter(|d| !d.is_zero())
        .unwrap_or_else(|| {
            std::time::Duration::from_secs_f64(1.0 / DEFAULT_P2P_MESSAGES_PER_SECOND)
        })
}

/// Default storage directory for the aggregation engine's journal.
///
/// Matches the writable data volume mounted in the container images. Journal
/// persistence across restarts requires a stable path — the commonware tokio
/// runtime otherwise defaults to a random per-process temp dir.
pub const DEFAULT_STORAGE_DIRECTORY: &str = "/app/data";

/// Resolves the storage directory for the engine journal.
///
/// Reads `STORAGE_DIR`; when unset, uses [`DEFAULT_STORAGE_DIRECTORY`] if it is
/// (creatable and) writable, else falls back to `$TMPDIR/counter-avs` for
/// bare-metal dev runs. The fallback is per-boot on most systems, so journal
/// replay across restarts is only guaranteed when `STORAGE_DIR` or the default
/// volume exists.
pub fn storage_directory() -> std::path::PathBuf {
    if let Ok(dir) = env::var("STORAGE_DIR") {
        let dir = dir.trim();
        if !dir.is_empty() {
            return std::path::PathBuf::from(dir);
        }
    }
    let default = std::path::Path::new(DEFAULT_STORAGE_DIRECTORY);
    if directory_is_writable(default) {
        return default.to_path_buf();
    }
    std::env::temp_dir().join("counter-avs")
}

/// Whether `path` exists (or can be created) and accepts file writes.
///
/// Probes with a real file create/delete rather than metadata: permission bits do
/// not capture read-only mounts or ACLs.
fn directory_is_writable(path: &std::path::Path) -> bool {
    if fs::create_dir_all(path).is_err() {
        return false;
    }
    let probe = path.join(format!(".counter-avs-write-probe-{}", std::process::id()));
    match fs::write(&probe, b"") {
        Ok(()) => {
            let _ = fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

/// Default number of heights the aggregation engine works on concurrently above
/// its tip (`Config::window`).
pub const DEFAULT_AGG_WINDOW: u64 = 8;

/// Reads the aggregation engine window from `AGG_WINDOW`, defaulting to
/// [`DEFAULT_AGG_WINDOW`]. Zero or unparseable values fall back to the default
/// (the engine requires a non-zero window).
pub fn agg_window() -> std::num::NonZeroU64 {
    env::var("AGG_WINDOW")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .and_then(std::num::NonZeroU64::new)
        .unwrap_or_else(|| {
            std::num::NonZeroU64::new(DEFAULT_AGG_WINDOW).expect("default window is non-zero")
        })
}

/// Default number of heights the aggregation engine keeps tracking below its tip
/// (`Config::activity_timeout`): ack collection + prune buffer.
///
/// Must be generous — heights pruned past this window can never certify locally,
/// so the router would miss their certificates (see the liveness model).
pub const DEFAULT_AGG_ACTIVITY_TIMEOUT: u64 = 256;

/// Reads the aggregation activity timeout (in heights) from `AGG_ACTIVITY_TIMEOUT`,
/// defaulting to [`DEFAULT_AGG_ACTIVITY_TIMEOUT`].
pub fn agg_activity_timeout() -> u64 {
    env::var("AGG_ACTIVITY_TIMEOUT")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(DEFAULT_AGG_ACTIVITY_TIMEOUT)
}

/// Default time the router waits for a certificate on its assigned height before
/// broadcasting `Skip` for it.
pub const DEFAULT_ROUND_TIMEOUT_SECS: f64 = 30.0;

/// Reads the round timeout from `ROUND_TIMEOUT` (seconds, fractional allowed),
/// defaulting to [`DEFAULT_ROUND_TIMEOUT_SECS`]. Non-positive, non-finite, or
/// unparseable values fall back to the default.
pub fn round_timeout() -> std::time::Duration {
    parse_secs_env_duration(
        env::var("ROUND_TIMEOUT").ok().as_deref(),
        DEFAULT_ROUND_TIMEOUT_SECS,
    )
}

/// Default cadence at which the router re-broadcasts the current `TaskDirective`
/// until the height certifies. Also reused as the engine's own ack
/// `rebroadcast_timeout`.
pub const DEFAULT_REBROADCAST_INTERVAL_SECS: f64 = 5.0;

/// Reads the rebroadcast interval from `REBROADCAST_INTERVAL` (seconds, fractional
/// allowed), defaulting to [`DEFAULT_REBROADCAST_INTERVAL_SECS`]. Non-positive,
/// non-finite, or unparseable values fall back to the default.
pub fn rebroadcast_interval() -> std::time::Duration {
    parse_secs_env_duration(
        env::var("REBROADCAST_INTERVAL").ok().as_deref(),
        DEFAULT_REBROADCAST_INTERVAL_SECS,
    )
}

/// Per-peer send/receive rate for the aggregation-engine ack channel (channel 0),
/// in messages per second.
///
/// The engine keeps rebroadcasting a signed height's ack every
/// `REBROADCAST_INTERVAL` until the height falls `AGG_ACTIVITY_TIMEOUT` below the
/// tip — even after it certifies — so steady-state demand approaches
/// `AGG_ACTIVITY_TIMEOUT / REBROADCAST_INTERVAL` messages per second per peer. The
/// p2p send-side limiter SILENTLY DROPS messages to rate-limited peers, so an
/// undersized quota starves fresh acks and stalls certification. The default is
/// computed from those two knobs with 2x headroom; override with
/// `P2P_ACK_MESSAGES_PER_SECOND` — distinct from `P2P_MESSAGES_PER_SECOND`, which
/// governs only the task-directive channel.
pub fn ack_messages_per_second() -> std::num::NonZeroU32 {
    if let Some(v) = env::var("P2P_ACK_MESSAGES_PER_SECOND")
        .ok()
        .and_then(|v| v.trim().parse::<u32>().ok())
        .and_then(std::num::NonZeroU32::new)
    {
        return v;
    }
    let demand =
        (agg_activity_timeout() as f64 / rebroadcast_interval().as_secs_f64()).ceil() as u32;
    std::num::NonZeroU32::new(demand.saturating_mul(2).saturating_add(8).max(8))
        .expect("quota is always at least 8")
}

/// Parses a seconds value (fractional allowed) into a `Duration`, falling back to
/// `default_secs` on malformed, non-positive, non-finite, or non-representable input.
fn parse_secs_env_duration(value: Option<&str>, default_secs: f64) -> std::time::Duration {
    value
        .and_then(|v| v.trim().parse::<f64>().ok())
        .filter(|&v| v > 0.0 && v.is_finite())
        .and_then(|v| std::time::Duration::try_from_secs_f64(v).ok())
        .filter(|d| !d.is_zero())
        .unwrap_or_else(|| std::time::Duration::from_secs_f64(default_secs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn p2p_quota_period_default_is_one_per_second() {
        assert_eq!(parse_p2p_quota_period(None), Duration::from_secs(1));
    }

    #[test]
    fn p2p_quota_period_converts_rate_to_period() {
        assert_eq!(
            parse_p2p_quota_period(Some("5.0")),
            Duration::from_millis(200)
        );
        assert_eq!(parse_p2p_quota_period(Some("0.5")), Duration::from_secs(2));
    }

    #[test]
    fn p2p_quota_period_rejects_invalid_values() {
        let default = Duration::from_secs(1);
        assert_eq!(parse_p2p_quota_period(Some("")), default);
        assert_eq!(parse_p2p_quota_period(Some("abc")), default);
        assert_eq!(parse_p2p_quota_period(Some("0")), default);
        assert_eq!(parse_p2p_quota_period(Some("-1.5")), default);
        assert_eq!(parse_p2p_quota_period(Some("inf")), default);
        assert_eq!(parse_p2p_quota_period(Some("NaN")), default);
    }

    #[test]
    fn p2p_quota_period_rejects_duration_overflow() {
        // 1.0 / 1e-20 overflows Duration; must fall back to the default, not panic.
        assert_eq!(
            parse_p2p_quota_period(Some("1e-20")),
            Duration::from_secs(1)
        );
    }

    #[test]
    fn p2p_quota_period_rejects_excessive_rate() {
        // 1.0 / 3e9 rounds below 1 ns and becomes Duration::ZERO; must fall back to default.
        assert_eq!(parse_p2p_quota_period(Some("3e9")), Duration::from_secs(1));
    }

    #[test]
    fn secs_env_duration_parses_and_falls_back() {
        assert_eq!(
            parse_secs_env_duration(Some("45"), 30.0),
            Duration::from_secs(45)
        );
        assert_eq!(
            parse_secs_env_duration(Some("0.5"), 30.0),
            Duration::from_millis(500)
        );
        let default = Duration::from_secs(30);
        assert_eq!(parse_secs_env_duration(None, 30.0), default);
        assert_eq!(parse_secs_env_duration(Some(""), 30.0), default);
        assert_eq!(parse_secs_env_duration(Some("abc"), 30.0), default);
        assert_eq!(parse_secs_env_duration(Some("0"), 30.0), default);
        assert_eq!(parse_secs_env_duration(Some("-3"), 30.0), default);
        assert_eq!(parse_secs_env_duration(Some("inf"), 30.0), default);
        assert_eq!(parse_secs_env_duration(Some("NaN"), 30.0), default);
    }

    #[test]
    fn storage_directory_falls_back_to_writable_path() {
        // Regardless of environment, the resolved directory must be usable for the
        // engine journal (env override, default volume, or temp fallback).
        let dir = storage_directory();
        assert!(!dir.as_os_str().is_empty());
    }

    #[test]
    fn agg_defaults_are_sane() {
        assert_eq!(DEFAULT_AGG_WINDOW, 8);
        assert_eq!(DEFAULT_AGG_ACTIVITY_TIMEOUT, 256);
        // The default window must construct the NonZeroU64 the engine config needs.
        assert_eq!(agg_window().get(), DEFAULT_AGG_WINDOW);
    }
}
