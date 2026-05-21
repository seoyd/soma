# Committee decision quality

Sprint 34 adds `CommitteeDecisionQualityReport` to describe **how** the committee behaves without overclaiming profitability.

## Core metrics

- no-trade ratio
- approval-candidate ratio
- reduce-size ratio
- require-confirm ratio
- risk-denial ratio
- hard-veto ratio
- emergency-stop ratio
- cooldown ratio
- groupthink warning ratio
- high-disagreement ratio

## Interpretation

- `AllNoTrade` is not automatic failure
- `RiskBlockedDominant` is not automatic failure
- `TooMuchGroupthink` is a warning about committee diversity
- `TooMuchDisagreement` is a warning about coordination stability

The report remains **outcome-light** and does not claim profitability unless real outcome evidence exists elsewhere.

