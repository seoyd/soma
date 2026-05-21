# Committee counterfactual audit

Sprint 37 adds deterministic local-only counterfactual construction.

Supported counterfactuals:
- `NoTrade`: what happened if the committee stayed out,
- `RiskDenied`: what happened if the Risk Governor veto stood.

Audit output reports:
- build status (`Built`, unavailable, estimated diagnostic-only, rejected),
- conservative avoided-loss value,
- missed-gain value,
- excursion metrics,
- no-lookahead safety.

Rules:
- local candle paths only,
- exact symbol/timestamp/horizon matching preferred,
- missing candle data is reported as unavailable,
- estimated matches are diagnostic-only,
- unsafe no-lookahead references are rejected.
