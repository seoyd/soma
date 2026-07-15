# Restart Sprint35 Report

## Scope

This change establishes serialization-independent snapshot identity, Protobuf
V1 local storage, explicit legacy migration, and an offline Toss capability
boundary. It does not add providers, market-data classes, credentials, account
access, order execution, streaming, compression, or model/trading behavior.

## Verification record

Default verification passed 601 tests and Metal verification passed 602 tests.
The Protobuf codec tests cover round-trip, corruption rejection, legacy sidecar
migration, and the `-0.0` policy. A single manually gated Upbit retry produced
a verified Protobuf snapshot; the local inventory accepted one series and the
evidence pack froze successfully. The momentum campaign then rejected the
series for safety, so no learned version or trading state was created.

On this local machine, using sanitized synthetic rows with two warm-up and
eight measured encode iterations, median JSON/Protobuf measurements were:

| Rows | JSON bytes | Protobuf bytes | JSON median | Protobuf median |
| ---: | ---: | ---: | ---: | ---: |
| 16 | 3,133 | 1,511 | 66,333 ns | 29,625 ns |
| 128 | 18,294 | 9,354 | 421,667 ns | 190,708 ns |
| 1,024 | 140,807 | 72,077 | 3,174,458 ns | 1,432,292 ns |

These figures are local, warm-cache observations rather than portability or
throughput guarantees. No compression was used.
