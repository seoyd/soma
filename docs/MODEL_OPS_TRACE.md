# Model Ops Trace

Sprint 68 adds a **static trace drill-down** layer on top of the Sprint 67 model ops rollup.

The trace layer:

- links each model-version card back to local artifacts
- builds a deterministic trace graph per model version
- shows decision, regression, QA, risk, and action rationale without adding execution
- emits TXT/JSON plus optional local HTML fragments

The trace remains **read-only, local-only, paper-only, and research-only**. It does not add training, live inference, broker/order/account controls, or runtime Mamba behavior.
