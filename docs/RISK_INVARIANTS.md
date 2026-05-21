# Risk invariants

Sprint 23 does not add new risk policy. It adds explicit verification around the existing `RiskGovernor`.

## Checked invariants

- deny by default
- veto remains absolute
- missing stop loss is denied
- negative edge is denied
- low data quality is denied
- invalid prediction path is denied
- schema mismatch is treated as denied
- emergency stop blocks all
- cooldown blocks new entries
- external model signal cannot bypass the governor

## Important interpretation

Passing this report does **not** mean profitable trading.

It only means later research layers still route through the same conservative veto path.
