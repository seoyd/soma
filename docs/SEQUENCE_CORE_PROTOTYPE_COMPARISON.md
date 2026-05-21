# Sequence Core Prototype Comparison

Sprint 80 adds an **offline external CSV comparison** layer for `Mamba3Fin` and `GatedDeltaNet`.

- inputs stay on the shared dataset / feature-schema / label-manifest / split-policy contract
- prototype artifacts are limited to prediction CSVs and model cards
- outputs are diagnostic-only and do not imply runtime readiness, deployability, or profitability
- runtime inference, training, live inference, and live trading remain deferred/forbidden

