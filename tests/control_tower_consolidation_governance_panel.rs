mod support;

use soma_zero::ControlTowerConsolidationGovernancePanel;
use support::sprint115_support::{read_fixture, run_sprint115};

#[test]
fn control_tower_consolidation_governance_panel_matches_expected() {
    let bundle = run_sprint115(
        "soma_control_tower_consolidation_governance.toml",
        "control-tower-consolidation-governance",
    );
    let expected: ControlTowerConsolidationGovernancePanel =
        read_fixture("sprint115_data/control_tower_consolidation_governance_expected.json");
    assert_eq!(
        bundle.control_tower_consolidation_governance_panel,
        expected
    );
    assert!(
        bundle
            .control_tower_consolidation_governance_panel
            .static_read_only
    );
    assert!(
        bundle
            .control_tower_consolidation_governance_panel
            .no_apply_button
    );
}
