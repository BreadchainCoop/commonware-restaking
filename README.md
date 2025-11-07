# Run on Solana

## Prerequisites

- Solana CLI: [install](https://solana.com/docs/intro/installation)
- Surfpool: [install](https://github.com/txtx/surfpool)

## Setup Solana Env

Go to BLS NCN repo
```bash
git clone https://github.com/jito-foundation/jito-bls-ncn
cd jito-bls-ncn
git checkout ck/ncn
```

Build the CLI
```bash
./run.sh
```

Start surfpool in one terminal:
```bash
surfpool start
```

Get a test setup:
```bash
jito-bls-ncn-cli surfpool-create-test-ncn
```

## Run Commonware P2P
Go back to this directory
```bash
cd ../commonware-avs-router
```

Edit the `.env` to have `TEST_BLS_NCN_CONFIG_FILE_PATH=/path/to/test_ncn_output.json`

Go to the node directory
```bash
cd commonware-avs-node
```

Edit the `.env` to have `TEST_BLS_NCN_CONFIG_FILE_PATH=/path/to/test_ncn_output.json`

In three different terminals run the following:
Terminal 1:
```bash
cargo run --release -- --port 3001 --operator_index 0
```
Terminal 2:
```bash
cargo run --release -- --port 3002 --operator_index 1
```
Terminal 3:
```bash
cargo run --release -- --port 3003 --operator_index 2
```

In a fourth terminal, run the orchestrator:
Terminal 4:
```bash
cd ..
cargo run --release -- --port 3000
```

And watch it go!
