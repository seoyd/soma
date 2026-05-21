# Baseline Snapshot Coverage

Sprint 69 adds a static baseline/current snapshot coverage layer on top of the Sprint 68 model ops trace bundle.

## What it does

- indexes local baseline and current snapshots per `model_id:model_version`
- builds a comparison target registry
- explains `MissingComparisonTarget` instead of leaving it implicit
- allows explicit current-only diagnostic handling when baseline comparison is unavailable

## Safety boundaries

- local files only
- static/read-only output only
- no live trading, broker, order, or account path
- no runtime inference or model training
- no secret values in rendered output

