# Sprint 139 Explicit Test Manifest

Sprint 139 locks the workspace test contract around explicit Cargo test targets.

## Why

The previous top-level integration test surface had over one thousand `tests/*.rs`
files. `cargo test --workspace --no-run --quiet` compiled those targets through
auto-discovery and repeatedly timed out. Sprint 138 recovered no-run by setting
`autotests = false` and listing only the active test targets in `Cargo.toml`.

## Active Targets

- `minimal_ai_committee_core`
  - Protects the current AI committee core, paper-only autonomous loop,
    watchlist recheck, Risk Governor behavior, deterministic behavior,
    local-only path guards, and no broker/order/account/training/live-inference
    safety assertions.
- `workspace_timeout_reduction_queue`
  - Protects no-run/full-acceptance truth, timeout diagnostic safety, migrated
    cargo-json assertions, CLI warning/forbidden-command checks, read-only
    Control Tower legacy safety, no hidden skip/default assertion deletion
    guards, and Sprint 118 determinism.

## Deleted Legacy Tests

The deleted tests were legacy sprint/report/diagnostic/control-tower/timeout
test binaries. Their active safety assertions are preserved in the two survivor
targets above, or they only covered obsolete report formatting and historical
diagnostic bundle shape.

## Future Rule

Full old auto-discovery is no longer the workspace test contract. New integration
tests must be added deliberately to `Cargo.toml` as explicit `[[test]]` targets
only when they protect current product behavior or safety.
