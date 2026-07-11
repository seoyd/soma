# Owner Evidence Runbook

## Local Files Only

Place sanitized daily CSV files below `data/historical/sanitized/` and the
private manifest below `data/historical/evidence_packs/`. Both locations are
ignored by default. Do not commit owner CSV files, private manifests, or
generated owner-data reports.

The local runner checks these manifest names in order when no manifest override
is supplied:

- `data/historical/evidence_packs/owner_pack.local.json`
- `data/historical/evidence_packs/owner_pack.json`
- `data/historical/evidence_packs/first_owner_pack.local.json`

An explicit `OwnerEvidenceTrialConfig.manifest_path` remains available for a
safe local override. URL paths, environment files, private paths, endpoint-like
paths, and temporary instruction-file paths are rejected.

## CSV and Manifest

Each CSV must contain `symbol,date,open,high,low,close,volume`, with `date` in
`YYYY-MM-DD` form. The manifest identifies the local CSV path, market, symbol,
currency, source kind, and expected minimum row count. Use only sanitized daily
bars for US, KR, and BTC evidence.

Never include credentials, Authorization or Bearer material, account IDs, order
IDs, API keys, raw provider responses, endpoint columns, live-provider fields,
or URLs. No API key or private provider document is needed for this flow.

## Running the Local Trial

The library entry point discovers the approved local candidates, invokes the
existing owner evidence trial, renders its text, and optionally stores it:

```rust
let run = soma_zero::run_owner_historical_evidence_trial_from_local_candidates(
    soma_zero::OwnerEvidenceTrialConfig::default(),
    soma_zero::OwnerEvidenceReportEmissionConfig::default(),
);
```

The default emission configuration writes only to
`reports/local/evidence_trials/owner_evidence_triage_report.txt`. That report
directory is ignored by default. Set `write_report` to `false` to return only
the report text and hash, without creating a file. A custom output directory
requires explicit opt-in and source, documentation, test, fixture, example,
and Cargo paths are rejected.

## Result Meanings

- `NoOwnerEvidencePackFound`: no candidate or supplied local manifest was
  available; no data was evaluated and the action checklist explains the next
  local files required.
- `RejectedForSafety`: a path, manifest, CSV, or report text contained unsafe
  material; no evaluation result is invented.
- `InsufficientEvidence`: accepted sources or samples do not meet the proof
  gate requirements.
- `Fail`: the computed committee comparison or voice adaptation lost.
- `Mixed`: symbols or markets disagree.
- `Pass`: the configured local historical proof gate passed. It remains a
  paper-only result.

The report always exposes rejected sources, failed symbols, market splits,
baseline losses, and VoiceAdaptiveCommittee failures. It makes no profitability
claim and no live-readiness claim.

## Safety Boundary

This path is local filesystem input and local text output only. It has no
downloader, network client, broker, order, cancellation, runtime model, or
live execution behavior. The existing Risk Governor and paper-only evaluation
boundaries remain unchanged.
