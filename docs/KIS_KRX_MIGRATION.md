# KIS vs KRX migration

- Operational reason for KIS primary: one bounded market-data-only path can cover Korean equity and eligible US equity collection.
- KRX remains exchange-reference and fallback for Korean equity.
- Do not mix KIS broker endpoints into research market-data flows.
- CLI: `soma-experiment kis-krx-migration --config examples/soma_kis_krx_migration.toml`.
