mod common;
#[path = "support/sprint44_support.rs"]
mod sprint44_support;

use soma_zero::OfficialReadyMatchClosureRunner;

#[test]
fn official_ready_match_closure_is_deterministic_for_same_inputs() {
    let mut first_cfg = sprint44_support::load_closure_config(
        "examples/soma_official_ready_match_close_official_replication.toml",
    );
    first_cfg.output_root = common::output_dir("sprint44-det-1").display().to_string();
    let mut second_cfg = sprint44_support::load_closure_config(
        "examples/soma_official_ready_match_close_official_replication.toml",
    );
    second_cfg.output_root = common::output_dir("sprint44-det-2").display().to_string();

    let first = OfficialReadyMatchClosureRunner::default()
        .run(&first_cfg)
        .expect("first");
    let second = OfficialReadyMatchClosureRunner::default()
        .run(&second_cfg)
        .expect("second");

    assert_eq!(
        first.closure_report.closure_status,
        second.closure_report.closure_status
    );
    assert_eq!(first.candidate_report, second.candidate_report);
    assert_eq!(first.repair_plan, second.repair_plan);
    assert_eq!(first.final_summary, second.final_summary);
}
