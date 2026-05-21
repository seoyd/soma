# CounterfactualBackfill Entry Gate

Sprint 96 only prepares the `CounterfactualBackfill` entry gate.

Entry becomes ready only after BaselineSignal is reduced or explicitly held with preserved safety coverage. Even when entry is ready:

- CounterfactualBackfill reduction has **not** started
- no-run/full workspace results remain separate reports
- the queue advancement is only about next-family eligibility

Sprint 96 therefore ends with `CounterfactualBackfillEntryReady` and `CounterfactualBackfillPrecheckReady`, not with CounterfactualBackfill closure.
