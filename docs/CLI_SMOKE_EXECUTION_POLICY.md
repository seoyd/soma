# CLI Smoke Execution Policy

Sprint 84 separates CLI smoke into representative, exhaustive, and safety tiers.

- representative smoke is the fast sprint loop
- exhaustive smoke documents the full Sprint 82/83 command family coverage for full/release flows
- safety smoke keeps help checks, remote-path rejection checks, and forbidden-command checks
- representative smoke never replaces safety smoke
- smoke tiering does not authorize runtime, training, broker, or live behavior

