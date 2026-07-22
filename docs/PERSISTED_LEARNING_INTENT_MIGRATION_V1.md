# Persisted Learning Intent Migration V1

## Purpose and boundary

This migration repairs one legacy Momentum learning intent without rewriting
any prior session, intent, view, evidence, opening, evaluation, reward, or
active-agent artifact. It is an offline, additive operation. Network permission
is rejected, and status and dry-run modes create no artifacts.

The production audit found `LegacySessionNotSelfDescribing`; the first failing
normal-validation invariant was `intent_version`. Required market evidence was
already complete, so evidence acquisition was not the blocker.

## Authoritative reconstruction

Only the following declared sources are accepted:

- `LegacySession`
- `LegacyIntentProjection`
- `VerifiedAgentPolicy`
- `CanonicalGapReport`
- `CompositeAcquisitionRegistration`
- `CanonicalSnapshot`
- `ExistingPrivateLearningState`

The 16 bound field groups are:

| Field group | Required authorities |
| --- | --- |
| agent ID | legacy session, legacy projection, gap report, composite registration |
| agent kind | legacy session, legacy projection, verified policy |
| market scope | legacy projection, verified policy, gap report, composite registration, snapshot |
| symbols | legacy projection, gap report, composite registration, snapshot |
| required datasets | legacy projection, verified policy, gap report, composite registration, snapshot |
| optional datasets | legacy projection, verified policy, gap report |
| cadence | legacy projection, gap report, composite registration, snapshot |
| lookback and boundaries | legacy projection, gap report, composite registration, snapshot |
| information cutoff | legacy session, legacy projection, gap report, composite registration, snapshot |
| maximum staleness | legacy projection, verified policy, gap report, snapshot |
| source policy digest | verified policy |
| feature policy digest | verified policy |
| label policy digest | verified policy |
| curriculum policy digest | verified policy |
| private namespace digest | legacy session, existing private-learning state |
| training ledger digest | legacy session, verified policy, existing private-learning state |

Empty values, conflicts, cutoff changes, shortened lookbacks, changed required
or optional datasets, incomplete required evidence, optional-evidence
misclassification, snapshot mismatches, and protected timestamp overlap all
fail closed with an explicit blocker.

## Policy compatibility proof

Opaque policy identities from different contract versions are recorded rather
than assumed equal. Compatibility is determined by the document-defined
semantic fields: required datasets, optional datasets, allowed market, cadence,
lookback, and staleness. The observed proof recorded:

```text
legacy_policy_digest = c1c648c523071a0a
current_policy_digest = c820cc07902bac08
semantically_compatible = true
proof_digest = fe4b9bb30167d9d0
```

An explicit nonempty source-policy digest that disagrees with the verified
policy is rejected. The canonical intent is then generated and checked through
the ordinary intent validator; no migration-only validation exception exists.

## Canonical result

```text
legacy_session_digest = 83b60e4a859e8287
legacy_intent_digest = 510f3e4d6475a6d1
canonical_gap_digest = cd5dbebee4063d34
composite_registration_digest = abe6bec30be0d1f4
merged_snapshot_digest = 07d65faef630a786

canonical_intent_digest = 4fc04796aab808e4
canonical_view_digest = 06d3e43b06497f42
migration_proof_digest = 9263b4c2b806dbd5
migration_journal_digest = 4c1f89afd96fe635
```

The view binds exactly the verified 312-row snapshot. Required evidence is
complete; optional evidence remains unavailable; the decision gate is `Ready`;
the resolution status is `OptionalEvidenceUnavailable`. Snapshot chronology,
content identity, cutoff, read-only state, sanitization, credential-free
provenance, and protected timestamp exclusions are verified.

## Persistence and replay

Five independent manual-Protobuf artifacts persist the canonical intent,
canonical view, policy compatibility proof, migration proof, and migration
journal. Every write uses create-new temporary storage, flush, `sync_all`,
temporary reopen and semantic validation, atomic rename, and final reopen and
validation. Existing identical identities are duplicate-rejected.

The first verified execution wrote five migration artifacts. The second wrote
none, duplicate-rejected all five, and reported `AlreadyMigrated`. The ordinary
reader reopened the persisted intent and view before the V1 pipeline consumed
them.

## Candidate and future-evaluation result

Momentum deterministically froze this validation-only family:

| Participant | Qualification |
| --- | --- |
| `ConstantProbabilityBaselineV1` | `Qualified` |
| `LinearMomentumBaselineV1` | `Qualified` |
| `FrozenMambaHeadV1` | `RejectedProbabilityCollapse` |

The family digest is `72cd657ea8a1f039`. Historical-test row, label,
inference, metric, checkpoint-selection, and identity-influence access remained
zero. Participant and family identities exclude validation metrics and
qualification receipts. No winner was selected, and the family is ineligible
for active use, promotion, and reward.

Because not every participant qualified, future evaluation registration was
`QualificationBlocked` with `validation_qualification_invalid`. No minimum
accepted timestamp was assigned and no evaluation artifact was written. The
existing protected registrations, four protected timestamps, source boundary,
provider-finality boundary, and reserved ranges remain unchanged. Cycle/Risk
remained `ProviderContractUnverified`; Value/Quality remained
`TrainerUnavailable`.

## Reward and authority replay

The existing one-time opening replay remained `Opened`, with one opening
attempt and two opened events. Momentum remained
`MissedMaterialOpportunity`; Cycle/Risk remained `CorrectUncertainty`. Both
eligibility results remained `IneligibleMinimumSamples`.

All new network, transport, credential, prospective-row, prospective-label,
future-evaluation, active-model, Chair, vote, reward, penalty, voice, cooldown,
promotion, quarantine, and execution counters are zero. The active committee
count remains three.
