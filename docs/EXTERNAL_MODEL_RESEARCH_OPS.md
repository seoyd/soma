# External Model Research Ops

Sprint 65 adds a **local-only external model research ops** layer on top of the Sprint 64 registry, drift, and leaderboard outputs.

The runner builds:

- lifecycle records
- review queue items
- owner-safe review impact
- watchlist state
- comparability matrix
- artifact completeness scores
- evidence risk profiles
- leaderboard changelog
- Control Tower model ops panel text

The workflow is conservative by design:

- only local paths are accepted
- owner actions can only make handling more conservative
- lifecycle transitions forbid `Live`, `RuntimeIntegrated`, and `BrokerExecutable`
- review items always carry forbidden live/runtime/training actions
- Mamba3Fin remains artifact-only and runtime-deferred

This layer is still **research-only**. It does **not** add training, live inference, broker execution, order/account controls, or runtime Mamba.
