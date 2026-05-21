# Owner Review Discipline v2

Sprint 61 keeps owner review manual and paper-only, but makes the queue stricter:

- reasons are required for hold / dismiss / paper-confirm actions
- stale reviews are surfaced explicitly
- research-only and diagnostic-only confirms remain policy-gated

Run:

```bash
cargo run --quiet --bin soma_experiment -- owner-review-discipline-v2 --config examples/soma_owner_review_discipline_v2.toml
```

The output is a cleanup report, not an execution path.

