# Commonware AVS

Monorepo for the Commonware AVS reference implementation on EigenLayer. It contains:

- [`core`](./core): Core protocol types, validators, wire formats, and utility code
- [`bindings`](./bindings): Standalone crate for on-chain contract bindings
- [`router`](./router): Generic service library for running an aggregation/orchestrator service
- [`node`](./node): Generic service library for running a contributor/operator node

### Test

#### Running Rust Tests

To run all Rust tests in this repository, use the following command:

```bash
cargo test --all-features
```

This will build and run all unit and integration tests for all crates in the workspace.

For more targeted testing, you can run:

```bash
# Run tests in a specific crate
cargo test -p <crate-name>
```

or

```bash
# Run a specific test by name
cargo test <test_name>
```

##### Note

- All code is checked for formatting and lint errors using `cargo fmt` and `cargo clippy`.
- Continuous Integration runs all tests automatically on GitHub Actions under `.github/workflows/rust-ci.yml`.

## Licensing

This repository is dual-licensed under both the [Apache 2.0](./LICENSE-APACHE) and [MIT](./LICENSE-MIT) licenses. You may choose either license when employing this code.
