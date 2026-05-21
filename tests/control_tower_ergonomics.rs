mod common;
#[path = "support/sprint60_support.rs"]
mod sprint60_support;

use soma_zero::{ControlTowerErgonomicsStatus, EvidenceHardeningRunner, EvidenceWarningBadge};

#[test]
fn control_tower_ergonomics_generates_badges_cards_and_copy_blocks() {
    let config = sprint60_support::config_from_example(
        "soma_evidence_hardening.toml",
        "control-tower-ergonomics",
    );
    let report = EvidenceHardeningRunner::default()
        .run(&config)
        .expect("run control tower ergonomics")
        .control_tower_ergonomics_v1_5_report;
    assert_eq!(report.render_status, ControlTowerErgonomicsStatus::Ready);
    assert!(!report.candidate_cards.is_empty());
    assert!(!report.copyable_commands.is_empty());
    assert!(report.no_execution_buttons);
    assert!(report.no_account_controls);
    assert!(
        report
            .evidence_badges
            .contains(&EvidenceWarningBadge::NeedMoreKISEvidence)
    );
}
