# Restart Sprint 50 Report

## Scope

Implemented the offline `cycle_risk_skeptic_shadow_v0` learned downside-risk lane. The implementation is intentionally ShadowOnly: all assessments abstain from committee, Chair, and trading use.

## Implemented controls

- Future maximum adverse log-excursion labels with train-only quantile thresholds.
- Deterministic downside/cycle feature schema, partition diagnostics, train-only feature normalization, and train-only representation normalization.
- Chronological two-pack replay, purge gaps, validation-only selection, and a sealed-once test result.
- R0 constant, R1 independent linear, and R2 frozen independent Mamba risk baselines with Brier-first evaluation and calibration, resolution, uncertainty, guarded AUC, false-negative/positive, and collapse diagnostics.
- Separate version and journal namespaces, explicit evidence-use classes, no network paths, and a machine-checkable independence proof.

## Preserved boundaries

The pre-existing Momentum prospective challenge remains untouched: status is `AwaitingFutureRows`; no request/retry, prospective row, label, event, metric, registry transition, vault update, or outcome evaluation was performed. The committee remains three members. No live execution or promotion capability was added.

## Verification record

Baseline default and Metal workspace suites passed before the implementation. The new deterministic independent-risk unit test passed after implementation. The offline local runner used accepted snapshot `snapshot-9867e361baa3a4b4` only and reported zero new network requests. Its aggregate verdict was `LinearBaselineStronger`: the older pack had R0/R1/R2 Brier values `0.41792855/0.38573864/0.39423698`, while the newer pack was positive at `0.35978308/0.25722048/0.22786850`. Both packs recorded zero high-confidence false negatives and no probability collapse. Complete default and Metal verification is recorded with the delivery commit.

## Outcome discipline

The historical verdict is never treated as a trading authorization. If R2 collapses, has a high-confidence false negative, or fails to beat a baseline, the result remains a documented negative/insufficient historical finding and all runtime assessments abstain.
