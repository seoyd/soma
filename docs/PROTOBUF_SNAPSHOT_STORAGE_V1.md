# Protobuf Snapshot Storage V1

New local historical snapshots are stored as `.pb` files. The implementation
uses manually derived `prost::Message` types only; there is no `protoc`, build
script, or generated schema dependency.

The outer envelope contains a fixed magic value, version `1`, schema name,
semantic digest, snapshot ID, payload byte length, payload digest, and payload.
Decode rejects a wrong magic/version/schema, length mismatch, payload digest
mismatch, malformed payload, semantic-digest mismatch, or identifier mismatch
before the snapshot reaches inventory or campaign code.

The payload contains all fields required to reconstruct `DataSnapshot`.
OHLCV numeric values are stored as `fixed64` IEEE-754 bit patterns. Ordered
rows stay repeated and chronological; non-semantic string and reason-code
collections are sorted before encoding. The Protobuf bytes and JSON bytes are
not identity material.

Writes use a temporary file, `sync_all`, temporary decode-and-verify, atomic
rename, and final reopen/decode/verify. Legacy `.json` is read only for an
explicit migration: validate its shape, recompute semantic digest and ID, write
a Protobuf sidecar, and leave the legacy file unchanged. New real snapshots are
never written as JSON.
