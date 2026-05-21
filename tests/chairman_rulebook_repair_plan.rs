mod support;

use std::fs;
use std::path::PathBuf;

use soma_zero::ChairmanRulebookSafetyRepairPlan;
use support::sprint100_support::run_sprint100;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("sprint100_data")
        .join(name)
}

#[test]
fn chairman_rulebook_repair_plan_matches_expected_fixture() {
    let bundle = run_sprint100(
        "soma_chairman_rulebook_repair_plan.toml",
        "chairman-rulebook-repair-plan",
    );
    let expected: ChairmanRulebookSafetyRepairPlan = serde_json::from_str(
        &fs::read_to_string(fixture_path("chairman_rulebook_repair_expected.json"))
            .expect("read fixture"),
    )
    .expect("parse fixture");
    assert_eq!(bundle.chairman_rulebook_safety_repair_plan, expected);
}
