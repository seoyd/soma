use soma_zero::{
    CommitteeSmokeFinalStatus, CommitteeSmokeTestConfig, CommitteeSmokeTestRunner, ReasonCode,
};

#[test]
fn smoke_runner_works_on_fixture_input() {
    let report = CommitteeSmokeTestRunner::default()
        .run(&CommitteeSmokeTestConfig {
            test_id: "committee-smoke-fixture".to_string(),
            require_core_check: false,
            reason_codes: vec![ReasonCode::CommitteeSmokeTestBuilt],
            ..CommitteeSmokeTestConfig::default()
        })
        .expect("committee smoke");
    assert_eq!(report.active_personas.len(), 3);
    assert!(!report.decisions.is_empty());
    assert!(report.source_summary.contains("fixture"));
}

#[test]
fn crypto_only_smoke_stays_crypto_only_not_research_only() {
    let report = CommitteeSmokeTestRunner::default()
        .run(&CommitteeSmokeTestConfig {
            test_id: "committee-smoke-crypto-only".to_string(),
            use_fixture_data: false,
            use_upbit_crypto_lane: true,
            use_yfinance_research_lane: false,
            require_core_check: false,
            reason_codes: vec![ReasonCode::CommitteeSmokeTestBuilt],
            ..CommitteeSmokeTestConfig::default()
        })
        .expect("committee smoke");
    assert!(report.source_summary.contains("upbit-crypto-only"));
    assert!(
        report
            .warnings
            .contains(&"Upbit committee smoke remains crypto-only".to_string())
    );
    assert_ne!(
        report.final_status,
        CommitteeSmokeFinalStatus::CommitteeResearchOnly
    );
}

#[test]
fn yfinance_smoke_remains_research_only() {
    let report = CommitteeSmokeTestRunner::default()
        .run(&CommitteeSmokeTestConfig {
            test_id: "committee-smoke-yfinance".to_string(),
            use_fixture_data: false,
            use_upbit_crypto_lane: false,
            use_yfinance_research_lane: true,
            require_core_check: false,
            reason_codes: vec![ReasonCode::CommitteeSmokeTestBuilt],
            ..CommitteeSmokeTestConfig::default()
        })
        .expect("committee smoke");
    assert_eq!(
        report.final_status,
        CommitteeSmokeFinalStatus::CommitteeResearchOnly
    );
    assert!(
        report
            .warnings
            .contains(&"yfinance remains research-only".to_string())
    );
}

#[test]
fn smoke_runner_enforces_max_decisions() {
    let report = CommitteeSmokeTestRunner::default()
        .run(&CommitteeSmokeTestConfig {
            test_id: "committee-smoke-max-decisions".to_string(),
            max_decisions: 1,
            require_core_check: false,
            reason_codes: vec![ReasonCode::CommitteeSmokeTestBuilt],
            ..CommitteeSmokeTestConfig::default()
        })
        .expect("committee smoke");
    assert_eq!(report.decisions.len(), 1);
}
