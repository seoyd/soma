# Batch Paper Replay

## Purpose

`run_local_dataset_batch_replay` runs multiple sanitized local CSV datasets
through the existing source registry, historical adapter, three-agent paper
replay, and owner-report path. The result is deterministic, read-only, and
paper-only. It does not establish profitability or live-trading readiness.

Supported source kinds are:

- `SyntheticFixture`
- `KoreanStockCsv`
- `UsStockCsv`
- `BtcCryptoCsv`

`Unknown` is rejected.

## Processing Path

For each enabled source, the batch runner:

1. validates metadata and private-data markers,
2. resolves the canonical `LocalDataSourceProfile`,
3. parses and normalizes CSV text into `HistoricalReplayDataset` and
   `CandleSeries`,
4. runs the existing Chair and Risk Governor paper replay with exactly three
   agents,
5. builds the existing owner learning report,
6. carries the resulting three agent states into the next accepted source,
7. records source and agent performance rows.

The runner uses no randomness or wall clock. Source rows are sorted by source
kind and source ID. Agent rows are sorted by source kind, source ID, and agent
ID. State transitions follow input order.

## Accepted And Rejected Sources

`require_all_sources_valid` or `stop_on_source_error` makes a rejected source
stop the batch with its source ID and reason codes. When both settings are
false, the rejected source remains visible in `BatchReplaySourceResult` and
`SourcePerformanceTable`, while later sources continue.

Disabled, unknown, malformed, oversized, non-monotonic, multi-symbol, private,
secret-like, account, order, raw-response, and unsupported sources are
rejected. Rejection never falls through to replay.

## Agent Performance

`AgentPerformanceTable` contains one row per accepted source and agent, overall
rows per agent, and rows per source kind and agent. Counts include selection,
support, opposition, abstention, Risk Governor alignment, correct and missed
`NoTrade`, learning deltas, rewards, penalties, voice changes, lifecycle
events, and sandbox candidates.

These values are replay observations. They are not real returns and do not
claim profitability.

## Source Performance

`SourcePerformanceTable` includes every accepted or rejected source. Accepted
rows expose data quality, timestamp range, symbol, close range, replay counts,
`NoTrade`, and risk denials. Every row sets `paper_only` and `not_live_ready`
to true. Counts by source kind support market-level inspection.

## Owner Report

`build_batch_owner_learning_report` produces an immutable batch summary with
source and agent tables, per-source report references, safety warnings, and
deferred items. The text renderer includes:

- paper-only and not-live-ready warnings,
- source summary,
- agent performance table,
- Risk Governor summary,
- sandbox summary,
- rejected source list,
- deferred live-readiness items.

Private-looking report lines are redacted.

## Safety Boundaries

The batch accepts caller-provided local CSV text only. It performs no file
discovery, download, network request, live provider call, account lookup,
broker call, order placement, or cancellation. `PaperBroker` remains the only
execution boundary in the wider paper system. Chair evaluation and Risk
Governor veto remain unchanged, and owner reports cannot mutate state.

Runtime LLMs, online learning, live self-mutation, heavy model training, and
eight-agent activation are outside this path.

## Limitations And Future Extension

Fixtures are synthetic and small. Results do not model production market-data
quality, exchange calendars, liquidity, latency, or live execution. A future
adapter may import separately sanitized local exports through the same
registry contract, but network providers and broker/account data require a
separate reviewed boundary.

This is batch paper replay infrastructure, not a complete AI trading system.
