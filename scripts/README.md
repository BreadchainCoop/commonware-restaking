# Local Test Scripts

This directory contains scripts for running a local end-to-end test of the AVS
stack.

## Overview

The test validates the complete end-to-end flow:

1. **Local Blockchain Setup**: Starts a local Ethereum blockchain and deploys
   EigenLayer contracts
2. **BLS Signature Aggregation**: Runs the router and 3 operator nodes
3. **Verification**: Confirms that the counter contract was incremented at
   least twice through successful signature aggregation

## Files

- `router_e2e_local.sh` - Main integration test script
- `verify_increments.rs` - Rust script that monitors and verifies counter
  increments
- `Cargo.toml` - Dependencies for the verification script (package
  `commonware-avs-scripts`)

## Running the Test Locally

### Prerequisites

- Docker
- Rust

### Run the Test

```bash
# From the project root
./scripts/router_e2e_local.sh
```

The script will:
1. Build the verification script
2. Set up environment files for local mode
3. Pull and start Docker Compose services (Ethereum, EigenLayer, signer, 3
   nodes, router)
4. Wait for EigenLayer setup and give the nodes time to initialize
5. Wait for signature aggregation cycles
6. Verify the counter was incremented at least twice
7. Clean up all containers

### Expected Output

```
✅ Integration test PASSED! Counter was incremented successfully.
```

## CI/CD Integration

The same flow runs in GitHub Actions:
- `.github/workflows/integration-test.yml` — full Docker build, on push/PR to
  `main`/`dev`/`staging`
- `.github/workflows/local-integration-test.yml` — pulls prebuilt node images
  and runs the router via `cargo run` (no Docker build), on PRs to
  `main`/`dev`/`staging`

## Troubleshooting

### Common Issues

1. **Docker containers fail to start**
   - Check if ports 8545, 3001-3003, 4000 are available
   - Ensure Docker daemon is running

2. **Contract deployment timeout**
   - Increase the timeout in the script
   - Check Docker logs: `docker compose logs`

3. **Nodes fail to connect**
   - Verify keyfiles exist in `config/.nodes/operator_keys/`
   - Check network connectivity between processes

4. **Not Using Funded Private Key**
   - Ensure `PRIVATE_KEY` in `.env` has sufficient ETH for transactions
   - Check balance: `cast balance $(cast --from-utf8 $(cast --private-key $PRIVATE_KEY))`
   - Fund if needed: `cast send --private-key $PRIVATE_KEY --value 1ether <address>`

### Debug Information

On failure, the script prints `docker compose logs` output for the router,
each node, and EigenLayer.

### Manual Verification

You can also run the verification script separately once the stack is running:

```bash
source .env
export AVS_DEPLOYMENT_PATH="config/.nodes/avs_deploy.json"
cargo run -p commonware-avs-scripts --release --bin verify_increments
```

## Configuration

### Environment Variables Needed for Local Test

- `PRIVATE_KEY` - private key for transactions
