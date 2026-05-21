mod support;

use support::sprint117_support::run_sprint117;

#[test]
fn workspace_timeout_evidence_matrix_v3_matches_expected() {
    let bundle = run_sprint117(
        "soma_workspace_timeout_evidence_matrix_v3.toml",
        "workspace-timeout-evidence-matrix-v3",
    );
    assert_eq!(bundle.workspace_timeout_evidence_matrix_v3.rows.len(), 9);
    assert!(
        !bundle
            .workspace_timeout_evidence_matrix_v3
            .supports_acceptance
    );
    assert_eq!(
        bundle.workspace_timeout_evidence_matrix_v3.status,
        "WorkspaceTimeoutEvidenceMatrixSupportingOnly"
    );
}
