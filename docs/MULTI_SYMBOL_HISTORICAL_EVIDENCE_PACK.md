# Multi-Symbol Historical Evidence Pack

## Purpose

Single-symbol proof is not enough to trust a committee rule. A result can be
symbol-specific, market-regime-specific, or a fixture artifact. The historical
evidence pack asks a broader falsifiable question: does the current
three-agent committee beat trivial baselines across multiple owner-provided
sanitized local daily CSV sources?

This is local-only, read-only, and paper-only. It is not a profitability claim
and not live trading readiness.

## Manifest Schema

`HistoricalEvidencePackManifest` contains:

- `pack_id`
- `description`
- `sources`
- `local_only`
- `sanitized_only`
- `reason_codes`

Each `HistoricalEvidenceSourceSpec` contains:

- `source_id`
- `source_kind`
- `symbol`
- `market`
- optional `currency`
- optional `csv_path`
- optional `csv_text` for deterministic tests
- `enabled`
- `expected_min_rows`
- `reason_codes`

Supported source kinds are `UsStockDaily`, `KoreanStockDaily`,
`BtcCryptoDaily`, and test-only `SyntheticDailySample`. Unknown or disallowed
source kinds are rejected with reason codes and remain visible in the report.

## Local And Sanitized Rules

Production evidence uses owner-provided local CSV paths. The loader does not
download data, scrape the web, call provider APIs, require API keys, or open a
broker path. Test code may provide `csv_text` directly.

The loader rejects URL-like paths, environment-file paths, private local
paths, temporary instruction-file paths, endpoint-looking values, live-provider
markers, secrets, account IDs, order IDs, raw provider responses, and private
mapping markers.

Large real datasets are not committed by default. If small committed samples
are needed, they must be synthetic or explicitly sanitized examples.

## Source Examples

US stock source:

- `source_kind=UsStockDaily`
- `symbol=AAPL`
- `market=US`
- `currency=USD`
- `csv_path=data/historical/sanitized/us/aapl_daily.csv`

Korean stock source:

- `source_kind=KoreanStockDaily`
- `symbol=005930.KS`
- `market=KR`
- `currency=KRW`
- `csv_path=data/historical/sanitized/kr/005930_daily.csv`

BTC crypto source:

- `source_kind=BtcCryptoDaily`
- `symbol=BTC-USD`
- `market=BTC`
- `currency=USD`
- `csv_path=data/historical/sanitized/crypto/btc_usd_daily.csv`

These paths are examples only. The implementation supports local owner paths
and does not include a downloader.

## Evaluation Flow

1. Load each source independently.
2. Validate daily CSV rows through the manual historical daily importer.
3. Convert each accepted source to a one-day candle series.
4. Run the existing walk-forward proof gate per symbol.
5. Keep rejected, disabled, and insufficient sources visible.
6. Aggregate after every source has its own out-of-sample result.

Ordering is deterministic by source kind, market, symbol, and source ID.

## Aggregation Logic

The pack aggregates four baselines:

- `AlwaysNoTrade`
- `BuyAndHold`
- `EqualWeightCommittee`
- `VoiceAdaptiveCommittee`

The current implementation uses mean aggregation for total return, max
drawdown, and Brier score. It separately reports:

- voice-adaptive wins, losses, and ties versus equal weight,
- committee wins and losses versus no-trade,
- committee wins and losses versus buy-and-hold,
- market-level results,
- symbol-level failures,
- rejected sources,
- insufficient-evidence warnings.

Market-level evidence remains separated so US, KR, and BTC results cannot hide
one another.

## Status Definitions

`Pass` means every accepted source supports the voice-adaptive committee over
equal weight and the committee beats both no-trade and buy-and-hold under the
computed risk-aware comparison.

`Fail` means the evidence does not show that edge.

`Mixed` means at least one source helps and at least one comparison fails or
ties.

`InsufficientEvidence` means accepted out-of-sample sources or prediction
samples are below the configured minimum.

Bad, failed, mixed, and insufficient results are valid outputs. The report must
show them rather than average them away.

## Owner Evidence Trial

`run_owner_historical_evidence_trial` is the owner-facing orchestration layer
over this pack loader and evaluator. It accepts a local manifest path or
test-only manifest JSON, then returns:

- `NoOwnerEvidencePackFound` when no local pack is supplied,
- `RejectedForSafety` when unsafe paths or private data are detected,
- `InsufficientEvidence` when source, row, or prediction samples are too thin,
- `Fail`, `Mixed`, or `Pass` from computed proof-gate results.

The trial keeps rejected, disabled, insufficient, failed, and mixed sources
visible in an owner-readable triage report. It also separates US, KR, and BTC
market-level results so market failures cannot be hidden by aggregate means.

## Boundaries

The evidence pack adds no network client, downloader, broker, order placement,
order cancellation, runtime LLM, online learning, heavy model, live mutation,
or eight-agent activation. Risk Governor review remains part of the underlying
walk-forward evaluator and invalid data leads to rejection or no-trade
behavior, not to proof.
