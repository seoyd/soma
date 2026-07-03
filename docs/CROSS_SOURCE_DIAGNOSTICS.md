# Cross-Source Consistency Diagnostics

## Purpose

Cross-source diagnostics compare the shape and quality of sanitized local
datasets and the consistency of three-agent paper replay observations. They do
not evaluate real strategy profitability, production readiness, or live
execution quality.

Source quality and strategy performance are separate:

- source diagnostics describe timestamps, scale, volume, schema, and data
  availability;
- agent consistency describes variation in paper-only attribution, reward,
  penalty, voice, and lifecycle observations.

A source warning does not prove a strategy failure. A stable agent result does
not prove a source is production quality.

## Source Diagnostics

`SourceConsistencyDiagnostics` records:

- source ID and source kind,
- row count and timestamp monotonicity,
- timestamp gap count and timestamp range,
- close range and relative close range,
- volume range, volume ratio, and optional trade-value ratio,
- optional trade-value availability,
- optional columns present,
- source profile match,
- suspicious scale status,
- expected profile cadence and optional-column coverage ratio,
- quality score and quality bucket,
- warning text and reason codes.

Timestamp gaps are detected relative to the smallest positive interval in the
dataset. Suspicious scale, abnormal volume range, wide OHLC range, and missing
optional trade value are warnings. Invalid ordering, invalid schema, and
private or endpoint material are rejected before replay.

Profile thresholds convert diagnostics into `Excellent`, `Good`, `Caution`,
`Poor`, or `Rejected`. Quality scoring describes input evidence only and is
kept separate from agent paper outcomes.

`CrossSourceConsistencyReport` sorts diagnostic rows by source kind and source
ID. It includes kind counts, accepted and rejected counts, suspicious source
count, and deduplicated warning text.

## Replay Modes

`IndependentPerSource` starts every accepted source from the same initial
three-agent state. This is the fair comparison mode for source-sensitive
behavior.

`SequentialCarryover` feeds each accepted replay's final state into the next
source. This preserves the earlier batch behavior and models continuous
paper-learning state.

`AsProvided` retains caller order. `SourceKindThenId` sorts processing by
source kind and source ID. The selected mode, order policy, and actual
processing order appear in the batch result and owner report.

## Agent Consistency

`AgentCrossSourceConsistencyTable` contains one deterministic row for each
active agent. It summarizes source-kind coverage, positive and negative paper
net-reward counts, voice-delta range, reward/penalty range, high-confidence
misses, avoided losses, cooldowns, and quarantines.

Statuses are:

- `Stable`: low observed range across at least two source kinds;
- `SourceSensitive`: moderate observed range;
- `Unstable`: large observed range requiring caution;
- `InsufficientData`: fewer than two sources or source kinds.

Source-sensitive is descriptive, not negative. Unstable means review the
paper evidence; it is not an execution failure.

## Safety And Limitations

Only synthetic or sanitized local CSV text is accepted. The diagnostic path
performs no download, network call, account lookup, broker call, order
placement, cancellation, runtime LLM operation, online learning, or live state
mutation. Chair processing and Risk Governor veto remain unchanged.

The expanded fixtures are small, fake examples. They do not represent real
market microstructure, exchange calendars, latency, liquidity, or
profitability. No live-readiness or profitability claim is made.

## Future Extension

Future paper-only work may add larger sanitized fixture packs, explicit
expected-interval metadata, and baseline ranges per source profile. Any live
provider or broker work requires a separate reviewed architecture and is not
part of these diagnostics.
