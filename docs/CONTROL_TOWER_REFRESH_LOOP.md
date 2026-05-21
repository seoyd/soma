# Control Tower Refresh Loop

`control-tower-refresh` rebuilds the local dashboard from local artifacts only.

Outputs:

- `dashboard_state_v1.json`
- `dashboard_v1.txt`
- `dashboard_v1.html`
- `dashboard_next_actions.txt`

It is read-only. It does **not** execute orders, open network sessions, or expose secrets.
