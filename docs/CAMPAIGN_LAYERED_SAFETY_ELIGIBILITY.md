# Campaign Layered Safety Eligibility

## Purpose

The offline momentum campaign has one eligibility to evaluate immutable historical evidence and three deliberately separate authorities: promotion, voting, and execution. A successful lower layer never grants an upper-layer authority.

## Deterministic safety trace

Every campaign result carries an ordered, sanitized safety trace. Each entry has a stable gate name, outcome, optional reason code, and count/boolean-only facts. The trace records the first rejecting gate and reason code, while gates after that rejection are explicitly marked as not evaluated. It does not contain paths, credentials, snapshot identifiers, or market values.

The evidence gates are immutable/sanitized provenance, real historical classification, canonical semantic digest, chronology, finite OHLCV, enough history, and purge-separated chronological windows. Runtime gates require the existing CPU full-inference path and capture the frozen encoder digest before evaluation; the digest is checked unchanged after evaluation.

## Offline Shadow learning eligibility

Offline Shadow learning is permitted only after all evidence and runtime gates pass. The campaign receives immutable snapshots and a frozen encoder as inputs; it receives no provider, broker, transport, account, order, committee, or execution capability. It creates only `ShadowOnly` model versions and offline assessments.

## Higher authority boundaries

Promotion is blocked for the experimental internal-reference model. Voting is blocked because every campaign version is `ShadowOnly`. Execution is blocked by the official-oracle execution boundary. These are recorded as blocked upper-layer gates rather than campaign-evidence rejection, so they cannot prevent safe offline evaluation and cannot be misread as permission.

## Integrity rule

Snapshot identity is verified with `historical_replay_dataset_digest_v0`, the same canonical semantic digest used by acquisition, inventory, frozen-pack verification, and Protobuf persistence. Serializing a dataset through JSON is not an identity verifier and is not used by the campaign.
