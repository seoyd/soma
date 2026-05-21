mod support;

use soma_zero::{
    CargoJsonProgressCaptureV6, CargoNoRunTimingCaptureV1,
    FixtureRenderCliFanoutAttributionReportV2, LinkMacroAttributionReportV2,
    Sprint111SummaryFixture, WorkspaceTimeoutRootCauseReportV2,
    build_cargo_target_stall_attribution_report_v2, build_workspace_timeout_root_cause_report_v2,
};
use support::sprint112_support::{read_fixture, run_sprint112};

#[test]
fn workspace_timeout_root_cause_v2_matches_fixture_and_separates_observed_from_inferred() {
    let bundle = run_sprint112("soma_workspace_timeout_root_cause_v2.toml", "root-cause-v2");
    let expected: WorkspaceTimeoutRootCauseReportV2 =
        read_fixture("sprint112_data/root_cause_v2_expected.json");
    assert_eq!(bundle.workspace_timeout_root_cause_report_v2, expected);
    assert!(
        !bundle
            .workspace_timeout_root_cause_report_v2
            .observed_evidence
            .is_empty()
    );
    assert!(
        !bundle
            .workspace_timeout_root_cause_report_v2
            .inferred_evidence
            .is_empty()
    );

    let summary = Sprint111SummaryFixture::default();
    let capture = CargoJsonProgressCaptureV6 {
        attempted: false,
        ..bundle.cargo_json_progress_capture_v6.clone()
    };
    let no_run = CargoNoRunTimingCaptureV1 {
        finished: false,
        ..bundle.cargo_no_run_timing_capture_v1.clone()
    };
    let target = build_cargo_target_stall_attribution_report_v2(&capture, &no_run, &summary);
    let partial = build_workspace_timeout_root_cause_report_v2(
        &summary,
        &target,
        &bundle.link_macro_attribution_report_v2,
        &bundle.fixture_render_cli_fanout_attribution_report_v2,
    );
    assert_eq!(partial.status, "TimeoutRootCausePartiallyIsolated");
    let empty_target = build_cargo_target_stall_attribution_report_v2(
        &CargoJsonProgressCaptureV6 {
            stalled_candidates: Vec::new(),
            last_targets: Vec::new(),
            ..bundle.cargo_json_progress_capture_v6.clone()
        },
        &no_run,
        &summary,
    );
    let ambiguous = build_workspace_timeout_root_cause_report_v2(
        &Sprint111SummaryFixture {
            observed_root_cause_evidence: Vec::new(),
            inferred_root_cause_evidence: Vec::new(),
            high_fanout_families: Vec::new(),
            ..summary.clone()
        },
        &empty_target,
        &LinkMacroAttributionReportV2 {
            observed_candidates: Vec::new(),
            inferred_candidates: Vec::new(),
            ..bundle.link_macro_attribution_report_v2.clone()
        },
        &FixtureRenderCliFanoutAttributionReportV2 {
            fixture_fanout: Vec::new(),
            render_fanout: Vec::new(),
            cli_fanout: Vec::new(),
            helper_fanout: Vec::new(),
            ..bundle
                .fixture_render_cli_fanout_attribution_report_v2
                .clone()
        },
    );
    assert_eq!(ambiguous.status, "TimeoutRootCauseStillAmbiguous");
}
