mod support;

use serde_json::to_value;
use support::sprint104_support::{read_fixture, run_sprint104};

#[test]
fn control_tower_paper_candidate_lifecycle_panel_matches_fixture_and_stays_read_only() {
    let bundle = run_sprint104(
        "soma_sprint104_dual_agent_paper_lifecycle.toml",
        "control_tower_paper_candidate_lifecycle",
    );
    let actual = to_value(&bundle.control_tower_paper_candidate_lifecycle_panel).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint104_data/control_tower_paper_lifecycle_expected.json");
    assert_eq!(actual, expected);
    assert!(
        bundle
            .control_tower_paper_candidate_lifecycle_panel
            .candidate_state_summary
            .contains_key("Watch")
    );
    assert!(
        bundle
            .control_tower_paper_candidate_lifecycle_panel
            .warnings
            .iter()
            .any(|warning| warning.contains("no promote-to-live button"))
    );
}
