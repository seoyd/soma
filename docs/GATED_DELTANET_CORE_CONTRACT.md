# Gated DeltaNet Core Contract

Sprint 79 adds **Gated DeltaNet** because it existed in the original Soma lineage, but only as a contract-first candidate.

The contract fixes:

- input tensor and sequence window assumptions,
- state matrix shape and key/value dimensions,
- q/k/v/output projection requirements,
- decay/update/beta/delta gate requirements,
- prediction heads matching the shared sequence-core interface.

What it does **not** do:

- no recurrent update runtime
- no delta-rule kernel implementation
- no Rust-native inference
- no training
- no live inference or broker execution

