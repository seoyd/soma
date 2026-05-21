# Expansion Readiness Gates

Sprint 11 does **not** expand the league. It adds evidence gates for deciding whether expansion should even be considered later.

## Why expansion is blocked by default

Moving from 3 personas to 6 personas is only justified when current behavior is stable across multiple datasets and current personas appear meaningfully non-redundant.

## Decision states

- `NeedMoreExperiments`: not enough usable datasets or outcome records
- `ImproveDataFirst`: bad or unusable data is polluting conclusions
- `ImproveRiskGovernorFirst`: denial behavior is too unstable across runs
- `ImproveSignalModelFirst`: signal outcomes are too weak or too negative
- `HoldCurrentScope`: personas still look redundant
- `ExpandToSixPersonas`: only when evidence is broadly strong
- `Blocked`: reserved for hard stop conditions

## Required evidence for expansion

- multiple usable datasets
- stable feature schema
- clean leakage guard behavior
- stable Risk Governor behavior
- non-redundant persona contribution
- non-catastrophic baseline performance
- validated external comparison when used

If evidence is mixed, the system stays conservative.
