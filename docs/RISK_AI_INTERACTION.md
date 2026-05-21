# Risk AI interaction

Sprint 21 reports how the model path and Risk Governor interact instead of treating denials as opaque side effects.

## Reported metrics

- `total_signals`
- `approved_candidates`
- `denied_by_risk`
- `no_trade_by_signal`
- `no_trade_by_risk`
- `emergency_stop_count`
- `cooldown_count`
- `avoided_loss_count`
- `missed_gain_count`
- `defensive_value`
- `opportunity_cost`
- `denial_rate`
- `approval_rate`

## How to read it

- high denial with meaningful avoided losses can be acceptable
- high denial with little defensive value suggests overtight risk settings
- low denial with elevated drawdown is a warning
- approval alone is not good if drawdown control is weak

## Principle

Risk denial is not automatically bad and model activity is not automatically good. The benchmark treats Risk Governor stability as a required condition for any usefulness claim.
