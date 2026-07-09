# Restart Sprint 19 Report

## Verification

Baseline verification began from clean commit `cab0c27`. Initial formatting and
workspace compile passed. The first full test run exposed a transient
integration-test target artifact ordering issue; rerunning the full workspace
test passed before feature work continued.

Focused Sprint 19 tests passed after implementation. Final full verification
also passed:

- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test --workspace --quiet`
- `git diff --check`

The final test run passed 526 tests: 110 library tests, 404 integration tests,
and 12 additional integration tests. Two pre-existing unused-private-function
warnings remain.

## Manual Historical Import

Added a local-only `ManualHistoricalDailyDataset` path with
`parse_manual_historical_daily_csv`, validation, and `to_daily_candle_series`.
The parser supports US stock, Korean stock, and BTC crypto daily CSV contracts
without downloader, API key, network, broker, account, or order integration.

## Walk-Forward Harness

Added deterministic expanding-train/fixed-eval splits with an explicit
no-lookahead invariant. Evaluation decisions use only data available at the
decision row; directional outcomes use the next close.

## Baselines

Implemented computed metrics for:

- `AlwaysNoTrade`,
- `BuyAndHold`,
- `EqualWeightCommittee`,
- `VoiceAdaptiveCommittee`.

Risk Governor config is applied to trade attempts without creating a broker or
order path.

## Brier Scoring

Added `PredictionQualitySample` and `compute_prediction_quality_metrics` with
Brier score, missing probability counts, abstention counts, high-confidence
errors, low-confidence correct calls, and insufficient-sample handling.

## Proof Gate Report

Added `build_proof_gate_report` and `render_proof_gate_report_text`. The report
states local historical daily CSV only, paper-only evaluation, no live trading
readiness, no profitability claim, voice-adaptive vs equal-weight comparison,
and synthetic fixture success is not market evidence.

## Tests

Added deterministic tests for:

- valid manual daily CSV import,
- invalid date, duplicate, non-monotonic, multi-symbol, invalid OHLC, and
  non-finite rejection,
- private/secret/account/order/raw/endpoint/live/temporary-instruction marker
  rejection,
- deterministic no-lookahead walk-forward splits,
- four-baseline evaluation and computed comparison booleans,
- Brier score and missing/abstention/high-confidence accounting,
- proof gate report safety warnings and determinism.

## Hardcoding Audit

Proof gate status, winners, returns, risk-adjusted comparisons, and Brier
scores are computed from input rows and config. Tests assert comparison booleans
against computed metric expressions rather than expected winners.

## Risk And Security Review

No live trading, downloader, web scraping, Toss live provider, exchange live
provider, broker/order path, runtime LLM, online learning, heavy AI model, UI,
database, cloud path, or eight-agent activation was added. Active agents remain
the existing three-agent set.

## Deferred Items

Real exchange calendars, timezone/session validation, large real dataset
curation, filesystem loader policy, multi-symbol portfolio evaluation,
individual agent sleeve reporting, real execution, live APIs, online learning,
heavy model training, UI, persistence, and deployment remain deferred.

## Next Sprint

Use owner-provided sanitized local daily CSV files across US, KR, and BTC
profiles and run the proof gate on multiple symbols. Keep the same no-download,
paper-only boundary and focus on whether committee behavior beats null and
passive baselines out of sample.
