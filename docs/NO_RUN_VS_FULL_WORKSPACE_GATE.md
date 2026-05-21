# No-Run vs Full Workspace Gate

`cargo test --workspace --no-run --quiet` is compile-only diagnosis. It can prove that the workspace finished compiling, but it does **not** prove that test logic executed successfully.

The final acceptance gate is still `cargo test --workspace --quiet`. If that run does not finish, Sprint 87 must report the gate as still blocked.
