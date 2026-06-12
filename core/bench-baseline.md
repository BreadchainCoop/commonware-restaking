# BLS Aggregation Benchmark Baseline

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

## Baseline — 2026-06-12 (Apple M3, release profile)

Commit: `bagelface/bls-aggregation-benchmarks` branch (criterion 0.5.1).

### `aggregate_signatures` — median wall time

| N operators | Time    | Per-operator |
|-------------|---------|-------------|
| 1           | 133 ns  | 133 ns      |
| 4           | 4.5 µs  | 1.1 µs      |
| 16          | 9.2 µs  | 575 ns      |
| 64          | 30.7 µs | 480 ns      |
| 128         | 62.7 µs | 490 ns      |

Scales linearly after the first point; ~480–490 ns / operator at quorum-scale loads.

### `get_points` — median wall time

| N operators | Time    | Per-operator |
|-------------|---------|-------------|
| 1           | 170 ns  | 170 ns      |
| 4           | 17.4 µs | 4.4 µs      |
| 16          | 46 µs   | 2.9 µs      |
| 64          | 213 µs  | 3.3 µs      |
| 128         | 830 µs  | 6.5 µs      |

~5–7x more expensive than `aggregate_signatures` at large N because it runs three
separate projective loops (G1 keys, G2 keys, and signatures).

### `aggregate_verify` — median wall time

| N operators | Time   |
|-------------|--------|
| 1           | 4.1 ms |
| 4           | 2.2 ms |
| 16          | 2.2 ms |
| 64          | 3.3 ms |
| 128         | 3.5 ms |

The two Miller-loop pairings (~2.2 ms floor) dominate completely. G2 key
aggregation adds only ~1 ms even at 128 operators. At a 16-operator quorum, the
aggregation step (`aggregate_signatures` + `aggregate_verify`) costs ~2.2 ms per
round — pairing-bound, not aggregation-bound.

## Key observations

1. **Pairing, not aggregation, is the bottleneck.** `aggregate_verify` is ~50–200×
   slower than `aggregate_signatures` at all operator counts. Optimising the
   projective additions would not materially improve round latency.

2. **`get_points` at 128 operators (~830 µs) is non-trivial.** If the BLS executor
   is on the critical path and quorums grow to 128+, replacing the three sequential
   scalar accumulations with a single multi-scalar multiplication (MSM) pass is
   worth investigating (#167).

3. **`aggregate_verify` cost is pairing-floor dominated.** An MSM for G2 key
   aggregation would reduce the ~1 ms contribution at N=128 but would not touch
   the fixed ~2.2 ms pairing cost. Switching to a batch-pairing or pre-aggregated
   key scheme (#167) would have the largest impact.
