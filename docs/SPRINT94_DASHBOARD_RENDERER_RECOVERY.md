# Sprint 94 DashboardRenderer recovery

Sprint 94 follows Sprint 93 because Sprint 93 proved `KrxProvenNonPrimary`, released `DashboardRendererEntryReleased`, and advanced the blocker queue to `DashboardRenderer`.

This sprint is renderer/test-cost reduction only. It does not add dashboard serve, browser execution, remote assets, POST/forms, runtime controls, training, broker/order/account paths, or live trading.

## Why entry can be consumed now

- Sprint 93 explicitly released DashboardRenderer entry.
- The current next-family summary is `DashboardRenderer`.
- Entry release only unlocks conservative reduction work. It does not imply no-run/full workspace acceptance.

## No-run vs full workspace

- `cargo test --workspace --no-run --quiet` remains compile-only.
- `cargo test --workspace --quiet` remains the quiet full-workspace gate.
- Sprint 94 never reports a DashboardRenderer reduction as a full workspace pass.

## No fake pass

Sprint 94 keeps all reporting honest: sample-backed compile impact stays labeled, no-run/full reruns stay `NotRun` unless actually executed, and runtime/training/live systems remain deferred.
