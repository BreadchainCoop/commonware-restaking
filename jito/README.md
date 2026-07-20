# commonware-avs-jito

Jito/Solana backend for the commonware-restaking chassis — the additive peer
of the `eigenlayer` crate. Zero changes to the EVM path.

Built against the cross-track interface contract in
[BreadchainCoop/jito-ncn-program `docs/INTERFACES.md`](https://github.com/BreadchainCoop/jito-ncn-program/blob/main/docs/INTERFACES.md)
(§1 signature domain, §2 `VerifyCertificate`, §5 router backend).

## What lives here

| Module | Role |
| --- | --- |
| `bn254` | Key/signature wrappers in the NCN program's signature domain: Solana `alt_bn128` compressed wire formats, `Sha256Normalized` hash-to-curve via `ncn-program-core` (imported, never reimplemented). |
| `scheme` | `JitoBn254Scheme`, a `commonware_cryptography::certificate::Scheme`. Certificates carry exactly the `VerifyCertificate` wire triple (agg sig G1 32B, agg G2 64B, signer bitmap); `verify_certificate` runs the program's own challenge-combined pairing (`verify_aggregated_signature`, EigenLayer-exact gamma). |
| `config` | `NcnDeployment` — the NCN deployment JSON (`NCN_DEPLOYMENT_PATH`), analog of the EVM `avs_deploy.json`, plus PDA derivations. |
| `network` | `JitoStakingClient` — operators via `getProgramAccounts` memcmp on `NCNOperatorAccount` (ncn field; ip+port sockets), stake/APK from the `Snapshot` PDA, all reads at `confirmed` minimum. Produces `JitoQuorum`: index-aligned participant set / G1 keys / on-chain operator indices / stakes. |
| `quorum` | Startup reconciliation (§5): the minimum total stake over all `(N−f)`-sized signer subsets must clear `consensus_threshold_bps`, else refuse to start. |
| `instruction` | Manual `VerifyCertificate` instruction construction (frozen §2 shape; borsh cross-checked). Discriminator (byte `10`) is FROZEN and differentially pinned against `ncn_program_core::instruction::NCNProgramInstruction` — the enum the generated client derives from (Phase 1, merged to `main`). |
| `submitter` | `JitoSubmitter` + the `SolanaCertificateHandler` trait (peer of the EVM `BlsSignatureVerificationHandler`). `VerifyCertificateHandler` sends the tx with a compute budget; `Resolution{Executed}` only at `finalized`; blockhash expiry → rebuild and resend. A settlement-program handler (INTERFACES §4, Track C `settlement_core`) plugs into the same seam. |

Participant indices (sorted G2 positions in the chassis `ordered::Set`) are
NOT the on-chain `ncn_operator_index`; `JitoQuorum` carries the aligned
translation table and the submitter maps certificate bitmaps to the LSB-first
on-chain bitmap (padding bits set, byte-exact with
`ncn_program_core::utils::create_signer_bitmap`).

## Dependency pinning (READ BEFORE `cargo update`)

- `ncn-program-core` is a git dependency on BreadchainCoop/jito-ncn-program
  `main`. Its workspace pins `solana-program` to the jito-solana fork at rev
  `87dcd08`; this workspace's `[patch.crates-io]` section mirrors the
  jito-ncn-program patch set so ONE set of solana types flows through the
  whole graph. The patch is inert for the EVM path.
- jito-foundation/restaking branch `v2.1-upgrade` was DELETED upstream; its
  head `358fbc3c` (what jito-ncn-program pins) remains fetchable by SHA. The
  workspace therefore pins every restaking crate by
  `rev = "358fbc3c20d947c977a136808f9fbf7f070e478b"` (mirroring
  jito-ncn-program main's own re-pin), so fresh clones and `cargo update`
  never touch the dead branch ref. Update surgically
  (`cargo update -p <pkg> --precise <ver>`) all the same — the jito-solana
  fork rev and the restaking rev must move together with the
  `ncn-program-core` pin.

## Example

`examples/counter-solana` mirrors `examples/counter`: router (verifier-only
engine + sequencer + `JitoSubmitter`) and node (signing participant) wired
against a live NCN deployment. It needs a real chain (deployment JSON +
funded payer); nothing is mocked — unit tests use the real crypto
(host-side `ncn-program-core` signing).

The full local e2e is ONE command: `./scripts/solana_e2e_local.sh` — it
builds the jito restaking/vault programs from source at the pinned rev and
the NCN program from main, boots a two-phase (`--warp-slot`) test validator,
runs `examples/counter-solana/deployer` (real registrations incl. BLS
proof-of-possession), starts 4 nodes + the router, and asserts a successful
on-chain `VerifyCertificate` at `confirmed`. CI runs the same script in
`.github/workflows/solana-e2e.yml`; `docker-compose.solana.yml` is the
containerized variant. See `scripts/README.md` for details.
