# Restart Sprint 17 Report

## Verification

The sprint began from clean commit `6c6e1e9`. Formatting, workspace compile,
512 tests, and diff validation passed before feature work. Final release
verification also passed:

- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test --workspace --quiet`
- `git diff --check`

The final run passed 516 tests: 100 library tests, 404 integration tests, and
12 additional integration tests. Two pre-existing unused-private-function
warnings remain.

## Expanded Fixtures

Four fake local fixtures were added for synthetic mixed data, Korean stock, US
stock, and BTC crypto profiles. Each contains 20 monotonic rows with trend,
range, drawdown or volatility, and recovery segments. No provider export,
account field, order field, credential, or endpoint is present.

## Diagnostics

Source diagnostics now expose timestamp gaps, price and OHLC range, volume and
trade-value ranges, optional trade-value availability, profile matching,
warning text, and stable reason codes. Cross-source results retain accepted
and rejected source counts and deterministic source-kind ordering.

## Replay Modes

`IndependentPerSource` resets agent state for every source.
`SequentialCarryover` preserves the previous behavior and carries final state
forward. `AsProvided` and `SourceKindThenId` make processing order explicit.

## Agent Consistency

The three active agents receive cross-source rows with voice and paper
reward/penalty ranges, source coverage, high-confidence misses, avoided
losses, cooldowns, quarantines, and deterministic consistency status.

## Batch Report Extension

The owner report now states replay mode, order policy, processing order,
source diagnostics, source warnings, and agent consistency. It explicitly
states the synthetic/sanitized, paper-only, not-live-ready, no-profitability,
Risk Governor, and advisory-owner boundaries.

## Tests

Coverage includes all expanded profile fixtures, deterministic independent and
sequential modes, carryover/reset behavior, source ordering, timestamp gap,
scale, volume and optional-column warnings, non-monotonic rejection, profile
mismatch diagnostics, endpoint rejection, three-agent consistency, and report
safety.

## Risk And Security Review

No dependency, live provider, downloader, broker, account, order, runtime LLM,
heavy model, database, UI, or eight-agent activation path was added. Existing
Chair and Risk Governor processing remains the only decision path used by
batch replay.

## Deferred Items

Real data ingestion, live APIs, brokerage, execution, online learning, live
self-mutation, full eight-agent operation, heavy models, UI, persistence, and
deployment remain deferred.

## Next Sprint

The next paper-only increment should define expected timestamp intervals and
quality thresholds per local profile, then evaluate larger synthetic packs
without adding provider or execution paths.
