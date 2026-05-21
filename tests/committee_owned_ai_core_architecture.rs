mod support;

use soma_zero::CommitteeOwnedAiCoreArchitectureStatus;
use support::shared_fixture_harness::assert_deterministic_text;
use support::sprint98_support::run_sprint98;

#[test]
fn architecture_is_committee_owned_and_deterministic() {
    let first = run_sprint98(
        "soma_sprint98_committee_owned_core.toml",
        "committee-owned-ai-core-architecture",
    );
    let second = run_sprint98(
        "soma_sprint98_committee_owned_core.toml",
        "committee-owned-ai-core-architecture-second",
    );
    let report = &first.committee_owned_ai_core_architecture;
    assert!(report.central_core_deprecated);
    assert!(report.committee_owned_core_enabled);
    assert_eq!(
        report.member_core_count,
        first.ai_committee_member_core_contracts.len()
    );
    assert!(report.runtime_deferred_required);
    assert!(report.training_deferred_required);
    assert!(report.live_trading_forbidden_required);
    assert_eq!(
        report.architecture_status,
        CommitteeOwnedAiCoreArchitectureStatus::CommitteeOwnedCoreReadyWithWarnings
    );
    assert_eq!(report, &second.committee_owned_ai_core_architecture);
    assert_deterministic_text(&first.final_summary, &second.final_summary);
    assert!(
        first
            .final_summary
            .contains("## 5. Committee-owned AI core architecture")
    );
}
