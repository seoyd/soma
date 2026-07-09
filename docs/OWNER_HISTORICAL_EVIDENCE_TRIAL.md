# Owner Historical Evidence Trial

## Purpose

The owner evidence trial runs the existing multi-symbol historical evidence
pack gate on owner-provided sanitized local daily CSV files. It answers one
bounded question: what does the current paper-only proof gate say when local
owner evidence is supplied?

This is local-only, read-only, deterministic, and paper-only. It is not a
profitability claim and not live trading readiness.

## Local Directory Contract

Preferred local directories:

- `data/historical/sanitized/`
- `data/historical/evidence_packs/`

Real owner CSVs and private manifests are ignored by default. Example manifests
may be committed only when they contain placeholders or synthetic examples.

## Manifest Example

Example-only manifest shape:

```json
{
  "pack_id": "owner-local-pack",
  "description": "Owner-provided sanitized daily CSV evidence pack",
  "local_only": true,
  "sanitized_only": true,
  "sources": [
    {
      "source_id": "owner-us-aapl-daily",
      "source_kind": "UsStockDaily",
      "symbol": "AAPL",
      "market": "US",
      "currency": "USD",
      "csv_path": "data/historical/sanitized/us/aapl_daily.csv",
      "enabled": true,
      "expected_min_rows": 120,
      "reason_codes": []
    },
    {
      "source_id": "owner-kr-005930-daily",
      "source_kind": "KoreanStockDaily",
      "symbol": "005930.KS",
      "market": "KR",
      "currency": "KRW",
      "csv_path": "data/historical/sanitized/kr/005930_daily.csv",
      "enabled": true,
      "expected_min_rows": 120,
      "reason_codes": []
    },
    {
      "source_id": "owner-btc-usd-daily",
      "source_kind": "BtcCryptoDaily",
      "symbol": "BTC-USD",
      "market": "BTC",
      "currency": "USD",
      "csv_path": "data/historical/sanitized/crypto/btc_usd_daily.csv",
      "enabled": true,
      "expected_min_rows": 120,
      "reason_codes": []
    }
  ],
  "reason_codes": []
}
```

CSV rows must include:

- `symbol`
- `date`
- `open`
- `high`
- `low`
- `close`
- `volume`

`date` must use `YYYY-MM-DD`.

## Runner

`run_owner_historical_evidence_trial` accepts `OwnerEvidenceTrialConfig` with
either a local `manifest_path` or test-only `manifest_json`. It reuses:

- `load_historical_evidence_pack_from_manifest`
- `validate_historical_evidence_pack`
- `evaluate_historical_evidence_pack`
- the Sprint 20 multi-symbol proof gate

The runner does not download data, call network APIs, invoke a broker, place
orders, cancel orders, or call a runtime model.

## No-Pack Behavior

If no manifest path or JSON is supplied, the result is
`NoOwnerEvidencePackFound`.

No fake evidence is generated. No source is evaluated. The report returns an
owner action checklist explaining what local evidence is required next.

## Triage Statuses

- `Pass`: enough accepted evidence exists and the computed comparisons support
  the committee under the configured proof gate.
- `Fail`: enough accepted evidence exists, but voice adaptation or committee
  baseline comparisons fail.
- `Mixed`: wins and failures coexist across symbols or markets.
- `InsufficientEvidence`: source count, row count, or prediction-quality
  samples are below the configured threshold.
- `NoOwnerEvidencePackFound`: no local owner manifest or pack was supplied.
- `RejectedForSafety`: unsafe paths, private markers, secrets, account data,
  order data, raw provider responses, live-provider markers, endpoint markers,
  or URLs were found.

Bad, failed, mixed, insufficient, rejected, and no-pack results are valid
outputs. They must be shown rather than averaged away.

## Market-Level Triage

US, KR, and BTC evidence is summarized separately when present. One market
cannot hide another market's failure. If BTC passes while US fails, the report
must show the split result.

The owner trial can configure minimum accepted source counts per market through
`min_sources_by_market`.

## Failure Visibility

The report makes these failures explicit:

- symbols that failed,
- symbols with insufficient evidence,
- markets that failed or were mixed,
- sources rejected for safety,
- VoiceAdaptiveCommittee losing to EqualWeightCommittee,
- BuyAndHold beating the committee,
- AlwaysNoTrade beating the committee,
- weak prediction-quality evidence.

VoiceAdaptiveCommittee must beat EqualWeightCommittee before it is trusted.

## Owner Action Checklist

When evidence is missing or insufficient, the checklist asks the owner to:

- provide at least the configured number of US daily CSV files,
- provide at least the configured number of KR daily CSV files,
- provide BTC daily CSV evidence,
- use `YYYY-MM-DD` dates,
- include the required OHLCV columns,
- remove account, order, API, private, raw-response, endpoint, and
  live-provider columns,
- keep files local,
- avoid API keys, broker credentials, private provider documents, and temporary
  instruction files.

The system must not ask the owner to paste API keys, broker credentials, or
private provider documents.

## Safety Rules

The trial is a read-only evidence path. It adds no downloader, network client,
broker integration, order path, cancellation path, live provider, runtime LLM,
online learning, heavy AI model, live mutation, or eight-agent activation.

Risk Governor behavior remains inside the reused walk-forward evaluator, and
invalid data leads to rejection or no-trade behavior, not execution.

## Claims Boundary

The report always says:

- local owner-provided sanitized historical daily CSV only,
- paper-only evaluation,
- no data was downloaded,
- no profitability claim,
- no live trading readiness.
