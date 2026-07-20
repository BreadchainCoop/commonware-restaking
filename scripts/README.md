# Local Test Scripts

This directory contains scripts for running a local end-to-end test of the AVS
stack.

## Solana/Jito e2e (`solana_e2e_local.sh`)

ONE command boots the full Solana leg from a clean checkout:

```bash
# From the project root (needs the Agave toolchain 4.1.x + rust + git)
./scripts/solana_e2e_local.sh
```

What it does — NO mocks, NO fake programs, real BLS certificates, real
registrations:

1. Builds the REAL jito restaking + vault programs from
   jito-foundation/restaking source at the pinned rev `358fbc3c` (program ids
   env-injected via `declare_id!(env!(...))`) and the NCN program from
   BreadchainCoop/jito-ncn-program `main` (`cargo-build-sbf`).
2. Boots `solana-test-validator` with the three programs at genesis.
3. Phase A (epoch 0): `counter-solana-deployer deploy` — restaking + vault
   `InitializeConfig`, NCN, 4 operators, the full handshake mesh (each
   init/warmup pair straddles a slot boundary — jito's `SlotToggle` refuses
   same-slot activation), SPL mint + vault + deposit + equal delegations, and
   4 REAL BLS proof-of-possession `RegisterOperator` calls + on-chain ip/port
   registration. Emits `ncn_deploy.json`, per-node BLS key files and the
   router connection file.
4. Waits for a full snapshot archive covering the deploy tip, then RESTARTS
   the validator with `--warp-slot` into epoch 2. This is the epoch_length
   answer: jito hardcodes 432,000 slots/epoch at `InitializeConfig` (no admin
   ix exists to change it) and `SlotToggle` requires
   `current_epoch > activation_epoch + 1`, so the empty epochs are compressed
   by the warp while every state transition stays a real transaction.
5. Phase B: `deploy activate` — vault update-state-tracker crank + NCN
   snapshot crank per operator; asserts all 4 operators can vote.
6. Starts 4 `counter-solana-node` processes + the `counter-solana-router`;
   they reach BLS quorum on the round digest and the router's `JitoSubmitter`
   lands `VerifyCertificate`.
7. `deploy assert-verified` — polls the chain and PASSES only when a
   successful `VerifyCertificate` transaction (discriminator byte 10,
   `meta.err == None`) is observed at `confirmed` commitment.

CI: `.github/workflows/solana-e2e.yml` runs exactly this script on a clean
runner. `docker-compose.solana.yml` runs the same script containerized (see
the design note at the top of that file for why the Solana leg is one
service, not per-role services).

Env overrides: `E2E_WORK_DIR`, `JITO_RESTAKING_SRC`, `NCN_PROGRAM_SRC`
(existing clones to skip cloning), `E2E_KEEP_LEDGER=1`, `E2E_TICKS_PER_SLOT`.

## EVM overview

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
