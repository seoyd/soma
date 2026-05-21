# Model Predictions Stale Closure

`ModelPredictionsStale` closes only when refreshed prediction coverage validates against the current real-evidence sequence context. The warning is never hidden; when closure is incomplete the report explains exactly why it remains.

Even after stale closure, deferred warnings such as `DirectWatchMonitoringOnly`, `RuntimeMambaDeferred`, `LiveTradingForbidden`, and `BrokerForbidden` remain visible by design.
