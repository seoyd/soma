mod common;
#[path = "support/sprint45_support.rs"]
mod sprint45_support;

use std::fs;

use soma_zero::CompleteRowClosureRunner;

#[test]
fn closure_runner_writes_deterministic_outputs() {
    let mut row = sprint45_support::row("deterministic");
    row.outcome_reference_available = false;
    row.baseline_reference_available = false;
    row.no_trade_counterfactual_available = false;
    row.risk_denied_counterfactual_available = false;
    let bundle_path = sprint45_support::write_bundle("closure-deterministic", vec![row]);
    let config = sprint45_support::closure_config("closure-deterministic", bundle_path);
    let first = CompleteRowClosureRunner::default()
        .run(&config)
        .expect("first");
    let first_text =
        fs::read_to_string(config.output_dir().join("complete_row_closure_summary.txt"))
            .expect("summary");
    let second = CompleteRowClosureRunner::default()
        .run(&config)
        .expect("second");
    let second_text =
        fs::read_to_string(config.output_dir().join("complete_row_closure_summary.txt"))
            .expect("summary");
    assert_eq!(first.final_summary, second.final_summary);
    assert_eq!(first_text, second_text);
}
