# Prospective holdout policy

Historical rows previously used for fitting, normalization, validation, testing,
diagnostics, counterfactual evaluation, or model-design decisions are consumed.
They are not a pristine final holdout, even if an earlier report called an
evaluation split sealed.

The prospective cutoff is the maximum timestamp in the deterministic consumed
evidence ledger. Only rows strictly after that cutoff may later become holdout
candidates. The initial manifest is sealed, unopened, and has no label access;
when no later rows have been collected its status is
`PolicySealedNoFutureRows`.

Prospective rows first enter a separate append-only vault under a sealed
challenge capsule. They are not merged into historical snapshots, development
packs, or training inputs. Before one-time opening, the vault and prediction
journal expose only count and digest status; neither future values, sealed
probabilities, labels, nor interim quality metrics are displayed.

The challenge freezes one Shadow-only candidate, frozen Linear and Constant
comparators, feature/label/support/collapse policies, metric policy, and
opening requirements before future-row access. Candidate substitution,
comparator refitting, early label access, vault/journal modification, or a
second opening permanently invalidates the challenge.

When an immutable history expands, its full accepted timestamp range is added
to the usage ledger as diagnostics/development evidence in addition to any
campaign-specific records. This prevents an uninspected portion of a newly
merged snapshot from being misrepresented as a fresh holdout.

Later rows may accumulate until the predeclared row and window requirements are
met. At that point one frozen holdout pack may be opened once to evaluate one
preselected model and policy. It must not be reused for redesign.

Fitting, validation selection, threshold tuning, candidate creation, pooling or
feature selection, support-threshold changes, or viewing labels for model
selection before the one-time opening invalidates the manifest. Future live
Shadow operations require their own protocol and are not final-holdout research.
