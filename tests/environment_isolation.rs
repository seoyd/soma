#[path = "support/sprint58_support.rs"]
mod sprint58_support;

use soma_zero::EnvironmentIsolationRunner;

#[test]
fn environment_isolation_reports_deterministic_test_policy() {
    let out = sprint58_support::output_dir("environment-isolation");
    let report = EnvironmentIsolationRunner::default()
        .run(&sprint58_support::env_isolation_config(&out))
        .expect("env report");
    assert!(report.shell_env_ignored_in_tests);
    assert!(report.deterministic_env_applied);
    assert!(!report.leaks_detected);
}
