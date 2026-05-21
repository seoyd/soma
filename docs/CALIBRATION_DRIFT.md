# Calibration Drift

Sprint 64 adds a deterministic offline **calibration drift** report between comparable model versions.

The report tracks:

- Brier score deltas
- ECE deltas
- confidence / hit-rate proxy shifts

Severity stays conservative:

- `Stable` means no material regression was detected in the compared offline artifacts
- `MildDrift` means calibration moved enough to require caution
- `SevereDrift` blocks research-candidate interpretation
- `InsufficientHistory` is a warning, not evidence of safety

Calibration drift is diagnostic only. It does **not** prove usefulness, deployability, or profitability.

