# Cargo JSON Observation V17

Cargo JSON parsing is diagnostic-only. Progress, parse counts, and artifact ordering help narrow root cause but do not upgrade acceptance.

If the cargo JSON command is not actually run, artifact ordering stays empty/deferred instead of using placeholder artifact names.
