# Restart Sprint 21 Report

## Verification

Baseline verification was run before feature work at commit `e5fb6ca`:

- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test --workspace --quiet`
- `git diff --check`
- `git diff --cached --check`

The baseline was green. Existing dead-code warnings in `persona_card.rs` were
unchanged.

The sprint acceptance verification is the same command set after all source
and documentation changes.

## Evidence Trial Runner

Sprint 21 adds `run_owner_historical_evidence_trial` and
`OwnerEvidenceTrialConfig` as the owner-facing orchestration layer over the
Sprint 20 evidence pack path.

The runner accepts either:

- a local `manifest_path`, or
- test-only `manifest_json`.

It reuses the existing manifest loader, evidence pack validator,
multi-symbol evaluator, and proof gate report builder. It does not duplicate
walk-forward logic.

## No-Pack Behavior

When no owner manifest is supplied, the runner returns
`NoOwnerEvidencePackFound`.

No fake source, fake pack, fake market result, or synthetic success is created.
The result contains no pack evaluation and no multi-symbol proof report. It
does contain a deterministic owner action checklist.

## Triage Statuses

The owner trial status set is:

- `NoOwnerEvidencePackFound`
- `RejectedForSafety`
- `InsufficientEvidence`
- `Fail`
- `Mixed`
- `Pass`

The implementation is conservative. Safety rejection dominates. Insufficient
evidence prevents a pass. Baseline and voice failures remain visible.

## Market-Level Triage

`MarketTriageResult` reports each present market separately:

- source count,
- accepted count,
- rejected count,
- insufficient count,
- voice status,
- committee versus no-trade status,
- committee versus buy-and-hold status,
- Brier/prediction-quality status,
- market status,
- reason codes.

US, KR, and BTC results are not averaged into a single opaque conclusion.

## Report Renderer

`render_owner_evidence_triage_report_text` renders an owner-readable report
with:

- trial status,
- pack summary,
- source summary,
- market triage,
- baseline failure summary,
- voice adaptation summary,
- prediction-quality summary,
- failed symbols,
- rejected sources,
- insufficient evidence reasons,
- owner action checklist,
- safety warnings,
- no-profitability and no-live-readiness warnings.

The renderer states that evaluation is local owner-provided sanitized daily
CSV only, paper-only, no data downloaded, no profitability claim, and no live
trading readiness.

## Owner Checklist

`OwnerActionItem` records deterministic next actions for missing or
insufficient packs:

- provide US daily CSV evidence,
- provide KR daily CSV evidence,
- provide BTC daily CSV evidence,
- use `YYYY-MM-DD`,
- include required OHLCV columns,
- remove account, order, API, private, raw-response, endpoint, and
  live-provider columns,
- keep files local,
- do not paste API keys, broker credentials, private provider documents, or
  temporary instruction files.

## Hardcoding Audit

New reason codes and tests preserve the hardcoding ban:

- status changes when input metrics change,
- report language changes when voice validity changes,
- no hardcoded evidence status is accepted,
- no hardcoded market result is accepted,
- no hardcoded VoiceAdaptive success claim is accepted.

## Tests

Sprint 21 adds deterministic coverage for:

- no owner pack returning `NoOwnerEvidencePackFound`,
- no-pack report saying no data was evaluated,
- owner checklist generation,
- valid test JSON pack evaluation,
- market triage generation,
- deterministic report rendering,
- unsafe manifest paths,
- rejected source visibility,
- account-data rejection,
- raw-provider-response rejection,
- computed pass status,
- computed fail status,
- computed mixed status,
- computed insufficient evidence status,
- baseline failures appearing in report text,
- market-level pass/fail split.

Existing Sprint 20 tests continue to cover URL, environment-file, private path,
source-level endpoint, order, authorization, live endpoint, temporary
instruction marker, PaperBroker, Risk Governor, three active agents, and
inactive future eight-agent placeholders.

## Risk And Security Review

The sprint adds no live broker path, no real order placement, no order
cancellation, no Toss live path, no exchange live path, no downloader, no web
scraping, no runtime LLM, no online learning, no heavy AI runtime, no live
mutation, and no eight-agent activation.

Evidence remains local-only, deterministic, read-only, and paper-only. Unsafe
data is rejected or kept visible as rejected evidence when source-level
triage is possible.

## What Was Proven

The code can now distinguish these states for an owner evidence trial:

- no pack supplied,
- pack or path rejected for safety,
- evidence insufficient,
- computed failure,
- computed mixed result,
- computed pass.

It can also render source-level, market-level, baseline, voice, and prediction
quality triage without hiding losing symbols or rejected sources.

## What Failed, Mixed, Insufficient, Or No-Pack Means

`Fail` means the committee or voice adaptation did not beat required
baselines.

`Mixed` means pass and fail evidence coexist across sources or markets.

`InsufficientEvidence` means the configured source, row, or prediction sample
minimums were not met.

`NoOwnerEvidencePackFound` means no local owner manifest or data was supplied,
and no evaluation was fabricated.

## Deferred Items

Deferred items remain:

- real data collection,
- provider integrations,
- live API smoke tests,
- broker integration,
- order placement,
- order cancellation,
- runtime LLM,
- heavy AI models,
- online learning,
- database storage,
- web UI,
- cloud deployment,
- full eight-agent activation.

## Next Sprint

The next useful gstack sprint should place a real owner-provided sanitized
local evidence pack on disk and run this trial. The goal should be to review
the actual no-pack, rejected, insufficient, failed, mixed, or pass output
before considering model complexity or agent expansion.
