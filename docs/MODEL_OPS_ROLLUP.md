# Model Ops Rollup

Sprint 67 adds an offline-only **model ops rollup** layer on top of the Sprint 66 closure/history/QA stack.

The rollup:

- groups outputs by `model_id` / `model_version`
- deduplicates repeated raw review rows
- builds one conservative summary card per model version
- keeps regression explanations and next actions static/read-only

The rollup does **not** imply deployment, live promotion, or profitability. It remains local-only, paper-only, and deterministic.
