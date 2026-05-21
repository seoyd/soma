# Official Committee Scenario Pack

Sprint 36 adds an official committee scenario pack layer on top of Sprint 35 materialization.

## Requirements

- local paths only
- bounded rows, symbols, and bytes
- provenance required by default
- preflight required by default
- yfinance excluded by default
- fixture excluded by default

## Source boundaries

- `OfficialApiCollected` contributes to official readiness
- official crypto/public evidence remains crypto-only
- yfinance remains research-only
- fixture remains architecture-test-only

## Pack behavior

- row-level artifacts are preferred
- summary-derived rows are allowed only when explicitly enabled
- storage bytes are counted in the pack
- outcome/baseline/external counters are surfaced for downstream linking and benchmark review

