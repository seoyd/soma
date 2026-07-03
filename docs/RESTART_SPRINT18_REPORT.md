# Restart Sprint 18 Report

## Verification

The sprint began from clean commit `d3dfbb0`. Formatting, workspace compile,
516 tests, and diff validation passed before feature work. Final verification
also passed:

- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test --workspace --quiet`
- `git diff --check`

The final run passed 520 tests: 104 library tests, 404 integration tests, and
12 additional integration tests. Two pre-existing unused-private-function
warnings remain.

## Source Profile Cadence

All four local profiles now declare a deterministic 60-second cadence,
two-interval gap tolerance, zero allowed gaps, and no automatic
weekend/session exception. Korean and US calendar validation is explicitly
deferred.

## Quality Thresholds And Buckets

Profiles define gap, duplicate, optional coverage, anomaly, scale, OHLC, row,
and minimum-score thresholds. Deterministic scoring produces `Excellent`,
`Good`, `Caution`, `Poor`, or `Rejected`.

## Expanded Fixtures

Five 20-row fake fixtures cover clean synthetic data, Korean timestamp gaps,
US scale mismatch, BTC volume anomaly, and BTC missing optional columns.

## Quality-Aware Replay

The default `RejectPoorAndBelow` policy blocks Poor and Rejected sources.
`RejectRejectedOnly` and `ReplayAllAcceptedWithWarnings` allow parsed Poor
sources with warnings. Rejected sources never replay.

## Agent Performance By Quality

Agent performance is grouped by agent ID and quality bucket with source and
episode counts, reward and penalty totals, voice range, misses, avoided losses,
cooldowns, and quarantines.

## Report Changes

The batch report includes policy, threshold summary, bucket counts,
blocked-by-quality sources, cadence and quality diagnostics, and agent
performance by quality bucket. It retains paper-only, advisory-owner, Risk
Governor, no-profitability, and not-live-ready warnings.

## Tests

Coverage includes cadence contracts, calendar-deferred reasons, all V2
fixtures, expected buckets, all quality policies, permanent Rejected blocking,
agent grouping and sorting, deterministic reports, and live-provider safety
rejection.

## Risk And Security Review

No network, downloader, live provider, broker, account, order, runtime LLM,
heavy model, database, UI, or eight-agent path was added. Quality blocking
occurs before paper replay and cannot bypass Risk Governor.

## Deferred Items

Real exchange calendars, timezone/session correctness, real data ingestion,
live APIs, brokerage, execution, online learning, live mutation, full
eight-agent operation, UI, persistence, and deployment remain deferred.

## Next Sprint

The next paper-only increment should calibrate profile thresholds against
larger synthetic packs and add deterministic quality-score sensitivity
reports without adding live providers or execution.
