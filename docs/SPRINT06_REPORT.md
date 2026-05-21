# Sprint 06 Report

## Implemented items

- added deterministic walk-forward split generation
- added leakage guard and unsafe-boundary reporting
- added dataset export types and deterministic CSV export
- added feature schema lock and validation
- added trade / decision / no-trade / risk / calibration / regime / persona / Chair metrics
- added `WalkForwardEvaluator` reusing the existing replay path

## Tests

Added Sprint 06 coverage for:

- deterministic fold generation
- embargo and insufficient-data behavior
- feature no-lookahead and label future dependency
- overlap and unsafe-boundary leakage checks
- deterministic dataset export
- feature schema validation
- metrics correctness on synthetic outcomes
- deterministic walk-forward evaluator output
- panic regime surfacing inside fold reports
- risk veto preservation on low-quality data

## Risk review

- no runtime LLM path added
- no real broker path added
- no random split added
- feature and label paths remain separated
- fold-boundary unsafe rows are marked explicitly
- Risk Governor veto remains absolute
- costs remain part of reported trade outcomes

## Deferred items

- LightGBM / XGBoost training
- JSONL export
- ONNX / Python bridge
- real data download
- live trading
- heavy attribution methods

## Next sprint recommendation

Use the Sprint 06 dataset/schema/reporting foundation to add a narrow offline model-training interface in a future sprint while keeping replay deterministic and leaving live execution out of scope.
