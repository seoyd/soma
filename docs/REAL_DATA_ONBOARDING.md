# Real data onboarding

Sprint 17 adds a **local-only onboarding flow** for real market CSV data.

## Flow

1. put a user-provided CSV under `data/local/`
2. run `data-preflight` or `onboard-data`
3. inspect the deterministic preflight report
4. only rerun `real-evidence` if preflight reaches `ReadyForRealEvidence`

## What onboarding checks

- local path only
- file exists
- CSV profile detection
- required columns
- row parsing and OHLC invariants
- ordering / duplicates / gaps
- data quality
- walk-forward feasibility
- triple-barrier feasibility
- conservative evidence-target estimate
- real-local provenance eligibility

## Local-only policy

- no downloader
- no exchange API
- no broker integration
- no runtime LLM
- no live trading recommendation

Actual real CSV data is still **user-supplied** and **not downloaded by Soma**.
