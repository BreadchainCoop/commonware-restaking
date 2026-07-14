# Commonware AVS Node

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org)

## Overview

A node runs an operator's participant actors for the `commonware-consensus`
aggregation engine: it validates announced tasks, signs their expected digests,
and gossips acks with its peers. See the [top-level README](../README.md) for the
full task-flow and quorum model.

## Architecture

- [`NodeAutomaton`](src/automaton.rs): answers the engine's `propose` calls by
  validating the router's announced task (via the application's `ValidatorTrait`,
  e.g. [`CounterValidator`](../examples/counter/common/src/validator.rs)) and
  resolving the expected digest, or the skip digest once a directive is
  abandoned.
- [`TaskBook`](src/task_book.rs): tracks the router's `TaskDirective` broadcasts
  on p2p channel 1 and resolves each height the automaton is asked about.
- [`NodeReporter`](src/reporter.rs): the engine's activity sink; deduplicates
  certificates across journal replays, tracks tip height for metrics, and prunes
  the `TaskBook`. Applications needing post-certification work (e.g. a
  non-BN254 scheme) opt in to a certificate tap via
  `NodeReporter::with_certificate_tap`.

## Configuration

### Router Connection File

A node locates the router via a JSON file passed to `--router` carrying the
router's public identity and socket address:

```json
{
    "g2_x1": "20265730220917057623326116620721648047640065506233168445998945605458084341755",
    "g2_x2": "1537141129484558011683382469842956131676085503509229854572844956364492197092",
    "g2_y1": "4380068110839997539835821427545270098552639074995346826656804866303457881635",
    "g2_y2": "479676018937294309080674601592141614301396550682703157902264620243097107417",
    "address": "192.168.1.100",
    "port": "3000"
}
```

`address` defaults to `"localhost"` if omitted.

### Environment Variables

- `STORAGE_DIR`: Directory for the engine's journal (must persist across
  restarts)
- `AGG_WINDOW`: Heights the aggregation engine works on concurrently above its
  tip (default: 8)
- `AGG_ACTIVITY_TIMEOUT`: Heights the engine keeps tracking below its tip
  (default: 256)
- `REBROADCAST_INTERVAL`: The engine's ack rebroadcast cadence, in seconds
  (default: 5)
- `P2P_MESSAGES_PER_SECOND` / `P2P_ACK_MESSAGES_PER_SECOND`: Per-peer rate quotas
  for the task-directive and ack channels respectively
- `P2P_MESSAGE_BACKLOG`: Per-peer message backlog size

### Running

```bash
cargo run --release -- --key-file operator1.bls.key.json --port 3001 --router public_router.json
```

Run one process per operator, each with its own `--key-file` and `--port`.
