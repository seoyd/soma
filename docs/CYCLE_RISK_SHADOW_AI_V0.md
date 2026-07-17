# Cycle/Risk Shadow AI V0

`cycle_risk_skeptic_shadow_v0` is an offline learned downside-risk shadow. It is not a fourth committee member, does not submit to the Chair, and cannot create an order. The active committee remains exactly three members.

## Evidence and chronology

The runner accepts one digest-verified, read-only BTC historical snapshot and deterministically freezes two chronological packs (`older`, then `newer`). No provider, transport, credential, or network-consent path is called. Each pack uses chronological train/validation/test partitions separated by a purge gap at least as large as the future-label horizon; model selection uses validation only and test is evaluated once after selection.

The label is the maximum future adverse log excursion from the anchor close to the lowest low over `H=horizon_rows`. The positive threshold is the configured quantile fitted only from training anchors. Validation and test labels never influence that threshold.

## Independent risk schema

The deterministic risk feature schema contains downside semivariance, negative-return frequency and tail mean, consecutive losses, drawdown depth/duration/recovery, short and long volatility, volatility ratio and volatility-of-volatility, range ratio, lower-tail quantile, and volume stress. Feature diagnostics are emitted separately for train, validation, and test. Both feature and encoded-representation normalizers fit on train only.

Three risk models are measured per pack:

- R0: constant probability equal to train prevalence.
- R1: independently initialized linear logistic risk head.
- R2: independently seeded frozen Mamba encoder, train-only representation normalizer, and independent logistic risk head. No gradient enters the encoder.

Primary selection metric is Brier score. Every partition records calibration reliability, resolution, uncertainty, guarded rank AUC, prevalence, probability mean/stddev, coverage, abstention count, high-confidence false negatives/positives, and collapse diagnostics. A positive historical verdict requires R2 to beat both R0 and R1 without collapse or excess high-confidence false negatives; otherwise it remains ShadowOnly.

## Boundaries and records

Risk versions use `cycle-risk-shadow-v0/...` and the separate journal namespace `cycle-risk-shadow-v0/journal`. The evidence ledger records CycleRisk feature, label, training, validation, test, and diagnostics use classes with monotonic cutoffs. It contains no Momentum prospective artifact, path, future row, prediction, label, performance result, or trading action.

Run locally without a network flag:

```text
soma-zero --historical-snapshot-campaign-config <local-config> --btc-cycle-risk-shadow-report
```
