# Cycle/Risk opinion provenance

The V1 provenance layer is an offline audit layer around immutable V0 reports.
It uses explicit byte encoding: big-endian integers, IEEE-754 float bits,
length-prefixed strings, enum tags, optional-value tags, and ordered vectors.
It never uses `Debug` output as V1 identity input.

Each Cycle/Risk result is resolved from its frozen-pack digest against the
runner's deterministic half-range plan. Display names and vector positions are
readability fields only and are not used as identity evidence. Result and
checkpoint identities include the resolved scope, configuration, threshold,
metrics, seal state, verdict, and accepted-model state.

The legacy Risk opinion adapter is replayed without model execution. A source
is verified only when exactly one resolved immutable result reproduces both the
legacy opinion digest and seal digest. Zero or multiple candidates are reported
as non-verifying outcomes. This is intentionally fail-closed.

Anchor evidence is audit-only. It materializes the actual Risk feature,
sequence, label-horizon, split, and purge rules without changing training,
thresholds, checkpoints, or model outputs. No raw OHLCV, model parameters,
probabilities, labels, local paths, network access, or trade actions are
published by the CLI summary.
