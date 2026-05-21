# Chair and risk calibration

Sprint 34 adds **research-only suggestions**, not automatic tuning.

## Chair calibration

The chair report can suggest:

- stronger contrarian protection
- stronger cluster penalty
- earlier groupthink warning
- more no-trade conservatism
- reduced over-filtering

All suggestions have `apply_automatically=false`.

## Risk calibration

The risk report can suggest:

- keep hard vetoes as-is
- tighten unstable areas
- investigate overblocking in research-only sandbox

`LoosenResearchOnly` never means changing production or bypassing Risk Governor. Hard veto rules remain immutable.

