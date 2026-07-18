# Prospective external row admission

This offline protocol is the only local path from one independently acquired
future BTC daily row to pre-label Shadow events. It does not perform a provider
request, construct a transport, read a credential or consent setting, access a
label, retrain a model, alter a frozen artifact, or grant authority.

## Contract audit and registration

Before opening any candidate capsule, the command validates both sealed
contracts. A digest-bound registration fixes the Momentum and Cycle/Risk
challenge identities, cutoffs, latest consumed timestamp, canonical provider,
market, symbol, cadence, accepted source classes, frozen model
configuration, pre-label isolation, shared-raw-only fanout, and zero network,
reward, and authority requirements. The registration is atomically written,
reopened, and validated before candidate parsing.

## Capsule admission

The capsule is one sanitized, read-only, credential-free, finalized canonical
row identity. It includes source class and export digests but no path. It must
be from an approved credential-free export or verified independent canonical
export, identify BTC daily data from the registered provider, contain finite
OHLCV values with valid high/low relationships and nonnegative volume, be
strictly later than all registered cutoffs, have no label/model-output access,
and not duplicate or follow an already admitted later row. Absence of a
qualified capsule is a valid `AwaitingQualifiedExternalRow` result.

## Independent fanout and sealing

Admission creates one shared raw-evidence reference containing only identity
metadata, digests, timestamp, cutoff checks, eligibility flags, and
`label_accessed=false`. Momentum and Cycle/Risk validate that reference against
their own frozen contracts independently. A valid side may seal only an
explicit abstention when external inference support is unavailable. Normal
status output exposes counts, statuses, and digests, never raw rows,
probabilities, labels, feature data, support details, or local paths.

## Storage and authority

Each local store writes atomically through a temporary file and validates on
reopen. Vaults and journals are append-only, accept one row/event identity per
agent/timestamp, and preserve unopened labels. The resulting state is awaiting
maturity and remains ineligible for reward application. It cannot vote,
promote, trade, change the Chair, mutate Risk governance, or alter Paper
execution.
