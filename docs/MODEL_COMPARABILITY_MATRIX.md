# Model Comparability Matrix

Sprint 65 adds a deterministic **model comparability matrix** for offline external model research review.

Each model/version is checked against the bounded sequence baseline for:

- dataset fingerprint
- feature schema hash
- label manifest hash
- split policy
- prediction schema version
- model-card availability
- evaluation metric availability
- coverage ratio
- calibration / risk / ablation / promotion availability

The matrix is conservative:

- mismatches remain explicit instead of silently downgraded
- partial comparability does not imply usefulness
- full comparability does not imply deployment readiness

The matrix is diagnostic only and remains local-only.
