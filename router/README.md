# Commonware AVS Router

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org)

## Overview

The router runs a verifier-only `commonware-consensus` aggregation engine alongside
a task sequencer and an on-chain submitter: it assigns aggregation heights to
application tasks, certifies operator acks via the engine, and submits certified
heights on-chain. See the [top-level README](../README.md) for the full task-flow
and quorum model.

## Architecture

- [`Sequencer`](src/sequencer.rs): assigns each application task the next
  aggregation height, rebroadcasts `TaskDirective::Announce` until a certificate
  lands, and falls back to `TaskDirective::Skip` after `ROUND_TIMEOUT`.
- [`RouterAutomaton`](src/automaton.rs): resolves the verifier-only engine's
  `propose` calls against the sequencer's assignments.
- [`CertReporter`](src/reporter.rs): the engine's activity sink; forwards
  certified heights to the submitter.
- [`Submitter`](src/submitter.rs): resolves a certified height into an EigenLayer
  `NonSignerStakesAndSignature` submission and calls the application's
  [`BlsSignatureVerificationHandler`](src/executor.rs).
- `TaskSource` (application-implemented — see
  [`examples/counter/router/src/source.rs`](../examples/counter/router/src/source.rs)):
  supplies the tasks the sequencer assigns heights to. The counter example polls
  the deployed contract every `POLLING_INTERVAL_MS`.

Applications add a usecase by implementing `TaskSource` and
`BlsSignatureVerificationHandler` against their own contract bindings — see
[`examples/counter`](../examples/counter) for a full implementation.

## Configuration

### Environment Variables

Required:
- `HTTP_RPC`: HTTP RPC endpoint
- `WS_RPC`: WebSocket RPC endpoint
- `AVS_DEPLOYMENT_PATH`: Path to the deployment JSON file
- `PRIVATE_KEY`: Private key for submitting transactions. **NOTE:** Address must
  be funded

Optional:
- `ROUND_TIMEOUT`: Seconds the router waits for a certificate on an assigned
  height before broadcasting `Skip` (default: 30, fractional allowed)
- `REBROADCAST_INTERVAL`: Seconds between `TaskDirective` rebroadcasts; also the
  engine's own ack rebroadcast cadence (default: 5)
- `AGG_WINDOW`: Heights the aggregation engine works on concurrently above its
  tip (default: 8)
- `AGG_ACTIVITY_TIMEOUT`: Heights the engine keeps tracking below its tip
  (default: 256)
- `P2P_MESSAGES_PER_SECOND` / `P2P_ACK_MESSAGES_PER_SECOND`: Per-peer rate quotas
  for the task-directive and ack channels respectively (the latter defaults to a
  value computed from `AGG_ACTIVITY_TIMEOUT` / `REBROADCAST_INTERVAL`)
- `P2P_MESSAGE_BACKLOG`: Per-peer message backlog size
- `STORAGE_DIR`: Directory for the engine's journal (must persist across
  restarts)
- `POLLING_INTERVAL_MS`: How often the counter `TaskSource` polls for a new round
  (default: 2000)

Contract addresses are loaded from the deployment JSON file.

### Running

```bash
cargo run -p counter-router --release -- --key-file router_key.json --port 3000
```

`--key-file` is the router's own BLS private key (`{"privateKey": "..."}`).
`--bootstrappers` optionally takes a comma-separated list of additional peer
addresses. Nodes locate the router via a separate file carrying its public
identity and socket address (`{g2_x1, g2_x2, g2_y1, g2_y2, address, port}`,
conventionally named `public_router.json`), passed to each node's `--router`
flag.

### Docker

See the `router` service in [`docker-compose.yml`](../docker-compose.yml) for a
complete example, including the volumes and command needed to run alongside
`node-1`/`node-2`/`node-3`.
