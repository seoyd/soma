mod support;

use soma_zero::WorkspaceDiagnosticEvidenceMatrixV1;
use support::sprint112_support::{read_fixture, run_sprint112};

#[test]
fn workspace_diagnostic_evidence_matrix_has_all_rows_and_supporting_only_truth() {
    let bundle = run_sprint112(
        "soma_workspace_diagnostic_evidence_matrix_v1.toml",
        "diagnostic-matrix",
    );
    let expected: WorkspaceDiagnosticEvidenceMatrixV1 =
        read_fixture("sprint112_data/diagnostic_evidence_matrix_expected.json");
    assert_eq!(bundle.workspace_diagnostic_evidence_matrix_v1, expected);
    let rows = bundle
        .workspace_diagnostic_evidence_matrix_v1
        .evidence_rows
        .iter()
        .map(|row| row.row_name.as_str())
        .collect::<Vec<_>>();
    for required in [
        "CargoCheck",
        "CargoBuild",
        "CargoNoRun",
        "CargoFull",
        "CargoJson",
        "Nextest",
        "Sccache",
        "RustcTimeline",
        "TargetStall",
        "FanoutMap",
    ] {
        assert!(rows.contains(&required), "missing row {required}");
    }
    assert!(
        !bundle
            .workspace_diagnostic_evidence_matrix_v1
            .supports_acceptance
    );
}
