# Outcome Link Depth Closure v2

Sprint 61 checks whether bounded official rows have enough outcome depth for sequence preparation.

The report tracks:

- outcome-link count
- TP / SL / TE coverage
- future-window gaps
- no-lookahead blocks

Run:

```bash
cargo run --quiet --bin soma_experiment -- outcome-link-depth-close-v2 --config examples/soma_outcome_link_depth_close_v2.toml
```

This command never enables trading, execution, or account paths.

