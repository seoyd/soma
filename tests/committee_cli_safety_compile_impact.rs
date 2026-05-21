mod support;

use soma_zero::{CommitteeCliSafetyCompileImpactStatus, Sprint95CommitteeCliSafetyRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn compile_impact_stays_sample_backed() {
    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
        .run_committee_cli_safety_compile_impact(&sprint::sprint95_config_from_example(
            "soma_committee_cli_safety_compile_impact.toml",
            "committee-cli-safety-compile-impact",
        ))
        .expect("report");
    assert_eq!(
        report.impact_status,
        CommitteeCliSafetyCompileImpactStatus::CompileImpactSampleBacked
    );
    assert_eq!(report.committee_cli_safety_delta, Some(0));
}
