# Report Bundle

`ExperimentReportBundle` is the deterministic output of a Sprint 10 experiment run.

## Included components

- experiment manifest
- data quality report
- optional dataset export summary
- optional baseline walk-forward report
- optional external walk-forward report
- optional comparison report
- optional calibration report
- optional threshold search report
- optional prediction validation result
- errors / warnings / reason codes

## Output files

Typical bundle layout:

- `manifest.txt`
- `data_quality_report.txt`
- `dataset.csv`
- `baseline_report.txt`
- `predictions.csv`
- `model_card.md`
- `external_report.txt`
- `comparison_report.txt`
- `experiment_summary.txt`

## Determinism

The bundle uses fixed ordering and deterministic text summaries. Wall-clock timestamps are not injected unless explicitly supplied in config metadata.

## Leakage warning

Rust can validate dataset schema, row alignment, feature schema, and fold boundaries. It cannot fully prove that external Python training code was leakage-free unless that external process follows the documented contract.
