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

## Learning-data envelope

Agent learning views use the same storage principles in a separate
`state/learning_data` namespace. Their manually derived envelope contains a
fixed magic value, major version, schema name, artifact kind, semantic digest,
payload length, payload digest, payload, and the referenced source artifact
digests. The payload has stable field numbers for view identity, agent identity,
source references, visible dataset kinds, cutoff, policy digests, private
namespace, training ledger, counts, missing evidence, decision gate, and view
digest. Removed field numbers must be reserved and never reused.

The decoder rejects wrong magic, unsupported major version, wrong schema or
kind, malformed payload, length or payload-digest mismatch, semantic-digest
mismatch, invalid source identity, invalid active-agent identity, unsorted
semantic collections, and inconsistent abstention state. The semantic digest is
recalculated from decoded meaning; it does not include Protobuf field ordering,
unknown fields, local paths, timestamps from the filesystem, compression, JSON
bytes, or terminal formatting.

Learning-view writes explicitly `flush`, `sync_all`, reopen and decode the
temporary file, atomically rename it, then reopen and decode the final file.
Explicit legacy learning-view migration writes a verified `.pb` sidecar beside
the JSON source and compares the original bytes after migration. Runtime raw
blobs remain separate from Protobuf provenance and normalized/view references;
large arbitrary internet content is not embedded as semantic message text.
