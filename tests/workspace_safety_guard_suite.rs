mod common;
#[path = "support/sprint59_support.rs"]
mod sprint59_support;
#[path = "support/sprint60_support.rs"]
mod sprint60_support;

use soma_zero::{
    ControlTowerUiReadinessStatus, DashboardServeReport, DashboardServeStatus,
    EvidenceHardeningRunner, LiveSafetyStatus, ReasonCode, RuntimeMode, RuntimeStage, RuntimeState,
    SystemIntegrationReviewRunner, UIFrameworkCurrentChoice, UIFrameworkFutureChoice,
    UIFrameworkRejectedOption, build_live_safety_report,
};

#[test]
fn live_mode_is_disabled() {
    assert_eq!(
        RuntimeMode::from_active_label("live"),
        Err(ReasonCode::LiveModeDisabled)
    );
}

#[test]
fn live_safety_report_stays_research_only() {
    let report = build_live_safety_report(
        &[
            "run".to_string(),
            "batch".to_string(),
            "ai-benchmark".to_string(),
            "core-check".to_string(),
        ],
        false,
    );
    assert!(!report.live_mode_constructible);
    assert!(!report.broker_path_present);
    assert!(!report.order_execution_path_present);
    assert!(!report.account_api_path_present);
    assert!(!report.runtime_llm_path_present);
    assert_eq!(report.status, LiveSafetyStatus::SafeResearchOnly);
}

#[test]
fn runtime_transitions_keep_live_execution_blocked() {
    assert_eq!(
        RuntimeMode::from_active_label("live"),
        Err(ReasonCode::LiveModeDisabled)
    );

    let mut live_disabled = RuntimeState::new(RuntimeMode::LiveDisabled);
    assert_eq!(
        live_disabled
            .transition_to(RuntimeStage::PaperExecution, false)
            .expect_err("live-disabled paper execution must fail"),
        ReasonCode::LiveModeDisabled
    );

    let mut research = RuntimeState::new(RuntimeMode::Research);
    assert_eq!(
        research
            .transition_to(RuntimeStage::PaperExecution, false)
            .expect_err("paper execution without risk stage must fail"),
        ReasonCode::RiskDenied
    );
    let mut outcome = RuntimeState::new(RuntimeMode::Research);
    assert_eq!(
        outcome
            .transition_to(RuntimeStage::OutcomeEvaluation, false)
            .expect_err("outcome evaluation without decision record must fail"),
        ReasonCode::NoTradeDefault
    );
}

#[test]
fn failed_stage_blocks_later_execution_unless_diagnostics_only() {
    let mut research = RuntimeState::new(RuntimeMode::Research);
    research.stage = RuntimeStage::Failed;
    assert!(
        research
            .transition_to(RuntimeStage::LoadConfig, false)
            .is_err()
    );

    let mut diagnostics = RuntimeState::new(RuntimeMode::DiagnosticsOnly);
    diagnostics.stage = RuntimeStage::Failed;
    assert!(
        diagnostics
            .transition_to(RuntimeStage::LoadConfig, false)
            .is_ok()
    );
}

#[test]
fn control_tower_ui_fixtures_keep_order_and_account_controls_out() {
    let config =
        sprint59_support::review_config_from_example("soma_system_review_full.toml", "ui-ready");
    let bundle = SystemIntegrationReviewRunner::default()
        .run(&config)
        .expect("run full ui review");
    assert_eq!(
        bundle.control_tower_ui_readiness_report.readiness_status,
        ControlTowerUiReadinessStatus::Ready
    );
    assert!(bundle.control_tower_ui_readiness_report.no_order_buttons);
    assert!(bundle.control_tower_ui_readiness_report.no_account_panels);
}

#[test]
fn safety_block_fixture_blocks_ui_readiness() {
    let config = sprint59_support::review_config_from_example(
        "soma_system_review_safety_block.toml",
        "ui-blocked",
    );
    let bundle = SystemIntegrationReviewRunner::default()
        .run(&config)
        .expect("run safety-block ui review");
    assert_eq!(
        bundle.control_tower_ui_readiness_report.readiness_status,
        ControlTowerUiReadinessStatus::Blocked
    );
}

#[test]
fn dashboard_serve_is_deferred_for_safety() {
    let report = DashboardServeReport::deferred();
    assert_eq!(report.status, DashboardServeStatus::DeferredForSafety);
    assert!(report.deferred_reason.contains("deferred"));
    assert_eq!(report.bind_address, "127.0.0.1");
    assert_eq!(report.methods_allowed, "GET");
}

#[test]
fn ui_framework_decision_stays_static_now_and_tauri_later() {
    let config = sprint60_support::config_from_example(
        "soma_ui_framework_decision.toml",
        "workspace-safety-ui-framework",
    );
    let report = EvidenceHardeningRunner::default()
        .run(&config)
        .expect("run ui framework decision")
        .ui_framework_decision_report;
    assert_eq!(
        report.current_choice,
        UIFrameworkCurrentChoice::StaticHtmlJsonTxt
    );
    assert_eq!(
        report.future_choice,
        UIFrameworkFutureChoice::TauriSvelteDesktop
    );
    assert!(
        report
            .rejected_options
            .contains(&UIFrameworkRejectedOption::ReactNextWeb)
    );
    assert!(
        report
            .rejected_options
            .contains(&UIFrameworkRejectedOption::CloudDashboard)
    );
}
