# Commonware AVS

Monorepo for the Commonware AVS reference implementation on EigenLayer. It contains:

- [`core`](./core): Core protocol types, validators, wire formats, and utility code
- [`bindings`](./bindings): Standalone crate for on-chain contract bindings
- [`router`](./router): Generic service library for running an aggregation/orchestrator node
- [`node`](./node): Generic service library for running a contributor/operator node
- [`usecase-counter`](./usecase-counter): Implementation of the example "counter" AVS usecase —- demonstrates BLS aggregation workflow
- [`config`](./config): Configuration files for local network, contract deployments, and test keys
- [`scripts`](./scripts): Helper scripts for end-to-end validation and local integration testing
- `docker-compose.yml`: One-command stack runner (Ethereum, EigenLayer, router, operator nodes, signer)

## Example

[Counter](https://github.com/BreadchainCoop/commonware-avs-counter) demonstrates a simple end‑to‑end AVS flow:

- BLS quorum signing (n‑of‑m) by [`node`](./usecase-counter/node) operators
- Aggregation and on‑chain execution by [`router`](./usercase-counter/router) (increments a counter contract)
- Message validation and payload hashing via [`core`](./core) wire + validator utilities

> [!NOTE]
> Usecase implementations (like `counter`) will be moved to dedicated repositories (e.g., `commonware-avs-counter`). This repository will converge on providing the core AVS libraries (shared protocol types, bindings, wire/validators) and base services, and is intended to serve primarily as a reusable library layer for downstream usecases.

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
- Router/orchestrator

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

# Stop and remove volumes (clean state)
docker compose down -v
```

## Licensing

This repository is dual-licensed under both the [Apache 2.0](./LICENSE-APACHE) and [MIT](./LICENSE-MIT) licenses. You may choose either license when employing this code.
