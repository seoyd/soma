# Model Evidence Risk Profile

Sprint 65 adds **model evidence risk profiles** that summarize how conservative the offline evidence should be treated.

The profile combines:

- coverage ratio
- calibration and drift state
- risk-adjusted behavior
- ablation stability
- promotion gate state
- leaderboard status
- sample-size warnings

It maps those signals to conservative actions such as:

- `RequestMorePredictions`
- `RequestCalibrationReview`
- `RequestRiskReview`
- `DowngradeToDiagnostic`
- `KeepResearchCandidate`

Risk profiles remain offline research guidance only. They do **not** authorize live promotion, runtime inference, or broker activity.
