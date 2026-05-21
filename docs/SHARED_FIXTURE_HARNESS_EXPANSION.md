# Shared Fixture Harness Expansion

The shared fixture harness expansion plan is limited to deterministic helpers:

- shared JSON/TOML loader reuse
- shared output-dir setup
- shared render helpers
- deterministic fixture normalization only

The plan does not introduce runtime behavior. It only reduces duplicated test-support setup while preserving deterministic fixture handling.
