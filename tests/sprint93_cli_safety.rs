use std::process::Command;

#[test]
fn sprint93_help_exposes_only_research_local_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for command in [
        "sprint93-timeout-attribution",
        "real-timeout-attribution",
        "real-no-run-diagnostic-pass",
        "real-full-diagnostic-pass",
        "cargo-message-capture",
        "active-rustc-snapshot",
        "target-dir-growth",
        "cargo-target-progress-timeline",
        "quiet-vs-diagnostic-gate",
        "krx-non-primary-proof",
        "unknown-timeout-closure",
        "workspace-timeout-attribution-decision",
        "dashboard-renderer-entry-release-gate",
        "dashboard-renderer-reduction-hold",
        "workspace-gate-recovery-v10",
        "remaining-blocker-queue-v9",
        "safety-coverage-preservation-v9",
        "control-tower-timeout-attribution",
    ] {
        assert!(stdout.contains(command), "missing command {command}");
    }
    for phrase in [
        "timeout attribution only",
        "diagnostic, not acceptance",
        "diagnostic, not full gate",
        "secret-safe capture",
        "redaction is enforced",
        "DashboardRenderer needs proof",
        "entry only",
        "reduction not started",
        "read-only",
    ] {
        assert!(stdout.contains(phrase), "missing phrase {phrase}");
    }
    let sprint93_section = &stdout[stdout
        .find("sprint93-timeout-attribution")
        .expect("sprint93 section start")
        ..stdout
            .find("system-benchmark-diff")
            .expect("sprint93 section end")];
    for forbidden in [
        "mamba-runtime",
        "gated-deltanet",
        "
  live",
        "
  order",
        "
  broker",
        "
  account",
    ] {
        assert!(!sprint93_section.contains(forbidden));
    }
}

#[test]
fn sprint93_commands_reject_remote_paths() {
    for command in [
        "sprint93-timeout-attribution",
        "real-timeout-attribution",
        "real-no-run-diagnostic-pass",
        "real-full-diagnostic-pass",
        "cargo-message-capture",
        "active-rustc-snapshot",
        "target-dir-growth",
        "cargo-target-progress-timeline",
        "quiet-vs-diagnostic-gate",
        "krx-non-primary-proof",
        "unknown-timeout-closure",
        "workspace-timeout-attribution-decision",
        "dashboard-renderer-entry-release-gate",
        "dashboard-renderer-reduction-hold",
        "workspace-gate-recovery-v10",
        "remaining-blocker-queue-v9",
        "safety-coverage-preservation-v9",
        "control-tower-timeout-attribution",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("run");
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("config path must be local"));
    }
}
