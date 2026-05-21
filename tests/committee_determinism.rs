use soma_zero::{CommitteeSmokeTestConfig, CommitteeSmokeTestRunner, ReasonCode};

#[test]
fn committee_smoke_report_is_deterministic() {
    let config = CommitteeSmokeTestConfig {
        test_id: "committee-deterministic".to_string(),
        require_core_check: false,
        reason_codes: vec![ReasonCode::CommitteeSmokeTestBuilt],
        ..CommitteeSmokeTestConfig::default()
    };
    let first = CommitteeSmokeTestRunner::default()
        .run(&config)
        .expect("first run");
    let second = CommitteeSmokeTestRunner::default()
        .run(&config)
        .expect("second run");
    assert_eq!(first.to_text(), second.to_text());
    assert_eq!(
        first.to_json_string().expect("first json"),
        second.to_json_string().expect("second json")
    );
}
