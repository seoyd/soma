# Barrier Profile Registry

Sprint 48 adds a local-only registry for bounded triple-barrier profiles.

## Profile types

- **Primary preregistered**: the default official profile. It can count toward official sufficiency only when it was registered before outcome evaluation.
- **Secondary preregistered**: an alternate preregistered profile. It can also count for official sufficiency when it was registered before outcome evaluation.
- **Diagnostic**: useful for plumbing and investigation only. Diagnostic profiles stay diagnostic-only and can never satisfy official sufficiency.
- **Exploratory**: useful for research exploration only. Exploratory profiles stay exploratory-only and can never satisfy official sufficiency.

## Anti-p-hacking policy

Official sufficiency only accepts preregistered primary or secondary profiles with `registered_before_outcome_eval = true`.

That means:

- no post-hoc barrier tuning from observed outcomes
- no promoting diagnostic or exploratory profiles into official sufficiency
- no remote profile sources
- deterministic ordering and rendering for repeatable audits

## CLI

```bash
cargo run --bin soma_experiment -- barrier-profiles --config examples/soma_barrier_profiles_primary.toml
```

This remains research-only, local-only, and never implies live trading readiness.
