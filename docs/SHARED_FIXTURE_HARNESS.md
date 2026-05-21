# Shared Fixture Harness

Sprint 84 adds a shared test harness for fixture loading and deterministic output checks.

- `load_json_fixture<T>`, `load_toml_fixture<T>`, and `load_csv_fixture` centralize fixture reads
- `temp_output_dir_for_test` keeps deterministic per-test output directories
- assertion helpers preserve secret scanning, no order/account fields, no runtime fields, source-boundary checks, and no-lookahead checks
- the harness reduces repeated setup only; it does not change report semantics

