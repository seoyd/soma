mod support;

use soma_zero::WorkspaceNoRunProgressTraceV1;
use support::sprint111_support::{read_fixture, run_sprint111};

#[test]
fn workspace_no_run_progress_trace_uses_diagnostic_trace() {
    let bundle = run_sprint111(
        "soma_workspace_no_run_progress_trace_v1.toml",
        "workspace-no-run-progress-trace-v1",
    );
    let expected: WorkspaceNoRunProgressTraceV1 =
        read_fixture("sprint111_data/no_run_progress_trace_expected.json");
    assert_eq!(bundle.workspace_no_run_progress_trace_v1, expected);
    assert!(!bundle.workspace_no_run_progress_trace_v1.attempted);
    assert!(
        bundle
            .workspace_no_run_progress_trace_v1
            .last_seen_target
            .is_some()
    );
    assert!(
        bundle
            .workspace_no_run_progress_trace_v1
            .progress_event_count
            > 0
    );
}
