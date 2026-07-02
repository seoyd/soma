# Repository Audit

## Summary

- Repository root examined: `/Users/seo/Projects/soma`
- Git status: unavailable here because this directory is **not a git repository root**
- Cargo metadata: captured in `target/cargo-metadata.json`
- File inventory: captured in `target/file-list.txt` and `target/rust-file-list.txt`

## Workspace map

Current active workspace after removing the legacy `soma-model`, `soma-metal`, `soma-core`, `soma-reason`, `soma-active`, `soma-latent`, `soma-infer`, `soma-validate`, `soma-config`, `soma-quant`, `soma-scheduler`, and `soma-telemetry` crates:

- **1 active package**
- **no custom build targets remain in the active workspace**

### Active workspace members

`soma-zero`

### Active binaries

- `soma-zero:soma-zero`

## Reachability findings

### Build / module reachability

- `Cargo.toml` is now the root `soma-zero` package manifest.
- `src/` now holds the active crate directly at the repository root.
- the older inference/validation stack (`soma-reason`, `soma-active`, `soma-latent`, `soma-infer`, `soma-validate`) and the former support stack (`soma-config`, `soma-quant`, `soma-scheduler`, `soma-telemetry`) have been removed from the active workspace.
- the old model-system crates (`soma-ssm`, `soma-gdn`, `soma-attn`, `soma-mor`, `soma-memory`, `soma-online`, `soma-adapt`) are no longer part of the active workspace
### Soma Zero v0 path

The clean MVP-aligned runtime path is now at the repository root:

- `src/core/*`
- `src/signal/*`
- `src/league/*`
- `src/chair/*`
- `src/risk/*`
- `src/paper/*`
- `src/backtest/*`
- `tests/mvp.rs`

This path has no runtime LLM references and no real broker path.

## Deep cleanup result

First quarantine batch (historical intermediate step before later deletion):

- `soma-cli`
- `soma-release`

Evidence:

- both had **zero inbound workspace dependencies**
- neither was required by any remaining workspace member
- non-doc references were limited to self references, root workspace membership, `Cargo.lock`, and one commented `bin/ci.sh` hint for `soma-release`
- after removing them from `[workspace].members` and moving their directories to `quarantine/unused_bins/`, `cargo fmt --all`, `cargo check --workspace`, and `cargo test --workspace --quiet` still passed

Second quarantine batch (historical intermediate step before later deletion):

- `soma-bench`
- `soma-train`
- `soma-orchestrate`
- `soma-canary`
- `soma-soak`

Evidence:

- all were zero-inbound workspace members in reverse-dependency analysis
- non-doc references were limited to workspace membership, self references, root/task notes, or commented CI snippets
- after moving them into quarantine and removing them from `[workspace].members`, the workspace still passed `cargo fmt --all`, `cargo check --workspace`, and `cargo test --workspace --quiet`

Final quarantine batch (historical intermediate step before later deletion):

- `soma-prop`
- `soma-replay`
- `soma-serve`

Evidence:

- all three remained zero-inbound workspace members in a fresh reverse-dependency audit
- non-code references were limited to the temporary instruction artifact, root README/docs, workspace membership, or already-quarantined wrappers
- after removing them from `[workspace].members` and moving them into quarantine, the workspace still passed `cargo fmt --all`, `cargo check --workspace`, and `cargo test --workspace --quiet`

First active-legacy isolation batch:

- `soma-online`
- `soma-adapt`

Evidence:

- both had zero inbound workspace dependencies despite still being active workspace members
- non-code references were limited to root docs, commented CI hints, self READMEs, and already-quarantined orchestration/report artifacts
- after removing them from `[workspace].members` and moving them into `quarantine/deprecated_online_learning/`, the workspace still passed `cargo fmt --all`, `cargo check --workspace`, and `cargo test --workspace --quiet`

Second active-legacy isolation batch:

- `soma-ssm`
- `soma-gdn`

Evidence:

- both became zero inbound workspace members after `soma-model` was refactored to use an internal deterministic block path instead of the old SSM/GDN crates
- non-code references were limited to docs, manifest entries, and quarantined report artifacts
- after removing them from `[workspace].members` and moving them into `quarantine/old_experiments/`, the workspace still passed `cargo fmt --all`, `cargo check --workspace`, and `cargo test --workspace --quiet`

Third active-legacy isolation batch:

- `soma-attn`
- `soma-mor`
- `soma-memory`

Evidence:

- `soma-model` was refactored to use internal deterministic attention/router/memory helpers instead of the old crates
- all three became zero inbound workspace members
- after moving them into quarantine, the workspace still passed `cargo fmt --all`, `cargo check --workspace`, and `cargo test --workspace --quiet`

## Tests and build baseline

Observed in Sprint 02:

- `cargo check --workspace` passed after Sprint 03 isolation
- `cargo test --workspace --quiet` passed after Sprint 03 isolation
- `cargo test` passed

## Suspicious / misaligned modules

No legacy model-system crates remain technically live in the workspace after the third Sprint 03 isolation batch.

## Cleanup candidates

### Classification summary

**KEEP_CORE**

- `src/`
- `tests/`
- root package (`Cargo.toml`)

**KEEP_TEST**

- `tests/mvp.rs`
- active crate unit tests that still execute under `cargo test --workspace --quiet`

**KEEP_DOC**

- `README.md`
- `docs/ARCHITECTURE.md`
- `docs/REPO_AUDIT.md`
- `docs/CLEANUP_PLAN.md`
- `docs/DEFERRED_MODULES.md`
- local temporary instruction artifact

**KEEP_DEFERRED**

- conceptual future items documented in `docs/DEFERRED_MODULES.md`:
  - Mamba3Fin
  - Sparse Mamba follow-up research
  - FA3 follow-up research
  - evolution sandbox
  - 6/12/18 investor league expansion

**HISTORICAL_RECORD**

- `cleanup_manifest.toml` entries that record the earlier quarantine phase before permanent deletion

**DELETE_CANDIDATE**

- none in this sprint

**UNKNOWN**

- local temporary instruction artifact retained outside build reachability

### Quarantined in Sprint 02

The following top-level report files were not part of Cargo/module/build/test reachability and were moved to quarantine:

- `ACCEPTANCE_REPORT.md`
- `EVAL.md`
- `MEMORY_LEAK_REPORT.md`
- `OPTIMIZATION.md`
- `UPGRADE_REPORT.md`

Additional deep-cleanup quarantine:

- `soma-cli/`
- `soma-release/`
- `soma-bench/`
- `soma-train/`
- `soma-orchestrate/`
- `soma-canary/`
- `soma-soak/`
- `soma-prop/`
- `soma-replay/`
- `soma-serve/`
- `soma-online/`
- `soma-adapt/`
- `soma-ssm/`
- `soma-gdn/`
- `soma-attn/`
- `soma-mor/`
- `soma-memory/`

Evidence:

- no longer active workspace members
- no remaining Cargo/build reachability from active packages
- no live code/module imports from active workspace crates
- only docs, manifest, or quarantine-local references remain
- workspace tests passed after the move

### Deferred / unknown

- local temporary instruction artifact, intentionally kept out of cleanup scope
- remaining active crates are support/runtime crates rather than legacy model-system modules
