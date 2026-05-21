# Sprint 91 KRX Evidence Recovery

Sprint 91 follows Sprint 90 because `ExternalPrediction` was the prior blocker and the remaining queue now starts with `KrxEvidence`.

This sprint stays conservative:

1. reduce only the `KrxEvidence` family,
2. keep auth, endpoint-template, source-boundary, and market-data-only gates explicit,
3. keep `cargo test --workspace --no-run --quiet` separate from `cargo test --workspace --quiet`,
4. never claim a fake pass.

No Sprint 91 artifact implies live trading, broker/order/account readiness, runtime inference, runtime LLM, browser execution, or training.
