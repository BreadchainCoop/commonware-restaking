# BLS Aggregation Benchmarks

Criterion benchmarks covering the three functions on the critical path between
threshold check and `execute_verification`:

- **`aggregate_signatures`** — O(N) G1 projective additions + one `into_affine`
- **`get_points`** — three independent O(N) projective-addition loops (G1 keys, G2 keys,
  signatures), each followed by `into_affine`; called by the BLS executor
- **`aggregate_verify`** — O(N) G2 projective additions + `into_affine` + two Miller-loop
  pairings; the pairing cost is fixed regardless of N

## How to run

```
cargo bench -p commonware-avs-core
```

HTML reports land in `target/criterion/`. Re-run after any change to `core/src/bn254/mod.rs`
to detect regressions before merging.

## How to read these results

Absolute wall times depend on hardware, toolchain, and commit, so treat the numbers
below as a representative snapshot rather than a fixed contract. The findings that
generalize across environments are the **relative costs** between the three functions
and their **scaling behavior** in N — those relationships hold even when the absolute
microseconds drift. Capture a fresh local run before drawing conclusions about a
specific change, and compare against the relationships described here rather than the
exact figures.

## Representative results

Measured on an Apple M3 (release profile, criterion 0.5.x); medians, rounded.

### `aggregate_signatures` — median wall time

| N operators | Time    | Per-operator |
|-------------|---------|-------------|
| 1           | ~130 ns | ~130 ns     |
| 4           | ~4.5 µs | ~1.1 µs     |
| 16          | ~9 µs   | ~575 ns     |
| 64          | ~31 µs  | ~480 ns     |
| 128         | ~63 µs  | ~490 ns     |

Scales linearly after the first point, settling to roughly a fixed cost per operator
at quorum-scale loads (sub-microsecond per operator on this machine).

### `get_points` — median wall time

| N operators | Time    | Per-operator |
|-------------|---------|-------------|
| 1           | ~170 ns | ~170 ns     |
| 4           | ~17 µs  | ~4.4 µs     |
| 16          | ~46 µs  | ~2.9 µs     |
| 64          | ~213 µs | ~3.3 µs     |
| 128         | ~830 µs | ~6.5 µs     |

Consistently several times more expensive than `aggregate_signatures` at large N
because it runs three separate projective loops (G1 keys, G2 keys, and signatures).

### `aggregate_verify` — median wall time

| N operators | Time   |
|-------------|--------|
| 1           | ~4 ms  |
| 4           | ~2.2 ms |
| 16          | ~2.2 ms |
| 64          | ~3.3 ms |
| 128         | ~3.5 ms |

The two Miller-loop pairings dominate completely — there is a roughly fixed pairing
floor that holds regardless of N, with G2 key aggregation adding only a small amount
even at the largest operator counts. The aggregation step
(`aggregate_signatures` + `aggregate_verify`) is pairing-bound, not aggregation-bound,
across the entire range tested.

## Key observations

1. **Pairing, not aggregation, is the bottleneck.** `aggregate_verify` is one to two
   orders of magnitude slower than `aggregate_signatures` at every operator count
   tested. Optimising the projective additions would not materially improve round
   latency.

2. **`get_points` becomes non-trivial as quorums grow.** Its three sequential scalar
   accumulations dominate its cost at large N. If the BLS executor is on the critical
   path and quorums grow large, replacing them with a single multi-scalar
   multiplication (MSM) pass is worth investigating (#167).

3. **`aggregate_verify` cost is pairing-floor dominated.** An MSM for G2 key
   aggregation would reduce the per-operator contribution at large N but would not
   touch the fixed pairing cost. Switching to a batch-pairing or pre-aggregated key
   scheme (#167) would have the largest impact on round latency.
