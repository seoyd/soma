mod support;

use soma_zero::{AICommitteeMemberCoreFamily, CommitteeOwnedCoreRegistryStatus};
use support::sprint98_support::run_sprint98;

#[test]
fn member_core_contracts_keep_deferred_and_prototype_only_families_member_owned() {
    let bundle = run_sprint98(
        "soma_sprint98_committee_owned_core.toml",
        "committee-member-core-contracts",
    );
    assert!(
        bundle
            .ai_committee_member_core_contracts
            .iter()
            .any(|contract| contract.core_family == AICommitteeMemberCoreFamily::Mamba3FinDeferred)
    );
    assert!(
        bundle
            .ai_committee_member_core_contracts
            .iter()
            .any(|contract| contract.core_family
                == AICommitteeMemberCoreFamily::GatedDeltaNetDeferred)
    );
    assert!(
        bundle
            .ai_committee_member_core_contracts
            .iter()
            .any(|contract| contract.core_family
                == AICommitteeMemberCoreFamily::ExternalPredictionPrototype)
    );
    assert!(
        bundle
            .ai_committee_member_core_contracts
            .iter()
            .all(|contract| !contract.runtime_allowed)
    );
    assert!(
        bundle
            .ai_committee_member_core_contracts
            .iter()
            .all(|contract| !contract.training_allowed)
    );
    assert!(
        bundle
            .ai_committee_member_core_contracts
            .iter()
            .all(|contract| !contract.live_inference_allowed)
    );
    assert!(
        bundle
            .ai_committee_member_core_contracts
            .iter()
            .all(|contract| contract.paper_only_required)
    );
    assert_eq!(
        bundle.committee_owned_core_registry.registry_status,
        CommitteeOwnedCoreRegistryStatus::MemberCoreRegistryReady
    );
}
