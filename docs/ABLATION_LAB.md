# Ablation Lab

Sprint 13 adds a deterministic, local-only ablation layer on top of the existing batch experiment runner.

## What it does

- loads a local batch matrix
- applies one or more conservative ablation variants
- reuses the existing batch runner for execution
- compares each variant against a stable baseline matrix
- emits:
  - `ablation_report.json`
  - `ablation_summary.txt`
  - `ablation_summary.md`
  - `sensitivity_summary.txt`

## Safety rules

- local paths only
- paper-only research surface
- no live execution path
- disabling the `data_quality` feature group requires `research_only = true`
- unknown override targets are ignored conservatively and marked in the report

## Supported dimensions

| Dimension | Example targets |
| --- | --- |
| `FeatureGroup` | `volume`, `spread_liquidity`, `time_context`, `data_quality` |
| `TripleBarrier` | `take_profit_pct`, `stop_loss_pct`, `horizon_bars` |
| `CostModel` | `fee_bps`, `slippage_bps`, `spread_bps`, `min_cost_bps` |
| `RiskGovernor` | `min_confidence`, `min_expected_edge`, `min_data_quality`, `max_spread_bps` |
| `Chair` | `strong_threshold`, `weak_threshold`, `allow_forced_contrarian`, `cluster_penalty_enabled` |
| `Regime` | `min_data_quality`, `high_vol_threshold`, `panic_return_threshold` |
| `NoTradeScoring` | `avoided_loss_weight`, `missed_gain_weight` |

## Example

```bash
cargo run --bin soma_experiment -- ablation --config examples/soma_ablation_feature_lab.toml
```

Outputs are written under `target/soma_ablations/<study_id>/`.
