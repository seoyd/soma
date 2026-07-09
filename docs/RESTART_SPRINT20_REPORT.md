# Restart Sprint 20 Report

## Verification

Baseline verification was run before feature work:

- `git status --short`
- `git rev-parse --short HEAD`
- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test --workspace --quiet`
- `git diff --check`
- `git diff --cached --check`

The baseline was green at commit `dc2b7d5`; existing dead-code warnings were
unchanged.

## Evidence Pack Manifest

Sprint 20 adds a typed multi-symbol historical evidence pack manifest:

- `HistoricalEvidencePackManifest`
- `HistoricalEvidenceSourceSpec`
- `HistoricalEvidencePackConfig`
- `HistoricalEvidenceSourceKind`

The manifest supports US stock daily CSV, Korean stock daily CSV, BTC crypto
daily CSV, and test-only synthetic daily samples. Source ordering is
deterministic by source kind, market, symbol, and source ID.

## Source Loading And Validation

`load_historical_evidence_pack_from_manifest` loads owner-provided local CSV
paths or test-only CSV text. It does not download data, call network APIs,
execute provider code, or require credentials.

`validate_historical_evidence_pack` checks local-only and sanitized-only
boundaries, minimum source counts, per-kind minimums, and optional all-source
validity. Rejected and disabled sources stay visible.

Safety rejection covers URL-like paths, environment-file paths, private local
paths, temporary instruction-file paths, endpoint markers, live-provider
markers, secrets, account IDs, order IDs, and raw provider response markers.

## Multi-Symbol Walk-Forward

`evaluate_historical_evidence_pack` runs each accepted source through the
existing Sprint 19 walk-forward evaluator. Each symbol gets its own
out-of-sample result before aggregation. No lookahead rules and Risk Governor
review remain preserved by the reused evaluator.

## Four-Baseline Aggregation

`AggregateBaselineComparison` records source counts, rejected and insufficient
counts, voice-adaptive wins/losses/ties against equal weight, committee
results against no-trade and buy-and-hold, and mean total return, drawdown,
and Brier score by baseline.

Statuses are computed from input metrics:

- `Pass`
- `Fail`
- `Mixed`
- `InsufficientEvidence`

No winner, proof status, P&L, or symbol outcome is hardcoded.

## Voice Adaptation Validity

`VoiceAdaptationValidity` skeptically compares `VoiceAdaptiveCommittee` with
`EqualWeightCommittee` across accepted sources. It reports helped, failed,
mixed, or insufficient evidence. The report explicitly says when voice
adaptation did not beat equal weight or when evidence is mixed or insufficient.

## Prediction Quality Aggregation

`AggregatePredictionQualitySummary` aggregates Brier-oriented prediction
quality across accepted sources. It counts samples, missing probabilities,
abstentions, high-confidence errors, best and worst source by Brier score, and
insufficient evidence.

Prediction quality remains separate from realized P&L.

## Proof Gate Report

`MultiSymbolProofGateReport` and
`render_multi_symbol_proof_gate_report_text` produce an owner-readable report
with:

- pack summary,
- source table,
- market table,
- aggregate baseline comparison,
- voice adaptation validity,
- prediction-quality summary,
- failed symbols,
- rejected sources,
- insufficient-evidence warnings,
- next required evidence.

The report states local owner-provided sanitized historical daily CSV only,
paper-only evaluation, no live trading readiness, no profitability claim,
synthetic fixture success is not market evidence, voice adaptation must beat
equal weight before it is trusted, and bad or mixed results are valid outputs.

## Hardcoding Audit

Tests construct alternate input metrics and verify that voice validity and
aggregate status change when the metrics change. The implementation computes
status from source results and config rather than fixed winners.

## Tests

New deterministic coverage includes:

- valid US/KR/BTC multi-symbol pack loading,
- manifest JSON parsing,
- per-source walk-forward execution,
- aggregate baseline comparison,
- market and symbol result tables,
- deterministic report rendering,
- disabled and unsupported source handling,
- rejected source visibility,
- URL, environment-file, private path, temporary instruction-file path,
  account, order, authorization, raw response, and live endpoint rejection,
- insufficient source validation,
- voice helped, failed, and mixed statuses,
- buy-and-hold and no-trade stronger cases,
- Brier aggregation with missing probabilities, abstentions, and
  high-confidence errors,
- PaperBroker live execution remaining unsupported,
- active agent count remaining three,
- future eight-agent placeholders remaining inactive.

## Risk And Security Review

The sprint adds no live broker path, no order path, no cancellation path, no
Toss live path, no exchange live path, no downloader, no web scraping, no
runtime LLM, no online learning, no heavy AI runtime, no live mutation, and no
eight-agent activation.

Evidence pack evaluation remains local-only, deterministic, read-only, and
paper-only.

## What Was Proven

The code can now load a sanitized local multi-symbol daily CSV evidence pack,
run each accepted source through the existing walk-forward proof gate, and
report cross-source four-baseline evidence without hiding failures.

## What Failed Or Remained Insufficient

This sprint does not prove market edge. It only creates the local evidence
pack mechanism and the computed proof gate. Real owner-provided historical
datasets still need to be supplied and evaluated.

## Deferred Items

Deferred items include real data collection, provider integrations, live API
smoke tests, broker integration, order placement, order cancellation, runtime
LLM, heavy AI models, online learning, database storage, web UI, cloud
deployment, and eight-agent activation.

## Next Sprint

The next useful sprint should run the new evidence pack against broader
owner-provided sanitized local daily CSV files and review the resulting failed,
mixed, or insufficient evidence before considering any model or agent
expansion.
