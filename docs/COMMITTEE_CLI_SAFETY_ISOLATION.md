# Committee CLI Safety Isolation

`committee_cli_safety` remains a high-risk isolated target in Sprint 88.

The isolation policy preserves:

- research-only help text
- remote config rejection
- no runtime LLM path
- no persona expansion
- no broker/order/account path
- deterministic help output

Sprint 88 does not merge this target into grouped suites unless isolation can be proven equally safe.
