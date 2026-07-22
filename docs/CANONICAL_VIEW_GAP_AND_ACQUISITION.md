# Canonical View Gap and Learning Evidence Acquisition V1

## Purpose

This protocol audits the canonical learning view of each active agent from its
persisted intent identity, current data policy, immutable local evidence, real
trainer capability, and verified provider contracts. Status and dry-run are
offline. Execute-local persists only verified Protobuf artifacts and remains
offline unless both explicit network-consent flags are present.

## Gap resolution

For each active agent the audit reports required and optional dataset kinds,
resolved and missing evidence, market and symbol scope, cadence, lookback,
information cutoff, maximum staleness, usable and rejected artifact identities,
authorized providers, trainer availability, and a semantic gap digest.

Current Protobuf sessions that contain full intent metadata are reconstructed
only when their original intent digest verifies. Legacy sessions retain their
persisted intent identity and cutoff; their missing metadata is projected from
available source snapshot identities and the current policy. A legacy
projection is audit-only and cannot bypass normal intent validation for
training.

Required evidence is evaluated before optional evidence. A real cadence,
market, symbol, cutoff, ambiguity, or integrity failure remains explicit. A
matching provider whose single-response capacity is too small is classified
separately. Segmented acquisition is required only when the complete bounded
partition is derivable before a response is observed; unsupported or
over-budget partitions fail closed. Provider availability never changes an
agent policy to match available data.

## Provider and request boundary

A provider contract is eligible only when its provider identity, dataset
semantics, market, symbols, cadence, bounded historical range, response limit,
finality, read-only behavior, and credential-free status all match. Upbit is
authorized only for BTC daily OHLCV and cannot satisfy index, volatility,
breadth, macro, fundamental, valuation, or adjusted-price contracts.

Selection considers only trainer-capable agents missing required evidence. It
orders exact requests by required status, semantic support, credential-free
access, number of blocked agents served, bounded response size, and stable
request identity. Equivalent requests are deduplicated before any transport is
constructed.

The current audit produced no eligible legacy single request. Momentum's exact
two-segment registration preserves the full persisted lookback. Cycle/Risk has
no exact index or volatility provider contract. Value/Quality has no trainer.

## Composite execution contract

When a matching daily provider has insufficient single-response capacity, the
complete expected timestamp sequence is derived from the persisted intent
before transport. The segment count is the ceiling of required rows divided by
the provider limit and is capped at two for one epoch. The current registered
plan contains a newer provider-sized segment and the exact older remainder;
both timestamp sets, exclusive request boundaries, execution order, budgets,
and semantic digests are fixed before the first response.

Each segment permits one attempt, zero retries, and one concurrent transport.
The second boundary never depends on the first response. A first-segment
failure suppresses the second request. A second-segment failure retains only
ignored forensic segment evidence and creates no merged snapshot.

Each response must match the exact registered timestamp set and count, market,
symbol, cadence, provider, schema, finality, bounded size, sanitization, finite
OHLCV values, and nonnegative volume and trade value. Missing, duplicate,
extra, excluded, or prospective timestamps reject. Newest-first provider data
is identity-checked and normalized chronologically.

A canonical snapshot exists only after every segment succeeds. Merge verifies
complete row count, unique and disjoint segment membership, strict daily
chronology, identical source semantics, and protected exclusions. Its semantic
identity derives from the complete normalized dataset and is independent of
segment order or Protobuf bytes.

The composite registration, segment receipts, segment capsules, epoch receipt,
merged provenance, and canonical snapshot use manual Protobuf. Each segment raw
blob is retained separately. All writes use temporary create-new storage,
flush, `sync_all`, temporary reopen and verification, atomic rename, and final
reopen and verification.

## Legacy one-request execution contract

When an eligible request exists, its ignored registration fixes one request,
one concurrent transport, zero retries, a bounded response, read-only and
credential-free access, and a prospective-storage prohibition. Execution also
requires a verified registration reopen, a current gap, no prior receipt, no
equivalent canonical snapshot, and explicit network consent. Every transport
attempt consumes the budget; failures are terminal and no fallback provider is
used.

Validated responses must match provider and dataset semantics, exact timestamp
range, chronology, cutoff, protected exclusions, schema, finality, finite
values, size, and sanitization. Successful evidence is stored as an ignored raw
blob, manual-Protobuf provenance, a canonical manual-Protobuf snapshot, and a
manual-Protobuf receipt. Semantic identity is independent of encoded bytes.

All writes use a temporary file, flush, `sync_all`, temporary reopen and
verification, atomic rename, and final reopen and verification. Identical
snapshots and persisted artifacts are duplicate-rejected.

## Current per-agent result

| Agent | Gap status | Required evidence result | Acquisition result |
| --- | --- | --- | --- |
| Momentum | `MissingOptionalEvidenceOnly` | Daily OHLCV is resolved; only optional evidence remains missing. | `EvidenceAcquired`; both registered segments and the merged canonical snapshot verified. |
| Cycle/Risk | `ProviderContractUnverified` | Index and volatility evidence remain missing. | Upbit OHLCV is not relabeled; request count 0. |
| Value/Quality | `TrainerUnavailable` | Adjusted prices and fundamentals remain missing. | Excluded from request priority. |

The offline rerun kept the three agents independent. Momentum's required view
completed, but V1 family generation remained `InsufficientEvidence` because a
complete persisted view under the normal intent-validation boundary was not
available; no evaluation registration was created. Cycle/Risk remained
`ProviderContractUnverified`, and Value/Quality remained `TrainerUnavailable`.

## Safety result

The terminal epoch contains two successful segment attempts, zero retries, and
maximum concurrency one. A repeated confirmed execution returned
`AlreadyTerminal` with zero new requests and zero new transports. Credential,
prospective-artifact, prospective-label, and future-evaluation reads were zero.
Active-model changes, Chair decisions, votes, rewards, penalties, voice changes,
promotions, and executions were zero. Active committee count remained three.
