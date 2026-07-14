# Commonware AVS

Monorepo for the Commonware AVS reference implementation on EigenLayer. It contains:

- [`core`](./core): Core protocol types — the BN254 certificate scheme for the commonware-consensus aggregation engine, validators, wire formats, and utility code
- [`bindings`](./bindings): Standalone crate for on-chain contract bindings
- [`router`](./router): Generic library for running the aggregation router: task sequencer, verifier-only aggregation engine, and on-chain certificate submitter
- [`node`](./node): Generic library for running an operator node: aggregation-engine participant actors (task book, automaton, reporter)
- [`eigenlayer`](./eigenlayer): EigenLayer integration utilities (registry, operator state, signing)
- [`examples/counter`](./examples/counter): Implementation of the example "counter" AVS usecase
- [`config`](./config): Configuration files for local network, contract deployments, and test keys
- [`scripts`](./scripts): Helper scripts for end-to-end validation and local integration testing
- `docker-compose.yml`: One-command stack runner (Ethereum, EigenLayer, router, operator nodes, signer)

## Architecture

Tasks flow through a fixed operator set as a sequence of *aggregation heights*:

1. **The router assigns heights.** Each application task gets the next monotonically
   increasing height and is broadcast to the operators as an `Announce` directive on
   p2p channel 1 (rebroadcast until the height resolves). A height the router
   abandons — no certificate within `ROUND_TIMEOUT` — is broadcast as `Skip` instead,
   so the pipeline always advances.
2. **Nodes validate and sign.** Each node independently validates an announced task
   against its own view of the world (the application's `ValidatorTrait`
   implementation) and signs the expected task digest — or the skip digest for an
   abandoned height. Signatures travel as acks on channel 0, gossiped by the
   commonware-consensus aggregation engine running the deployment's certificate
   scheme.
3. **The router certifies and submits.** The router runs the same engine
   verifier-only: it validates acks and assembles a certificate once quorum is
   reached. A scheme-specific submitter resolves the certificate into on-chain
   calldata and hands it to the application's handler for the verification call.

### Signature schemes

The actor chassis (sequencer, task book, automatons, reporters) is generic over
`commonware_cryptography::certificate::Scheme` and the p2p identity type, so a
deployment picks its scheme at wiring time. One single-shot scheme ships in
[`core`](./core):

- **BN254 multisig** (`core::bn254`): the G2 operator key doubles as the p2p
  identity; certificates carry one aggregated G1 signature. The
  [`Submitter`](./router/src/submitter.rs) resolves signers through
  `BLSApkRegistry` and fetches `NonSignerStakesAndSignature` for on-chain BLS
  verification.

Schemes with needs beyond single-shot signing (extra protocol rounds, nonce
management, side storage) live in the application, plugged in through the same
seams the built-in schemes use: implement `certificate::Scheme` for the engine,
`CertIndex` to drive the shared sequencer, and consume certificates generically —
router-side via the `CertifiedHeight<S>` channel, node-side via the reporter's
opt-in `CertificateObservation<S>` tap (`NodeReporter::with_certificate_tap`).

Quorum is engine-fixed at N3f1 — `n − ⌊(n−1)/3⌋` of `n` operators (3-of-3 at n=3,
3-of-4 at n=4). Deployments should run n ≥ 4 in production so a single unavailable
operator does not halt certification.

**Operators must be mutually reachable.** Liveness depends on ack gossip between
participants: a node only advances its own tip by assembling certificates from its
peers' acks, and its engine only proposes heights within `AGG_WINDOW` of that tip.
In a topology where operators reach the router but not each other, node tips never
advance and aggregation wedges after exactly `AGG_WINDOW` heights — the registered
operator sockets must resolve and connect from every other operator, not just from
the router.

Both engines journal their progress under `STORAGE_DIR`. The journal is safe to
lose: nodes simply re-sign whatever the router drives next, and a router that lost
its journal recovers its position from journal replay and the nodes' tip reports.

Tuning environment variables:

| Variable | Default | Purpose |
| --- | --- | --- |
| `ROUND_TIMEOUT` | `30` (s) | Time the router waits for a certificate before broadcasting `Skip` |
| `REBROADCAST_INTERVAL` | `5` (s) | Directive rebroadcast cadence; also the engine's ack rebroadcast timeout |
| `AGG_WINDOW` | `8` | Heights the engine works on concurrently above its tip |
| `AGG_ACTIVITY_TIMEOUT` | `256` | Heights the engine keeps tracking below its tip |
| `P2P_ACK_MESSAGES_PER_SECOND` | computed | Per-peer quota for the ack channel (default derived from the two knobs above) |
| `STORAGE_DIR` | `/app/data` | Stable directory for the engine journals |

## Example

[Counter](./examples/counter) demonstrates a simple end‑to‑end AVS flow:

- Task validation and BN254 quorum signing by [`node`](./examples/counter/node) operators
- Height sequencing, certificate assembly, and on‑chain execution by the [`router`](./examples/counter/router) (increments a counter contract)
- Task digests and directive wire formats via [`core`](./core) wire + validator utilities

## Quick Start

### Prerequisites
- Docker and Docker Compose
- Git

### Local Development

1. **Configure environment:**
```bash
cp example.env .env
```

For LOCAL mode (default), the example.env is pre-configured. You'll need to set a private key:
```bash
# Use Anvil's default test key for local development
echo "PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80" >> .env
echo "FUNDED_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80" >> .env
```

2. **Start all services:**
```bash
docker compose up -d
```

This will automatically build or pull the latest pre-built images (from ghcr.io) and start:
- Ethereum node (Anvil fork)
- EigenLayer contract deployment
- Signer service
- 3 operator nodes
- Router

3. **Monitor services:**
```bash
# View logs
docker compose logs -f router

# Check service status
docker compose ps
```

### Stop Services

```bash
# Stop all services
docker compose down

# Stop and remove volumes (clean state, including engine journals)
docker compose down -v
```

## Licensing

This repository is dual-licensed under both the [Apache 2.0](./LICENSE-APACHE) and [MIT](./LICENSE-MIT) licenses. You may choose either license when employing this code.
