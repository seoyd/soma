# Conservative Downgrade Evidence

Sprint 69 adds a downgrade evidence audit so diagnostic downgrade or retirement is only treated as justified when enough static evidence is present.

## Required evidence

Depending on the recommendation, the audit checks for combinations of:

- regression evidence
- risk evidence
- calibration evidence
- coverage evidence
- comparability evidence
- artifact completeness evidence
- decision conflict evidence
- owner review evidence

## Conservative rule

If evidence is incomplete, the output should prefer more review or more predictions over a stronger downgrade claim.

