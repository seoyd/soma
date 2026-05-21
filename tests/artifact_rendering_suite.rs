mod common;
mod support;

use std::fs;

use soma_zero::{
    ArtifactRenderCachePlanStatus, ArtifactRenderCachePolicy, ArtifactRenderingCostReportStatus,
    DashboardRenderConfig, DashboardRenderer, DashboardSnapshotBuilder, DashboardSourceConfig,
    DashboardState, ProviderPanel, ReasonCode, build_artifact_render_cache_plan,
    build_artifact_rendering_cost_report, redact_dashboard_state,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

fn secret_state() -> DashboardState {
    let mut state = DashboardState::default();
    state.dashboard_id = "secret-dashboard".to_string();
    state.provider_panel = ProviderPanel::default();
    state.provider_panel.kis_status.endpoint_policy_status =
        "https://example.kis.local/path?token=abc123".to_string();
    state.warnings = vec![
        "KIS_APP_KEY=secret-value".to_string(),
        "KIS_APP_SECRET=top-secret-value".to_string(),
        "KIS_WS_APPROVAL_KEY=approval-secret".to_string(),
        "KRX_API_KEY=krx-secret".to_string(),
        "KIS_BASE_URL=https://example.kis.local/path?token=abc123".to_string(),
        "password=hunter2".to_string(),
    ];
    state.reason_codes = vec![ReasonCode::DeterministicPath];
    state.with_fingerprint()
}

#[test]
fn artifact_render_cache_plan_requires_deterministic_fingerprint() {
    let report = build_artifact_rendering_cost_report("artifact-cost");
    let plan = build_artifact_render_cache_plan("artifact-cache", &report);
    assert_eq!(
        plan.cache_policy,
        ArtifactRenderCachePolicy::LocalFingerprintCache
    );
    assert!(plan.deterministic_fingerprint_required);
    assert!(
        plan.invalidation_rules
            .iter()
            .any(|rule| rule.contains("re-render on cache miss"))
    );
    assert_eq!(
        plan.plan_status,
        ArtifactRenderCachePlanStatus::RenderCachePlanReady
    );
    assert_eq!(
        plan,
        build_artifact_render_cache_plan("artifact-cache", &report)
    );
}

#[test]
fn artifact_rendering_cost_report_detects_repeated_render_kinds() {
    let report = build_artifact_rendering_cost_report("artifact-cost");
    for artifact_kind in ["txt", "json", "bundle", "storage_report"] {
        assert!(
            report
                .records
                .iter()
                .any(|record| record.artifact_kind == artifact_kind)
        );
    }
    assert!(
        report
            .records
            .iter()
            .all(|record| record.deterministic_fingerprint_available)
    );
    assert_eq!(
        report.report_status,
        ArtifactRenderingCostReportStatus::ArtifactCostReady
    );
    assert_eq!(
        report,
        build_artifact_rendering_cost_report("artifact-cost")
    );
}

#[test]
fn secret_values_are_redacted_from_dashboard_state() {
    let (state, report) = redact_dashboard_state(&secret_state()).expect("redact");
    let json = state.to_json_string().expect("json");
    for secret in [
        "secret-value",
        "top-secret-value",
        "approval-secret",
        "krx-secret",
        "hunter2",
        "abc123",
    ] {
        assert!(!json.contains(secret));
    }
    assert!(report.passed);
    assert!(!report.redacted_field_paths.is_empty());
    harness::assert_no_secret_like_values(&json);
}

#[test]
fn redaction_report_is_deterministic() {
    let first = serde_json::to_string(&redact_dashboard_state(&secret_state()).expect("redact").1)
        .expect("json");
    let second = serde_json::to_string(&redact_dashboard_state(&secret_state()).expect("redact").1)
        .expect("json");
    harness::assert_deterministic_text(&first, &second);
}

#[test]
fn dashboard_snapshot_and_render_outputs_are_deterministic() {
    let mut source = DashboardSourceConfig::from_toml_path(&common::example_path(
        "soma_dashboard_source_kis_control_tower.toml",
    ))
    .expect("source config");
    source.output_root = common::sprint52_output_dir("artifact-rendering-suite-source")
        .display()
        .to_string();
    let first_state = DashboardSnapshotBuilder::default()
        .build(&source)
        .expect("state");
    let second_state = DashboardSnapshotBuilder::default()
        .build(&source)
        .expect("state");
    assert_eq!(
        first_state.to_json_string().expect("json"),
        second_state.to_json_string().expect("json")
    );

    let mut render = DashboardRenderConfig::from_toml_path(&common::example_path(
        "soma_dashboard_render_static.toml",
    ))
    .expect("render config");
    render.output_root = common::sprint52_output_dir("artifact-rendering-suite-render")
        .display()
        .to_string();
    let first = DashboardRenderer::default()
        .render(&render)
        .expect("render");
    let second = DashboardRenderer::default()
        .render(&render)
        .expect("render");
    let first_html = fs::read_to_string(first.html_path.expect("html 1")).expect("html 1");
    let second_html = fs::read_to_string(second.html_path.expect("html 2")).expect("html 2");
    harness::assert_deterministic_text(&first_html, &second_html);
}

#[test]
fn sprint84_render_outputs_stay_safety_clean() {
    let bundle = sprint::run_sprint84_bundle(
        "soma_sprint84_test_cost_reduce.toml",
        "artifact-rendering-suite-sprint84",
    );
    let panel_json = serde_json::to_string(&bundle.control_tower_test_cost_panel).expect("panel");
    harness::assert_no_secret_like_values(&bundle.final_summary);
    harness::assert_no_order_account_fields(&panel_json);
    harness::assert_no_runtime_fields(&panel_json);
}
