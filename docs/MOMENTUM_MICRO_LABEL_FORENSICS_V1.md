# Momentum Micro Label Forensics V1

## Scope

Momentum Micro Label Forensics V1 is a post-result historical research
diagnostic. It reads only the existing Qualified-Six development and
validation event ranges and qualified closed 10-minute candles. The sealed
historical holdout remains closed.

Every result is labeled:

```text
HistoricalResearchOnly
PostResultResearchDesignOnly
MicroChallengerNotExecuted
HoldoutClosed
NotLiveAuthority
NotTradingAuthority
```

The diagnostic cannot train or execute a model, make a confirmation claim,
open a holdout, change live state, rank participants, apply reward or penalty,
invoke Chair behavior, or trade.

## Registration-before-read boundary

The registration fixes the source replay and diagnostic identities, protected
live-state identity, readable partitions, three candidate horizons, magnitude
and prevalence summaries, temporal grouping, overlap rules, serial lags, and
disposition thresholds. It is atomically persisted, reopened, decoded, and
validated before any target return is derived.

Only these partitions are readable:

```text
development
validation
```

The candidate horizons are:

| Horizon | Target | Diagnostic cadence |
|---|---|---|
| `NextTenMinutes` | One completed 10m candle | Completed 10m boundaries |
| `NextThirtyMinutes` | Three completed 10m candles | Non-overlapping 30m boundaries |
| `NextSixtyMinutes` | Six completed 10m candles | Non-overlapping 60m boundaries |

The 60-minute horizon is diagnostic-only and is not a registered challenger
task.

## Aggregate diagnostics

For each horizon and readable partition, the implementation derives finite
target-return magnitude summaries, positive/negative/neutral counts, and
scorable prevalence. It publishes aggregate values only; event timestamps,
event-level returns, and event-level labels are not public report fields.

Temporal stability is summarized deterministically by UTC day, UTC week, UTC
month, rolling 144 events, and rolling 1,008 events. Target-interval overlap is
audited with half-open intervals. Serial dependence is summarized at fixed
lags 1, 2, 3, 6, and 12 using scorable signs only.

The disposition policy is fixed before target access. A disposition remains a
future-screening design observation and never authorizes model execution.

## Persistence and replay

Registration, event plans, distributions, temporal summaries, overlap
receipts, serial-dependence receipts, dispositions, horizon reports, the
research journal, and the final public report use hand-written Protobuf
contracts and the existing verified atomic persistence path.

Identical completed execution reopens the final report with zero writes and
zero label computations. A conflicting or malformed artifact is rejected as
an integrity failure.

## CLI

```text
--momentum-micro-label-forensics --status --output-format text|json
--momentum-micro-label-forensics --dry-run --output-format text|json
--momentum-micro-label-forensics --execute-local --output-format json
```

The command accepts no network or unrelated execution authority. It recomputes
the completed live-lane identity before and after the operation and requires
the two protected states to be identical. The historical identity freezes all
pre-existing historical, Qualified-Six replay, and Qualified-Six diagnostic
artifacts while excluding only the two runtime roots owned by this research
operation, avoiding a self-referential digest.
