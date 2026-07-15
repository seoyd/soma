# Momentum temporal diagnostic report

The offline local-snapshot campaign accepts `--momentum-temporal-diagnostics` with `--output-format text` or `--output-format json`. It verifies the local snapshot and frozen in-memory evidence pack before deterministic campaign replay; it does not accept a network path.

The sanitized report contains evidence row and window counts, selected candidate/checkpoint references, aggregate distribution metrics, validation and sealed-test support decisions, earliest stage, root cause, warm/cold summary, aggregate counters, permissions, reason codes, and a semantic digest. It deliberately excludes local paths, raw candles, local configuration contents, credentials, provider responses, and model-weight vectors.

Support is decided from label-free values before outcome labels are read. An `InSupport` result remains Shadow-only. Every other support result produces `shadow_abstain`; later test metrics are explicitly counterfactual research evidence and cannot vote, execute, promote, or become positive predictive evidence. The report digest excludes filesystem paths, timestamps, formatting, and JSON whitespace.

The cross-market wrapper reuses this report only for independently accepted
snapshots. Contract-blocked markets are visible as blocked rows and cannot be
silently represented as temporal evidence.
