# External Model Artifact Registry

Sprint 64 adds a **local-only external artifact registry** for bounded external model research artifacts.

The registry accumulates:

- model cards
- prediction CSV artifacts
- import reports
- evaluation reports
- ablation reports
- promotion-gate reports
- Mamba3Fin-lite contract artifacts

The registry is conservative by design:

- only local paths are accepted
- remote paths are rejected
- incompatible dataset / feature-schema / label-manifest contracts are marked non-comparable
- missing model cards or missing evaluation reports block leaderboard inclusion
- registry readiness never implies deployment readiness

The registry is still **research-only**. It does **not** add training, live inference, broker execution, order/account actions, or runtime Mamba.

