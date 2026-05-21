# Sprint 07 Report

## Implemented items

- added model artifact metadata
- added prediction row/frame types and validation
- added deterministic prediction CSV import/export
- added `ExternalPredictionSignalModel`
- added calibration report
- added validation-only threshold search
- added baseline vs external comparison report
- integrated external prediction mode into walk-forward evaluation

## Tests

Added Sprint 07 coverage for:

- prediction validation
- CSV round-trip and parse failure handling
- strict schema mismatch blocking
- conservative missing/invalid prediction fallback
- calibration determinism and empty reports
- threshold search determinism and research-only mode
- conservative model comparison
- walk-forward external mode determinism

## Risk review

- no runtime LLM path added
- no real broker path added
- no model training added
- schema mismatch blocks strict external evaluation
- missing/invalid predictions collapse to `NoTrade`
- Risk Governor remains above imported predictions
- leakage warning documented for external training provenance

## Deferred items

- real model training
- JSONL import/export
- ONNX / Python bridge
- live prediction feeds
- real brokers / live execution
- model-serving infrastructure

## Next sprint recommendation

Use the Sprint 07 bridge to add an **offline training handoff** sprint: keep Rust responsible for schema/export/evaluation, and let external tooling train models against the locked dataset contract without changing the live execution posture.
