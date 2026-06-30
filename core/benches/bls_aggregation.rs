use ark_bn254::Fr as Scalar;
use commonware_avs_core::bn254::{
    Bn254, G1PublicKey, PublicKey, Signature, aggregate_signatures, aggregate_verify, get_points,
};
use commonware_cryptography::Signer;
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

// Operator counts that span the practical quorum range.
const OPERATOR_COUNTS: &[usize] = &[1, 4, 16, 64, 128];

// Fixed 32-byte pre-hashed message; passing a digest directly bypasses the SHA-256
// step inside sign/verify so the benchmark isolates the curve arithmetic.
const MESSAGE: &[u8] = &[0x42u8; 32];

struct OperatorSet {
    g1_keys: Vec<G1PublicKey>,
    g2_keys: Vec<PublicKey>,
    signatures: Vec<Signature>,
}

// Generates n deterministic keypairs and their signatures over MESSAGE.
// Using from_u64 scalars (1..=n) keeps setup fast and reproducible without
// pulling in UniformRand or a separate RNG dependency.
fn make_operators(n: usize) -> OperatorSet {
    let mut g1_keys = Vec::with_capacity(n);
    let mut g2_keys = Vec::with_capacity(n);
    let mut signatures = Vec::with_capacity(n);
    for i in 0..n {
        let scalar = Scalar::from((i as u64) + 1);
        let signer = Bn254::from_scalar(scalar);
        g1_keys.push(signer.public_g1());
        g2_keys.push(signer.public_key());
        signatures.push(signer.sign(&[], MESSAGE));
    }
    OperatorSet {
        g1_keys,
        g2_keys,
        signatures,
    }
}

// Benchmarks aggregate_signatures: O(N) G1 projective additions + one into_affine.
fn bench_aggregate_signatures(c: &mut Criterion) {
    let mut group = c.benchmark_group("aggregate_signatures");
    for &n in OPERATOR_COUNTS {
        let ops = make_operators(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| aggregate_signatures(black_box(&ops.signatures)));
        });
    }
    group.finish();
}

// Benchmarks get_points: three independent O(N) projective-addition loops (G1 keys,
// G2 keys, and signatures) each followed by into_affine.
fn bench_get_points(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_points");
    for &n in OPERATOR_COUNTS {
        let ops = make_operators(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                get_points(
                    black_box(&ops.g1_keys),
                    black_box(&ops.g2_keys),
                    black_box(&ops.signatures),
                )
            });
        });
    }
    group.finish();
}

// Benchmarks aggregate_verify: O(N) G2 projective additions + into_affine +
// two Miller-loop pairings (the dominant cost). The aggregated signature is
// pre-computed outside the timed loop so only the verify path is measured.
fn bench_aggregate_verify(c: &mut Criterion) {
    let mut group = c.benchmark_group("aggregate_verify");
    for &n in OPERATOR_COUNTS {
        let ops = make_operators(n);
        let agg_sig = aggregate_signatures(&ops.signatures).unwrap();
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                aggregate_verify(
                    black_box(&ops.g2_keys),
                    black_box(None),
                    black_box(MESSAGE),
                    black_box(&agg_sig),
                )
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_aggregate_signatures,
    bench_get_points,
    bench_aggregate_verify
);
criterion_main!(benches);
