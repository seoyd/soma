# Sprint 10 Report

## Implemented items

- added offline experiment config, manifest, stage tracking, report bundle, and runner
- added mode-specific flows for validate, dataset export, baseline, external prediction, and train+compare
- added minimal local-only research CLI and example TOML configs
- wired Sprint 09 local CSV adapter into the experiment harness

## Tests

- experiment config tests
- baseline/dataset/external runner tests
- failure-mode tests
- determinism tests

## Risk review

- no live API path
- no broker path
- no runtime LLM path
- Python remains optional and research-only
- invalid data and invalid predictions fail or skip explicitly
- Risk Governor veto remains in the evaluation path

## Deferred items

- richer config file examples
- more detailed JSON report emission
- wider CLI surface beyond minimal research commands
- richer stage-level artifact metadata

## Next sprint recommendation

Expand the experiment harness with stronger provenance capture for external research runs so each bundle records exact dataset hash, prediction hash, model-card hash, and Python environment metadata without weakening the current offline-only safety boundary.
