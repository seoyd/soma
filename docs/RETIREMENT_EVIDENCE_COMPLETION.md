# Retirement Evidence Completion

Sprint 71 separates **retirement evidence** from **diagnostic downgrade support**.

## Retirement vs diagnostic downgrade

- retirement means the version is excluded from the current comparison set
- retirement does **not** mean deletion
- diagnostic downgrade is the conservative fallback when evidence is still incomplete

## Required evidence

- owner retirement rationale
- conservative regression evidence
- historical leaderboard or comparable baseline evidence
- a newer local successor comparison

## Conservative behavior

If evidence is still incomplete, the system keeps the model version diagnostic-only or blocked for review instead of over-claiming that retirement is fully complete.
