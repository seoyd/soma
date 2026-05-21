mod support;

use soma_zero::{KrxEvidenceEndpointTemplatePreservationStatus, Sprint91KrxEvidenceRecoveryRunner};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn krx_evidence_endpoint_template_preserves_market_data_only_requirements() {
    let config = sprint::sprint91_config_from_example(
        "soma_krx_evidence_endpoint_template_preservation.toml",
        "krx-endpoint-template",
    );
    let report = Sprint91KrxEvidenceRecoveryRunner::default()
        .run_krx_evidence_endpoint_template_preservation(&config)
        .expect("report");
    assert_eq!(
        report.endpoint_status,
        KrxEvidenceEndpointTemplatePreservationStatus::EndpointTemplatePreserved
    );
    assert!(report.endpoint_template_required);
    assert!(report.missing_template_blocked);
    assert!(report.template_value_not_secret_leaked);
    assert!(report.operator_action_preserved);
    assert!(report.request_builder_still_market_data_only);
    harness::assert_no_secret_like_values(&serde_json::to_string(&report).expect("json"));
}
