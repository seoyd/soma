# Control Tower Auto Refresh

Sprint 58 adds a wrapper around the existing Control Tower refresh so new KIS smoke status, secret-audit state, and updated dashboard outputs are attached in one local bundle.

## CLI

```bash
cargo run --quiet --bin soma_experiment -- control-tower-auto-refresh --config examples/soma_control_tower_auto_refresh.toml
```

## Outputs

- refreshed dashboard JSON/HTML/TXT
- next-action text
- owner action drafts
- `control_tower_auto_refresh.json`
- `control_tower_auto_refresh.txt`

## Safety

- read-only local rendering only
- no order/account controls are introduced
- secret audit state is attached to the refresh result
