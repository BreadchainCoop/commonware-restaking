# Commonware AVS Router

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org)
[![Docker](https://img.shields.io/badge/docker-ghcr.io/breadchaincoop/commonware--avs--router-blue.svg)](https://github.com/BreadchainCoop/commonware-avs-router/pkgs/container/commonware-avs-router)

A BLS signature aggregation protocol with onchain execution for EigenLayer AVS operators.

## Overview

The router coordinates multiple operators to sign messages, aggregates their signatures when a threshold is reached, and executes the result onchain.

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

## Architecture

The system consists of:

- **Orchestrator**: Coordinates the aggregation process
- **Creator**: Generates payloads and manages rounds  
- **Executor**: Handles onchain execution
- **Validator**: Validates messages and signatures
- **Contributors**: Operator nodes that sign messages (implemented in [`commonware-avs-node`](https://github.com/BreadchainCoop/commonware-avs-node) submodule)

### Usecases

The router supports multiple usecases for different onchain operations:

- **[Counter Usecase](src/usecases/counter/README.md)**: Simple counter increment with BLS signature aggregation
- More usecases can be added by implementing the `Creator` and `Executor` traits

See individual usecase READMEs for detailed architecture diagrams and implementation details.

## Configuration

### Environment Variables

Required environment variables:
- `HTTP_RPC`: HTTP RPC endpoint
- `WS_RPC`: WebSocket RPC endpoint
- `AVS_DEPLOYMENT_PATH`: Path to deployment JSON file
- `CONTRIBUTOR_X_KEYFILE`: BLS key files for contributors
- `PRIVATE_KEY`: Private key for transactions. **NOTE:** Address must be funded on Holesky testnet

Optional environment variables:
- `AGGREGATION_FREQUENCY`: Signature aggregation frequency in seconds, supports fractional values (default: 30)
  - Examples: `30` (30 seconds), `1` (1 second), `0.1` (100ms), `0.5` (500ms)
- `THRESHOLD`: Minimum signatures required for aggregation
- `INGRESS`: Enable HTTP ingress mode (true/false)
- `INGRESS_ADDRESS`: Address for ingress server (default: 0.0.0.0:8080)
- `INGRESS_TIMEOUT_MS`: Timeout for waiting for ingress tasks in milliseconds (default: 30000)

Contract addresses are automatically loaded from the deployment JSON file.

### Deployment Options

### Kubernetes with Helm

Deploy the router to Kubernetes using Helm charts:

#### Prerequisites
- Kubernetes cluster (1.19+)
- Helm 3.x installed
- kubectl configured with cluster access

#### Installation

1. **Basic installation with default values:**
```bash
helm install my-router charts/commonware-avs-router
```

2. **Installation with custom values:**
```bash
# Create custom values file
cat > custom-values.yaml <<EOF
replicaCount: 3
image:
  tag: "v1.0.0"
env:
  - name: HTTP_RPC
    value: "https://ethereum-rpc.example.com"
  - name: WS_RPC
    value: "wss://ethereum-ws.example.com"
  - name: AGGREGATION_FREQUENCY
    value: "10"
secrets:
  private-key: "your-private-key-here"
EOF

# Install with custom values
helm install my-router charts/commonware-avs-router -f custom-values.yaml
```

3. **Enable ingress:**
```bash
helm install my-router charts/commonware-avs-router \
  --set ingress.enabled=true \
  --set ingress.className=nginx \
  --set ingress.hosts[0].host=router.example.com \
  --set ingress.hosts[0].paths[0].path=/ \
  --set ingress.hosts[0].paths[0].pathType=Prefix
```

#### Upgrade

```bash
# Upgrade with new values
helm upgrade my-router charts/commonware-avs-router -f custom-values.yaml

# Rollback if needed
helm rollback my-router
```

#### Uninstallation

```bash
helm uninstall my-router
```

#### Configuration

Key Helm values you can configure:

| Parameter | Description | Default |
|-----------|-------------|---------|
| `replicaCount` | Number of replicas | `1` |
| `image.repository` | Image repository | `ghcr.io/breadchaincoop/commonware-avs-router` |
| `image.tag` | Image tag | `latest` |
| `image.pullPolicy` | Image pull policy | `IfNotPresent` |
| `service.type` | Service type | `ClusterIP` |
| `service.port` | Service port | `8080` |
| `ingress.enabled` | Enable ingress | `false` |
| `resources.limits.cpu` | CPU limit | `500m` |
| `resources.limits.memory` | Memory limit | `512Mi` |
| `env` | Environment variables | `[]` |
| `secrets` | Sensitive data | `{}` |
| `autoscaling.enabled` | Enable HPA | `false` |
| `autoscaling.minReplicas` | Min replicas for HPA | `1` |
| `autoscaling.maxReplicas` | Max replicas for HPA | `10` |

See `charts/commonware-avs-router/values.yaml` for complete configuration options.

### Docker

Pull the latest image:
```bash
docker pull ghcr.io/breadchaincoop/commonware-avs-router:latest
```

Run with Docker Compose:
```yaml
version: '3.8'
services:
  orchestrator:
    image: ghcr.io/breadchaincoop/commonware-avs-router:latest
    volumes:
      - ./config:/app/config
      - ./keys:/app/keys
    environment:
      - HTTP_RPC=${HTTP_RPC}
      - WS_RPC=${WS_RPC}
      - AVS_DEPLOYMENT_PATH=/app/config/avs_deploy.json
      - PRIVATE_KEY=${PRIVATE_KEY}
      - AGGREGATION_FREQUENCY=${AGGREGATION_FREQUENCY:-30}
      - CONTRIBUTOR_1_KEYFILE=/app/keys/contributor1.bls.key.json
      - CONTRIBUTOR_2_KEYFILE=/app/keys/contributor2.bls.key.json
      - CONTRIBUTOR_3_KEYFILE=/app/keys/contributor3.bls.key.json
    ports:
      - "3000:3000"
    command: ["--key-file", "/app/config/orchestrator.json", "--port", "3000"]
```

## Ingress Mode

Enable HTTP endpoints for external task requests:

1. **Enable ingress in .env:**
```bash
INGRESS=true
```

2. **Restart the router:**
```bash
docker compose restart router
```

3. **Trigger tasks via HTTP:**
```bash
curl -X POST http://localhost:8080/trigger \
  -H "Content-Type: application/json" \
  -d '{"body": {"metadata": {"request_id": "1", "action": "increment"}}}'
```

## Development

### Dependencies
- `alloy`: Ethereum interaction
- `bn254`: BLS signature operations  
- `commonware_cryptography`: Cryptographic operations
- `commonware_p2p`: P2P networking
- `commonware_runtime`: Runtime utilities

### Code Quality
```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

### Testing

Run unit tests:
```bash
cargo test --lib
```

Run end-to-end tests:
```bash
chmod +x scripts/router_e2e_local.sh
./scripts/router_e2e_local.sh
```
