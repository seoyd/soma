# Counterfactual Depth Plan

The counterfactual depth plan reports why comparable rows are still shallow.

It tracks:
- missing outcome references
- missing baseline references
- missing NoTrade / RiskDenied counterfactuals
- missing local candles or timestamp alignment
- no-lookahead violations
- source-ineligible rows
- summary-derived-only rows

Each gap is marked as buildable or unavailable and points to the safest next builder or manual operator action.
