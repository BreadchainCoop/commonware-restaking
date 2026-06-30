# Commonware migration: `0.0.63` → `2026.5.0`

Commonware switched from `0.0.x` to CalVer and re-architected the crypto stack onto a
generic algebra foundation. New crates appear in the tree: `commonware-math`,
`commonware-formatting`, `commonware-parallel`, `commonware-actor`. This document lists
every change that affects this workspace and how each was resolved.

## 1. `Signer` / `Verifier`: namespace is now mandatory

`namespace` changed from `Option<&[u8]>` to `&[u8]`. "No namespace" is now the empty slice.
Upstream rationale: a namespace must always be present to prevent cross-domain signature reuse.

```diff
- fn sign(&self, namespace: Option<&[u8]>, msg: &[u8]) -> Self::Signature;
+ fn sign(&self, namespace: &[u8], msg: &[u8]) -> Self::Signature;
- fn verify(&self, namespace: Option<&[u8]>, msg: &[u8], sig: &Self::Signature) -> bool;
+ fn verify(&self, namespace: &[u8], msg: &[u8], sig: &Self::Signature) -> bool;
```

- Trait impls in `core/src/bn254/mod.rs` updated; `namespace.is_none()` → `namespace.is_empty()`.
- All call sites passing `None` now pass `&[]` (benches, executor, contributor, tests).

## 2. `Signer` now requires the `Random` supertrait

`Signer: Random + Send + Sync + Clone + 'static`, where `Random` lives in
`commonware_math::algebra`. It is a normal `pub trait` — the "sealed / not accessible"
compiler message only means `commonware-math` was not a direct dependency.

- Added `commonware-math` and `rand_core` (0.6, matching commonware) as direct deps.
- Implemented `Random for Bn254` (samples a scalar, derives the keypair). This is additive.

## 3. `hex` moved out of `commonware-utils`

`commonware_utils::hex` was removed; the function now lives in `commonware-formatting`.
We use the external `hex` crate instead (`hex::encode`), aliased to keep call sites unchanged.

## 4. `commonware_utils::set` → `commonware_utils::ordered`

`set::OrderedAssociated<K, V>` → `ordered::Map<K, V>` (and `Ordered<K>` → `Set<K>`).
Construction now uses `from_iter_dedup` instead of `FromIterator`. Affects the counter examples only.

## 5. p2p `Sender` / `Receiver` redesign

- `Sender` split into `LimitedSender` (rate-limit `check` → `CheckedSender`) + a blanket
  `Sender` on top. `Sender::send` is now **synchronous** and returns `Vec<PublicKey>`
  (the recipients it will attempt) instead of a `Result` future. Removed `.await` and
  error handling at all broadcast sites.
- Messages and received payloads moved from `bytes::Bytes` to `commonware_runtime::IoBuf`
  (`Message<P> = (P, IoBuf)`). `Bytes: Into<IoBufs>` still holds, so send args are
  unchanged; `IoBuf: AsRef<[u8]> + Buf`, so decode via `Cursor::new(msg)` is unchanged.
- `authenticated::lookup` changes (examples only): `Config::attempt_unregistered_handshakes`
  **renamed** to `bypass_ip_check` (same semantics — let known peers connect from unexpected
  source IPs; load-bearing for the K8s router deployment, so kept set to `true`), `Oracle::update`
  removed, and `Network::register` rate-limiting arity changed.

## 6. Runtime metrics & context redesign

- `Metrics::register(name, help, raw_metric)` now **returns** a `Registered<M>` handle that
  must be retained (dropping it unregisters). Old pattern (`let m = Counter::default();
  ctx.register(.., m.clone())`) replaced by `let m = ctx.register(.., raw::Counter::default())`.
- Convenience aliases: `metrics::{Counter, Histogram} = Registered<raw::{Counter,Histogram}>`;
  status counters use `status::Raw::default()`. `status::CounterExt` removed (methods inherent).
- Context scoping `with_label("x")` → `Supervisor::child("x")`. `Metrics: Supervisor` now,
  so any custom context (e.g. test `MockClock`) must also implement `Supervisor`.

## 7. Dependencies: `governor` 0.6 → 0.10, `prometheus-client` 0.22 → 0.24, new `commonware-actor`

- commonware-p2p pulls `governor` 0.10; bumped the workspace pin to avoid two versions in the
  graph (the `Quota`/rate-limit types in the p2p API must match). Note: `commonware_runtime::Clock`
  now requires `governor::clock::Clock<Instant = SystemTime> + ReasonablyRealtime` as supertraits,
  so the test `MockClock` implements those (trivial: `now()` + an empty `ReasonablyRealtime` impl).
- commonware-runtime re-exports `prometheus-client` 0.24; bumped the workspace pin to match.
  Behavioral change: 0.24 suppresses the descriptor of a metric **family** with no samples, so an
  unobserved `status::Counter` family no longer appears in `encode()` until its first observation
  (one orchestrator metrics test expectation updated accordingly).
- Added `commonware-actor` (dev-dependency) — `CheckedSender::send` returns
  `commonware_actor::Unreliable<Feedback>`, needed by the test `MockSender`.

## 8. `authenticated::lookup` peer/config API (examples)

- `Config::local(crypto, namespace, listen, max_message_size)` — dropped the separate local-listen
  address; `max_message_size` is now `u32`. The `attempt_unregistered_handshakes` field was renamed
  to `bypass_ip_check` (same meaning); the router example keeps it `true` for K8s.
- Peer registration: `oracle.update(id, OrderedAssociated).await` → `oracle.track(id, Map<PublicKey, Address>)`
  (synchronous, via the `AddressableManager` trait; `SocketAddr` wraps into `p2p::Address`).
- The runtime context (`tokio::Context`) is no longer `Clone`; derive owned sub-contexts with
  `Supervisor::child(...)` instead.

## Public API impact

None forced. The only public surface that *must* change is the commonware trait method
signatures themselves (upstream contract). Our own `aggregate_verify` keeps its
`Option<&[u8]>` signature and converts internally, so downstream callers are unaffected.
The `Random for Bn254` impl is purely additive.
