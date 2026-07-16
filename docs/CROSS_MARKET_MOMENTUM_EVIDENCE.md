# Cross-market momentum evidence

Momentum behavior, candidates, thresholds, checkpoint gates, and temporal
support policy are shared across BTC, Korean equity, and US equity. Each series
uses an independent immutable snapshot and frozen evidence pack; markets are
never mixed during training.

The cross-market report always renders BTC, Korean equity, and US equity rows.
Unavailable equity rows remain visible with their independent Toss contract
status. A contract-blocked row is not substituted with synthetic data and does
not contribute to positive evidence, accepted versions, or an architecture
conclusion.

Only two independently accepted market classes can support a recurrent
frozen-representation conclusion. Out-of-support metrics remain research-only.
All rows remain ShadowOnly with voting, execution, and promotion disabled.

Contract-intake and capability qualification are separate from acquisition.
A blocked or unavailable manifest produces a visible market row without
creating synthetic equity evidence.
