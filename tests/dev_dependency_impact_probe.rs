mod support;

use soma_zero::{DevDependencyImpactProbeReportStatus, Sprint88SevenBlockerRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn dev_dependency_impact_probe_lists_known_candidates_and_is_deterministic() {
    let config = sprint::sprint88_config_from_example(
        "soma_dev_dependency_impact_probe.toml",
        "dev-dependency-impact",
    );
    let first = Sprint88SevenBlockerRecoveryRunner::default()
        .run_dev_dependency_impact_probe(&config)
        .expect("first");
    let second = Sprint88SevenBlockerRecoveryRunner::default()
        .run_dev_dependency_impact_probe(&config)
        .expect("second");
    assert_eq!(
        first.report_status,
        DevDependencyImpactProbeReportStatus::DevDependencyImpactReadyWithWarnings
    );
    assert!(
        first
            .suspected_dependencies
            .contains(&"serde_json".to_string())
    );
    assert!(
        first
            .suspected_dependencies
            .contains(&"tempfile".to_string())
    );
    assert!(first.suspected_dependencies.contains(&"insta".to_string()));
    assert!(
        first
            .dependency_impact_by_family
            .contains_key("CandleExpansionOps")
    );
    assert_eq!(first, second);
}
