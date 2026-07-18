# Chair Shadow Owner Advisory Review

## Scope

This offline, retrospective-only path connects an audited owner advisory to a verified Chair Shadow observation report. It records deterministic consideration only. It does not create or alter a Chair decision, vote, model state, risk policy, reward, penalty, speaking right, Risk Governor handoff, paper action, or execution.

## Input boundary

`ChairShadowOwnerAdvisoryReviewInputV0` binds the owner input fingerprint to the observation packet, receipt, and Chair Shadow firewall digests. The input requires retrospective-only evidence and rejects any decision or candidate context. The original owner input and observation report are read only.

## Policy and status

The review calls the existing `validate_owner_input` function unchanged. Conservative risk-tightening, reanalysis, and evidence requests are acknowledged without mutation or replay. Paper confirmation and candidate hold/dismiss have no eligible target in this observation path. Diagnostic and free-form inputs remain diagnostic; unknown and forbidden runtime requests fail closed.

## Deterministic receipt

The result has sorted stable reason codes, a fixed status-to-explanation mapping, and a digest that excludes storage paths. Explanations never include owner identity, local paths, raw free-form text, market data, model values, probabilities, or trade actions.

## Decision firewall

`OwnerAdvisoryDecisionFirewallProofV0` asserts that an owner input cannot become a vote or Chair input, the observation cannot become a risk decision, and no Chair, owner-trade-review, risk, paper-broker, reward, penalty, or speaking-right runtime path is invoked. It also asserts that no decision and no action were created.

## Local ledger

`ChairShadowOwnerReviewLedgerV0` is a separate ignored local JSON ledger. It is atomically written, reopened and verified, sorted by owner-input fingerprint, and idempotent for the same completed review. Its digest contains no storage path.

## CLI

`--chair-shadow-owner-advisory-review` runs only deterministic fixture inputs against the locally reconstructed verified observation report. Text and JSON output expose only fingerprints, policy state, status, sorted codes, fixed explanation, review/ledger/firewall digests, and zero counters.
