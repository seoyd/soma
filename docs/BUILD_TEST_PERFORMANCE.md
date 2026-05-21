# Build And Test Performance

Sprint 76 records or reuses local build/test baseline data without faking timings.

- baseline report covers fmt/check/full test/focused sprint tests/representative CLI smoke
- slow test inventory makes long-running paths explicit
- heavy fixture inventory highlights duplicate or repeated local fixture loads
- CLI smoke tiering keeps safety help coverage while reducing repetitive exhaustive runs

Optional acceleration ideas stay local-only: `cargo-nextest` and `sccache` are plans, not hard requirements.
