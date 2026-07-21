# Sprint 71R Report

## 1. Prospective-lane freeze audit

The existing prospective registrations, acquisitions, events, journals,
maturity plans, receipts, and capsules were kept outside the learning-data
namespace. Eight protected artifact identities were recorded before work.

## 2. Existing acquisition architecture audit

The implementation reuses `AgentDataIntent`, `AgentDataPolicy`,
`AcquisitionPlan`, `AcquisitionRequest`, `DataAcquisitionBroker`, the provider
registry, snapshot provenance, semantic digests, and existing Protobuf storage
patterns. No second acquisition framework was added.

## 3. Neutral data-plane implementation

The existing acquisition module now provides learning intents, four visibility
classes, authorized agent views, neutral planning, provenance manifests,
independence and Chair-firewall proofs, and an isolated pilot plan.

## 4. Agent intent ownership

Exactly three intents derive from the active agent states, configured universe,
and existing policies. Their semantic collections and policy identities are
stable, validated, and distinct. The broker rejects invalid intent semantics
without rewriting valid intents.

## 5. Deduplication result

Equivalent cross-agent requests collapse to one existing acquisition request
while preserving both requester sets and agent mapping. Cadence or time-range
differences remain separate requests.

## 6. Three independent views

Three view identities derive at runtime. Identical shared raw evidence can fan
out to multiple views while feature, label, private namespace, and training
ledger identities remain distinct.

## 7. Private-data isolation

Cross-agent private references, unauthorized datasets, and post-cutoff evidence
reject. Missing required evidence creates an abstaining empty-source view.

## 8. Chair firewall

The authority proof denies all seven learning-data actions to Chair. Learning
intents and views are not routed through Chair input, voting, or evaluation.

## 9. Protobuf storage

Agent views use a manually derived Protobuf payload and canonical envelope.
Decode verifies magic, major version, schema, kind, payload length and digest,
semantic digest, source identities, agent identity, and view invariants.
Writes flush and sync a temporary file, reopen and verify it, atomically rename,
then reopen and verify the final artifact.

## 10. JSON compatibility

Existing JSON remains unchanged. Explicit learning-view migration validates the
source, writes a verified `.pb` sidecar, and confirms the source bytes did not
change. JSON and Protobuf bytes are not semantic identity material.

## 11. Optional network-pilot result

The result is `DeferredToProtectProspectiveEvaluation`. No request was made and
no runtime learning artifact was created.

## 12. Tests and complete verification

Focused acquisition tests pass `20/20`. Complete default and Metal verification
also passed sequentially with one build job and one test thread. Default counts
are `393/404/12`; Metal counts are `394/404/12`. Formatting, both checks, and
working/cached diff checks pass. The five pre-existing warnings are unchanged.

## 13. Network and authority counters

Network requests, credential reads, prospective mutations, label reads, Chair
decisions, votes, rewards, penalties, voice changes, and executions are zero.
The active committee count remains three.

## 14. Protected-artifact freeze

All eight protected identities match their before-state values after focused
and complete verification. The pending outcome receipt, raw response, and
evidence capsule remain absent. No runtime learning-data file remains.

## 15. Files changed

The existing acquisition module and data exports were extended. Two focused
architecture documents and this report were added. No Rust module, fixture,
schema-generation file, or build script was created.

## 16. File boundaries

The execution-guidance artifact is absent from source, documentation,
configuration, fixtures, schemas, and payloads. The unrelated untracked note
remains unread, unmodified, unstaged, uncommitted, and unreferenced.

## 17. What was proven

The local architecture can derive independent intents and views, deduplicate
shared requests, isolate private data, abstain on missing evidence, deny Chair
authority, and persist semantic views through a verified Protobuf boundary.

## 18. What remains unproven

No internet evidence was acquired or consumed, no model was trained or
improved, and no prospective performance, reward, penalty, Chair learning, or
trading readiness was established.

## 19. Commit/push result

The verified change set targets
`agent/sprint71r-neutral-learning-data-plane`. Exact commit and push status are
reported in the final handoff.

## 20. Next Sprint recommendation

Review the neutral data-plane API and its Protobuf field reservations before
authorizing any isolated research-only provider pilot. Keep active model
training and prospective evaluation as separate explicit work.
