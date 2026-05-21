# Sprint 67 Report

## Implemented items

- model ops rollup config, key, runner, and bundle
- model version summary cards
- regression cause explanations
- operator QA / decision log / risk / action-priority rollups
- static summary and Control Tower model ops rollup panel

## Tests

- focused Sprint 67 integration coverage for config, rollup key, cards, regression explanations, QA rollup, decision rollup, risk rollup, action priority, summary, panel, CLI safety, and determinism

## Rollup status

- example rollup stays conservative and offline-only
- one card is emitted per model version
- repeated raw review rows are collapsed

## Regression explanation status

- example regression explanations isolate `ext-model-b:1.0.0`
- coverage/calibration/risk/comparability/artifact/leaderboard causes stay explicit

## Operator QA rollup status

- example QA rollup still needs more predictions for the weak-coverage model
- blocked actions remain preserved when present

## Control Tower panel status

- static summary cards render without train/live/order controls
- Mamba runtime remains deferred

## Risk review

- no live trading
- no broker/order/account APIs
- no runtime LLM decision path
- no runtime Mamba
- no model training

## Next sprint recommendation

- keep improving offline operator readability and conservative evidence interpretation
