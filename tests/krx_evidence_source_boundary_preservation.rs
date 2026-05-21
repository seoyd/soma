mod support;

use soma_zero::{KrxEvidenceSourceBoundaryPreservationStatus, Sprint91KrxEvidenceRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn krx_evidence_source_boundary_stays_preserved() {
    let config = sprint::sprint91_config_from_example(
        "soma_krx_evidence_source_boundary_preservation.toml",
        "krx-source-boundary",
    );
    let report = Sprint91KrxEvidenceRecoveryRunner::default()
        .run_krx_evidence_source_boundary_preservation(&config)
        .expect("report");
    assert_eq!(
        report.source_boundary_status,
        KrxEvidenceSourceBoundaryPreservationStatus::SourceBoundaryPreserved
    );
    assert!(report.official_krx_reference_preserved);
    assert!(report.official_api_collected_preserved);
    assert!(report.fixture_only_preserved);
    assert!(report.diagnostic_only_preserved);
    assert!(report.no_source_promotion);
    assert!(report.no_lookahead_preserved);
}
