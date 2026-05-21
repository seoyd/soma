# Fixture Setup Dedup

Sprint 77 focuses on repeated local fixture/setup cost, not semantic changes.

- repeated JSON/TOML/CSV loads are identified
- repeated output-dir setup is identified
- dedup/cache plans only target setup/loading reuse
- fixture content and expected outputs must remain unchanged

Any follow-up refactor stays conservative and requires verification that test semantics did not change.
