#[path = "support/shared_fixture_harness.rs"]
mod shared_fixture_harness;
mod support;

use support::sprint69_support as sprint;

#[test]
fn sprint97_cli_outputs_remain_secret_safe_and_read_only() {
    let bundle = sprint::run_sprint97_bundle(
        "soma_sprint97_counterfactual_backfill_recover.toml",
        "sprint97-cli-safety",
    );
    let text =
        serde_json::to_string_pretty(&bundle.control_tower_counterfactual_backfill_recovery_panel)
            .expect("serialize panel");
    shared_fixture_harness::assert_no_secret_like_values(&text);
    shared_fixture_harness::assert_no_order_account_fields(&text);
    shared_fixture_harness::assert_no_runtime_fields(&text);
}
