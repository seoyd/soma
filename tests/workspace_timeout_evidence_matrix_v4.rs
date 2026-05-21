mod support;

use soma_zero::WorkspaceTimeoutEvidenceMatrixV4;
use support::sprint118_support::{read_fixture, run_sprint118};

#[test]
fn workspace_timeout_evidence_matrix_matches_expected() {
    let bundle = run_sprint118(
        "soma_workspace_timeout_evidence_matrix_v4.toml",
        "workspace-timeout-evidence-matrix-v4",
    );
    let expected: WorkspaceTimeoutEvidenceMatrixV4 =
        read_fixture("sprint118_data/workspace_timeout_evidence_matrix_v4_expected.json");
    assert_eq!(bundle.workspace_timeout_evidence_matrix_v4, expected);
    assert!(
        !bundle
            .workspace_timeout_evidence_matrix_v4
            .supports_acceptance
    );
}
