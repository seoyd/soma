# Mamba3Fin Core Contract

Sprint 78 defines **Mamba3Fin as a contract-only surface**.

The contract fixes:

- input tensor shape expectations,
- required feature-schema and label-manifest hashes,
- supported horizons,
- target heads for take-profit, stop-loss, time-expiry, return, drawdown, and confidence,
- required risk integration and Control Tower visibility.

It explicitly keeps runtime deferred:

- recurrence/streaming specs are contract-only,
- inference is `NotImplemented`,
- training is `NotImplemented`,
- live inference, broker execution, runtime training, runtime LLM decisions, and risk bypass are forbidden.

