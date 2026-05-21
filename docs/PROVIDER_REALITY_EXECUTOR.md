# Provider reality executor

`ProviderRealityEvidenceExecutor` is the Sprint 31 orchestrator.

## Flow

1. Load provider reality or explicit lane config.
2. Build an `ExecutableEvidencePlan`.
3. Enforce bounded storage budget.
4. Run each lane in deterministic order.
5. Build `EvidenceReadinessMatrix`.
6. Emit operator actions and final recommendation.

## Execution behavior

- core-check is run before benchmark-capable lanes
- diagnostics lanes never claim benchmark success
- yfinance lanes remain research-only
- skipped lanes are first-class outputs, not silent failures

## Operator actions

The executor emits local-only, research-only follow-up actions such as:

- waiting for KRX approval
- setting KRX / data.go.kr / AlphaVantage / Alpaca auth
- buying or configuring bounded realtime entitlement
- using Upbit crypto-only evidence while equity gaps remain

