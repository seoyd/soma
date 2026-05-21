#[path = "support/sprint48_support.rs"]
mod support;

use soma_zero::{CommitteeTripleBarrierLabel, OutcomeDiversityAuditRunner, OutcomeDiversityStatus};

#[test]
fn all_take_profit_outcomes_produce_single_outcome_dominated() {
    let config = support::outcome_audit_config("audit-all-tp");
    let report = OutcomeDiversityAuditRunner::default()
        .run(&config)
        .expect("audit report");

    assert_eq!(
        report.outcome_diversity_status,
        OutcomeDiversityStatus::SingleOutcomeDominated
    );
}

#[test]
fn mixed_take_profit_stop_loss_time_expired_improves_diversity() {
    let config = soma_zero::OutcomeDiversityAuditConfig {
        audit_id: "audit-mixed".to_string(),
        min_total_outcomes: 4,
        max_single_outcome_label_ratio: 0.8,
        ..soma_zero::OutcomeDiversityAuditConfig::default()
    };
    let report = OutcomeDiversityAuditRunner::default().run_from_inputs(
        &config,
        Some(&support::mixed_outcomes()),
        Some(&support::mixed_counterfactuals()),
        Some(&support::mixed_set()),
    );

    assert_eq!(
        report.outcome_diversity_status,
        OutcomeDiversityStatus::HealthyOutcomeDiversity
    );
}

#[test]
fn outcome_entropy_is_computed_deterministically() {
    let config = soma_zero::OutcomeDiversityAuditConfig {
        audit_id: "audit-entropy".to_string(),
        min_total_outcomes: 4,
        ..soma_zero::OutcomeDiversityAuditConfig::default()
    };
    let report = OutcomeDiversityAuditRunner::default().run_from_inputs(
        &config,
        Some(&support::mixed_outcomes()),
        Some(&support::mixed_counterfactuals()),
        Some(&support::mixed_set()),
    );

    assert!((report.outcome_entropy - 1.5).abs() < 1e-9);
}

#[test]
fn missing_stop_loss_status_works() {
    let config = soma_zero::OutcomeDiversityAuditConfig {
        audit_id: "audit-missing-stop-loss".to_string(),
        min_total_outcomes: 3,
        max_single_outcome_label_ratio: 0.8,
        ..soma_zero::OutcomeDiversityAuditConfig::default()
    };
    let mut outcomes = support::mixed_outcomes();
    outcomes.records.retain(|record| record.row_id != "msft-sl");

    let report = OutcomeDiversityAuditRunner::default().run_from_inputs(
        &config,
        Some(&outcomes),
        Some(&support::mixed_counterfactuals()),
        Some(&support::mixed_set()),
    );

    assert_eq!(
        report.outcome_diversity_status,
        OutcomeDiversityStatus::MissingStopLoss
    );
}

#[test]
fn missing_time_expired_status_works() {
    let config = soma_zero::OutcomeDiversityAuditConfig {
        audit_id: "audit-missing-time-expired".to_string(),
        min_total_outcomes: 3,
        max_single_outcome_label_ratio: 0.8,
        ..soma_zero::OutcomeDiversityAuditConfig::default()
    };
    let mut outcomes = support::mixed_outcomes();
    for record in &mut outcomes.records {
        if record.row_id == "nvda-te" {
            record
                .outcome_reference
                .as_mut()
                .expect("outcome ref")
                .triple_barrier_label = CommitteeTripleBarrierLabel::StopLoss;
        }
    }

    let report = OutcomeDiversityAuditRunner::default().run_from_inputs(
        &config,
        Some(&outcomes),
        Some(&support::mixed_counterfactuals()),
        Some(&support::mixed_set()),
    );

    assert_eq!(
        report.outcome_diversity_status,
        OutcomeDiversityStatus::MissingTimeExpired
    );
}

#[test]
fn too_few_outcomes_status_works() {
    let config = soma_zero::OutcomeDiversityAuditConfig {
        audit_id: "audit-too-few".to_string(),
        min_total_outcomes: 2,
        ..soma_zero::OutcomeDiversityAuditConfig::default()
    };
    let report = OutcomeDiversityAuditRunner::default().run_from_inputs(
        &config,
        Some(&support::single_outcomes()),
        Some(&support::single_counterfactuals()),
        Some(&support::single_set()),
    );

    assert_eq!(
        report.outcome_diversity_status,
        OutcomeDiversityStatus::TooFewOutcomes
    );
}

#[test]
fn audit_is_deterministic() {
    let config = support::outcome_audit_config("audit-deterministic");

    let first = OutcomeDiversityAuditRunner::default()
        .run(&config)
        .expect("first audit");
    let second = OutcomeDiversityAuditRunner::default()
        .run(&config)
        .expect("second audit");

    assert_eq!(first, second);
    assert_eq!(first.to_text(), second.to_text());
    assert_eq!(first.fingerprint(), second.fingerprint());
}
