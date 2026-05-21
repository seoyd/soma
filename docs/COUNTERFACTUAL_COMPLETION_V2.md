# Counterfactual Completion V2

Counterfactual completion v2 depends on outcome linkage.

- `NoTrade` and `RiskDenied` counterfactuals are only built when an outcome reference exists or can be loaded safely.
- `avoided_loss_value` credits defensive value when the realized outcome is negative.
- `missed_gain_value` records the opportunity cost when the realized outcome is positive.
- Risk-denied completion requires a deny/block/reject style risk decision; it is not guessed from missing data.
- Diagnostic, yfinance, fixture, and crypto-only boundaries are preserved.
