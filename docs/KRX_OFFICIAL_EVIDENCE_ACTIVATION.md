# KRX Official Evidence Activation

Sprint 49 adds a bounded, local-first KRX market-data activation flow.

## Scope

- market-data-only
- research-only and paper-only
- no live trading, broker, order, or account paths
- no secret storage or secret rendering

## Auth

The flow only reads env-var presence:

- `KRX_API_KEY`
- `KRX_ENDPOINT_TEMPLATE`

Reports only show env-var names and a redacted endpoint preview. Missing env vars block live KRX collection, but they do not prevent local fixture or local canonical CSV import.

## Activation flow

1. validate local-only config paths and bounded limits
2. build KRX auth readiness
3. load a compact KRX symbol whitelist
4. plan local import first, collection only when explicitly enabled
5. validate canonical CSV, provenance, preflight, and manifest artifacts
6. run official replication and conservative downstream reruns when configured
7. emit operator actions, storage report, and activation summary

## Outputs

Bundles are written under:

`target/soma_krx_official_activation/<activation_id>/`

Key artifacts:

- `krx_auth_readiness.txt`
- `krx_symbol_whitelist.txt`
- `krx_evidence_job_plan.txt`
- `krx_canonical_validation.txt`
- `krx_operator_actions.txt`
- `krx_downstream_rerun_summary.txt`
- `krx_official_evidence_activation_report.txt`
- `storage_report.txt`
- `krx_official_activation_summary.txt`

## Safety notes

- readiness does not imply strategy usefulness
- imported official rows do not imply profitability
- downstream reruns remain conservative when outcome links stay sparse
