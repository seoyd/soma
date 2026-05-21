# KIS candle sufficiency and outcome closure

- Candle sufficiency is no-lookahead constrained.
- Barrier-profile horizon controls required future bars.
- Outcome closure can derive candle sufficiency from canonical CSVs when a prebuilt sufficiency JSON is absent.
- CLI:
  - `soma-experiment kis-candle-sufficiency --config examples/soma_kis_candle_sufficiency.toml`
  - `soma-experiment kis-outcome-link-close --config examples/soma_kis_outcome_link_close.toml`
