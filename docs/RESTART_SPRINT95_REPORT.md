# Sprint 95 Implementation Report

## PR #25 review and merge

All eight PR #25 files were reviewed. Canonical discovery, contamination
classification, causal folds, fold-local reconstruction, aggregate metrics,
offline backfill planning, and the zero-authority boundary were verified.

The shared dataset validator, label classifier, raw encoder, bounded trainer,
interaction expander, metric helper, Protobuf helper, and atomic writer retain
their existing live semantics. The historical store is additive and isolated
from live prospective artifacts.

PR #25 preserved commits `70372e9` and `770f5d5` and was merged with merge
commit `57d782a6ec5d64ae7025b07244a992046b6fbf2f`. The remote Sprint 94 branch was
deleted and synchronized `main` matched `origin/main`.

The historical snapshot remains `91fc9425cd92ce18`, contamination audit
`00321b724d144d4f`, and walk-forward registration `0dd64fb1535a4701`.
Historical results remain `HistoricalResearchOnly`,
`NotIndependentHoldout`, and `NotProspectiveAuthority`. The older-history
backfill plan `53d2e54cf4d9ff5e` remains unexecuted.

## Live before-state and readiness

The protected live store contained 164 preexisting artifacts. The historical
store contained 5,415 artifacts. Status and dry-run did not change either
identity.

The existing live chain reopened with:

- series `36fbad02f19e8a6f`;
- event-one adoption `eb53b8ee0e988d91`;
- context-delta plan `02bff54a0886cdd4`;
- epoch-two registration `86e25b571800a7fc`;
- request fingerprint `d27a501c76a64c78`;
- one completed and one scorable event;
- reward eligibility `IneligibleMinimumSamples`.

At `2026-07-26T04:08:31Z`, actual UTC was after input finality and before
outcome finality. Two text status runs, two JSON status runs, and text and JSON
dry-runs all returned `ReadyForInputAcquisition`. They agreed on 15 reused
timestamps, missing timestamp `1784937600000`, one maximum request, zero
retries, concurrency one, no prior receipt, and all-zero work counters.

## Epoch-two input and seal

The exact registered Upbit `KRW-BTC` daily input executed once. It consumed one
request, constructed one transport, performed zero retries, and accepted
exactly the registered single finalized row.

The successful input receipt is `ec2806f2d5d234e5`; input capsule
`3a918381cf1cedfa`; and 16-row context proof `41aa5585171d28a5`. The unchanged
live participants were reconstructed with zero parameter updates, normalizer
refits, training, qualification, or event-one private-result reads.

Exactly three private predictions were sealed. Atomic prediction capsule
`f0fc2d24e1c920e4`, journal `ed46f8a8b3f4f806`, and locked outcome plan
`ae798b355d36bb74` were persisted. Probabilities, raw input values, labels,
parameters, normalizer values, and features are not published.

The operation added exactly 12 live artifacts. The 5,415 historical artifacts
and their aggregate identity remained unchanged. Event-one artifacts, event
counts, scorable counts, and reward eligibility remained unchanged.

## Replay defect and recovery

The first status replay exposed a persisted-chain validation defect. Reopened
seal files were ordered by digest filename, but validation incorrectly
expected that order to equal the capsule's participant order. The artifacts
and their digest bindings were valid.

The existing validator now resolves seals through the capsule's frozen
`(seal digest, prediction digest)` pairs. A focused regression reverses the
reopened seal order and proves completed-chain validation remains exact.

After the fix, status, dry-run, and repeated execute-input replay all returned
`PredictionAlreadySealed` with zero network requests, transports, raw loads,
participant reconstruction, feature or prediction computations, and writes.

## Authority and outcome boundary

The completed operation has:

- live input attempts: one;
- live retries: zero;
- maximum live concurrency: one;
- historical and backfill network requests: zero;
- parameter updates, normalizer refits, and training uses: zero;
- outcome requests, openings, labels, metrics, and evaluations: zero;
- winner selections and rankings: zero;
- rewards, penalties, Chair decisions, and votes: zero;
- voice, tier, cooldown, promotion, and quarantine changes: zero;
- paper and live executions: zero;
- active committee count: three.

Event two is sealed but unscored. Its outcome timestamp is
`1785024000000`, finality is `1785110400000`, maximum future outcome requests
is one, retry count is zero, and the outcome stage remains locked. Event two
does not enter completed or scorable counts until a later separately
authorized opening.

The historical warning remains only a research warning: the current learned
configurations did not beat the constant benchmark on previously consumed
historical evidence. It did not remove, replace, rank, reward, or penalize a
live participant.

Official Mamba-3 was not implemented or evaluated. Chair remained inactive.
No other-agent blocker remains; external Rust work was allowed to finish and
all Soma Rust commands were executed sequentially.

## Verification and proof boundary

Focused verification covers the directory-order recovery regression and the
complete historical and live prospective modules in Default and Metal
configurations. Full Default and Metal workspace checks and tests are run with
one build job, disabled incremental compilation, and one test thread.

Final results:

- format, Default workspace check, and Metal workspace check passed;
- live prospective focused tests: 53 of 53 in Default and Metal;
- historical replay focused tests: 43 of 43 in Default and Metal;
- full Default tests: 915 library, 404 integration, and 12 queue tests;
- full Metal tests: 916 library, 404 integration, and 12 queue tests;
- status, dry-run, and completed execute replay returned zero new work.

Proven:

- PR #25 preserves live semantics and historical zero authority;
- exact one-request live acquisition and validation;
- exact 16-row context assembly;
- three unchanged frozen participant reconstructions and seals;
- prediction-before-outcome persistence;
- completed replay with zero new work;
- event-one, historical-store, outcome, reward, Chair, and execution isolation.

Not proven:

- event-two correctness or participant performance;
- future generalization;
- participant superiority;
- reward effectiveness or Chair learning;
- official Mamba-3 behavior;
- paper- or live-trading readiness.

The next single step is to keep the outcome stage closed until
`2026-07-27T00:00:00Z`, then use a separate explicit authorization to acquire
only the locked event-two outcome.
