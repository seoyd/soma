mod support;

use soma_zero::{
    ArtifactGoldenReusePolicyStatus, CliSmokeExecutionPolicyStatus,
    FixtureOutputDirReusePolicyStatus,
};
use support::sprint69_support as sprint;

#[test]
fn sprint84_cli_smoke_policy_and_reuse_policies_are_deterministic() {
    let first = sprint::run_sprint84_bundle(
        "soma_cli_smoke_execution_policy.toml",
        "sprint84-smoke-policy-a",
    );
    let second = sprint::run_sprint84_bundle(
        "soma_cli_smoke_execution_policy.toml",
        "sprint84-smoke-policy-b",
    );
    assert_eq!(
        first.cli_smoke_execution_policy.policy_status,
        CliSmokeExecutionPolicyStatus::SmokePolicyReady
    );
    assert_eq!(
        first.fixture_output_dir_reuse_policy.policy_status,
        FixtureOutputDirReusePolicyStatus::ReusePolicyReady
    );
    assert_eq!(
        first.artifact_golden_reuse_policy.policy_status,
        ArtifactGoldenReusePolicyStatus::GoldenReuseReady
    );
    assert!(
        first
            .fixture_output_dir_reuse_policy
            .per_test_namespace_required
    );
    assert!(first.artifact_golden_reuse_policy.fingerprint_required);
    assert_eq!(
        first.cli_smoke_execution_policy,
        second.cli_smoke_execution_policy
    );
    assert_eq!(
        first.fixture_output_dir_reuse_policy,
        second.fixture_output_dir_reuse_policy
    );
    assert_eq!(
        first.artifact_golden_reuse_policy,
        second.artifact_golden_reuse_policy
    );
}
