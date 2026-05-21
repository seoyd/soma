mod common;

use soma_zero::{ConfigGenerationPolicy, build_real_evidence_rerun_plan};

#[test]
fn ready_plan_includes_real_evidence_batch_and_ablation_commands() {
    let config = common::onboarding_config("rerun-plan", "generic_ohlcv_valid_alt.csv");
    let report = common::run_preflight(&config);
    let plan = build_real_evidence_rerun_plan(&config, report, ConfigGenerationPolicy::ReadyOnly);
    assert!(
        plan.suggested_commands
            .iter()
            .any(|cmd| cmd.contains("real-evidence"))
    );
    assert!(
        plan.suggested_commands
            .iter()
            .any(|cmd| cmd.contains("batch"))
    );
    assert!(
        plan.suggested_commands
            .iter()
            .any(|cmd| cmd.contains("ablation"))
    );
}

#[test]
fn non_ready_plan_keeps_caveats_and_skips_runnable_commands() {
    let mut config = common::onboarding_config("rerun-plan-blocked", "generic_ohlcv_valid.csv");
    config.min_rows_for_preflight = 100;
    let report = common::run_preflight(&config);
    let plan = build_real_evidence_rerun_plan(&config, report, ConfigGenerationPolicy::ReadyOnly);
    assert!(plan.suggested_commands.is_empty());
    assert!(plan.caveats.iter().any(|item| item.contains("local-only")));
}
