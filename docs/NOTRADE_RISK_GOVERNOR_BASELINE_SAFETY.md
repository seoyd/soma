# NoTrade and Risk Governor Baseline Safety

`NoTrade` is still a correct and expected outcome after Sprint 96.

- uncertain signal -> `NoTrade`
- missing feature -> `NoTrade`
- poor data -> `NoTrade`
- score alone cannot force action

Risk Governor still overrides BaselineSignal.

- `RiskDenied` overrides baseline output
- emergency stop overrides baseline output
- cooldown overrides baseline output
- baseline score cannot bypass veto or force a trade

This keeps BaselineSignal firmly inside the research-only and paper-only operating model.
