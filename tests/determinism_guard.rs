use soma_zero::{
    DeterminismCheck, DeterminismInputFingerprint, DeterminismOutputFingerprint, ReasonCode,
    stable_ordered_strings, stable_reason_codes,
};

#[test]
fn same_config_data_and_report_produce_same_fingerprint() {
    let input =
        DeterminismInputFingerprint::new("fixture", "config-a", Some("data-a"), Some(11), Some(22));
    let left = DeterminismOutputFingerprint::new("report-a", 3, &[ReasonCode::MissingFile], 1, 128);
    let right =
        DeterminismOutputFingerprint::new("report-a", 3, &[ReasonCode::MissingFile], 1, 128);
    let check = DeterminismCheck::compare(input, &left, &right);

    assert!(check.deterministic);
}

#[test]
fn different_config_produces_different_input_fingerprint() {
    let left = DeterminismInputFingerprint::new("fixture", "config-a", Some("data-a"), None, None);
    let right = DeterminismInputFingerprint::new("fixture", "config-b", Some("data-a"), None, None);

    assert_ne!(left.config_fingerprint, right.config_fingerprint);
}

#[test]
fn file_ordering_is_normalized_deterministically() {
    let ordered = stable_ordered_strings(&[
        "b/report.txt".to_string(),
        "a/report.txt".to_string(),
        "c/report.txt".to_string(),
    ]);

    assert_eq!(
        ordered,
        vec![
            "a/report.txt".to_string(),
            "b/report.txt".to_string(),
            "c/report.txt".to_string()
        ]
    );
}

#[test]
fn reason_code_ordering_is_deterministic() {
    let ordered = stable_reason_codes(&[
        ReasonCode::MissingFile,
        ReasonCode::BudgetExceeded,
        ReasonCode::MissingFile,
        ReasonCode::LiveModeDisabled,
    ]);

    assert_eq!(
        ordered,
        vec![
            ReasonCode::BudgetExceeded,
            ReasonCode::LiveModeDisabled,
            ReasonCode::MissingFile,
        ]
    );
}

#[test]
fn no_wall_clock_appears_unless_explicitly_passed() {
    let input = DeterminismInputFingerprint::new("fixture", "config-a", None, None, None);
    let output = DeterminismOutputFingerprint::new("report-a", 1, &[], 0, 0);
    let check = DeterminismCheck::compare(input, &output, &output);

    assert!(!check.to_text().contains("timestamp"));
}
