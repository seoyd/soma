# Committee V1 readiness

`CommitteeV1ReadinessReport` is the gate between diagnostics and broader committee research use.

## Readiness states

- `ReadyForMoreEvidence`
- `ReadyForCommitteeBenchmark`
- `ReadyForChairTuning`
- `ReadyForRiskReview`
- `ReadyForSixPersonaDesignReviewOnly`
- `NotReadyEvidenceTooWeak`
- `NotReadyResearchOnly`
- `NotReadyFixtureOnly`
- `NotReadyRiskUnstable`
- `NotReadyGroupthink`
- `NotReadyTooFewSamples`

## Boundaries

- yfinance-only cannot pass official readiness
- fixture-only cannot pass Committee V1 readiness
- six-person output is still **design-review-only**
- nothing here activates 6 personas

The gate stays conservative when evidence quality, disagreement quality, or risk stability is weak.

