# Equivalent Coverage Proof

Sprint 109 allows target retirement only after the migrated assertions have an equivalent destination target.

For the second patch, the retired source target is blocked unless `tests/shared_fixture_harness_application_v1.rs` exists as the destination and the moved output-dir helper assertions remain represented there. Any coverage gap keeps retirement blocked.
