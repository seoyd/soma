# Snapshot Canonical Identity

`DataSnapshot` identity is the digest of its normalized historical dataset, not
the bytes of a JSON or Protobuf file. The identifier is
`snapshot-<semantic-digest>` and the digest never includes that identifier or a
storage container field.

## Canonical byte contract V1

The byte stream begins with the ASCII domain separator
`SOMA-HISTORICAL-DATASET-SEMANTIC-V1`. Every following field is encoded as one
byte field tag, a four-byte big-endian byte length, and field bytes. Strings
are UTF-8. Counts are four-byte big-endian. Timestamps are eight-byte
big-endian. Reason-code tags are two-byte big-endian, sorted and deduplicated.
OHLCV rows remain ascending by timestamp and are never reordered. Every finite
number is its IEEE-754 `f64` bits in big-endian order; `-0.0` is normalized to
the `+0.0` bit pattern because validation treats the two values identically.
An optional trade value has an explicit one-byte presence marker.

The resulting byte stream is FNV-1a 64-bit hashed and emitted as lowercase,
16-character hexadecimal. This contract uses no serialization bytes, debug
formatting, unordered map iteration, or native-endian integer representation.

## Validation boundary

Before a snapshot is accepted, row counts, one-symbol consistency, timestamps,
finite OHLCV values, the canonical digest, and the digest-derived identifier
are all checked. A storage decode repeats these checks.
