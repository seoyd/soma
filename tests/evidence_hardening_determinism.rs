mod common;
#[path = "support/sprint60_support.rs"]
mod sprint60_support;

use soma_zero::EvidenceHardeningRunner;

#[test]
fn evidence_hardening_runner_is_deterministic_for_same_fixture() {
    let config = sprint60_support::config_from_example(
        "soma_evidence_hardening.toml",
        "evidence-hardening-determinism",
    );
    let first = EvidenceHardeningRunner::default()
        .run(&config)
        .expect("first evidence hardening run");
    let second = EvidenceHardeningRunner::default()
        .run(&config)
        .expect("second evidence hardening run");
    assert_eq!(first, second);
}
