# Sprint 95 CommitteeCliSafety recovery

Sprint 95 follows Sprint 94 because Sprint 94 reduced DashboardRenderer, advanced the blocker queue to CommitteeCliSafety, and left CommitteeCliSafety explicitly isolated.

This sprint is CommitteeCliSafety-only. It preserves remote-path rejection, research-only help text, forbidden-command absence, runtime-deferred semantics, persona guards, order/account guards, browser-execution guards, and deterministic CLI surface checks.

## Why isolation decision is required

CommitteeCliSafety is a safety sentinel. The sprint must decide whether grouped suites represent enough coverage to advance the queue safely or whether the sentinel must remain permanently isolated.

## No-run vs full workspace

- `cargo test --workspace --no-run --quiet` remains compile-only.
- `cargo test --workspace --quiet` remains the full-workspace gate.
- Sprint 95 does not claim a CommitteeCliSafety closure equals a full workspace pass.

## No fake pass

Sprint 95 keeps sample-backed timing clearly labeled, keeps no-run/full reruns honest, and leaves runtime, training, live inference, live trading, broker/order/account, and browser execution deferred.
