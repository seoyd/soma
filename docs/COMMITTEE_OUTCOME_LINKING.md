# Committee Outcome Linking

Sprint 36 adds explicit outcome linking for committee scenario rows.

## What gets linked

- triple-barrier outcomes
- no-trade counterfactuals
- risk-denied counterfactuals
- baseline actions
- external prediction actions when schema-valid

## Safety rules

- matching is deterministic
- symbol matching stays explicit
- horizon matching stays explicit
- timestamp tolerance is configurable, never hidden
- `no_lookahead_safe = false` blocks readiness

## Interpretation

Outcome-linked evidence is stronger than summary-derived evidence, but it is still research-only and does not prove profitability or live-trading readiness.

