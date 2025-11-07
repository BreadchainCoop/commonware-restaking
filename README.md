# Commonware AVS

Monorepo for the Commonware AVS reference implementation on EigenLayer. It contains:

- [`router`](./router): Orchestrator and BLS aggregation service (Rust crate + Docker image)
- [`node`](./node): Operator/contributor node participating in aggregation (Rust crate + Docker image)
- [`shared`](./shared): Shared protocol types, on-chain bindings, validators and wire codecs
- [`scripts`](./scripts): Local E2E helpers (e.g. counter verification)
- `docker-compose.yml`: One‑command local stack (Ethereum, EigenLayer, router, 3 nodes, signer)

## Example

[Counter](https://github.com/BreadchainCoop/commonware-avs-counter) demonstrates a simple end‑to‑end AVS flow:

- BLS quorum signing (n‑of‑m) by [`node`](./node) operators
- Aggregation and on‑chain execution by [`router`](./router) (increments a counter contract)
- Message validation and payload hashing via [`shared`](./shared) wire + validator utilities

> **NOTE**
>
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

This will automatically pull the latest pre-built images from the GitHub Container Registry (ghcr.io) and start:
- Ethereum node (Anvil fork)
- EigenLayer contract deployment
- 3 operator nodes
- Router/orchestrator
- Signer service

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

### Building from Source (Development Only)

If you're developing the router and want to test local changes:

```bash
# Build the router image locally
docker build -t ghcr.io/breadchaincoop/commonware-avs-router:dev .

# Run with locally built image
docker compose up -d
```

## Licensing

This repository is dual-licensed under both the [Apache 2.0](./LICENSE-APACHE) and [MIT](./LICENSE-MIT) licenses. You may choose either license when employing this code.