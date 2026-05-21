use soma_zero::{AuditLedger, AuditRecord, RuntimeMode, RuntimeStage};

fn record(id: &str, stage: RuntimeStage, with_reason: bool) -> AuditRecord {
    AuditRecord {
        audit_id: id.to_string(),
        mode: RuntimeMode::Research,
        stage,
        source_kind: "test".to_string(),
        input_fingerprint: "input".to_string(),
        output_fingerprint: Some("output".to_string()),
        decision_summary: Some("decision".to_string()),
        risk_decision: Some("deny".to_string()),
        reason_codes: if with_reason {
            vec![soma_zero::ReasonCode::RuntimeStateInitialized]
        } else {
            vec![]
        },
        timestamp_ms: None,
    }
}

#[test]
fn audit_ledger_add_record_works() {
    let mut ledger = AuditLedger::default();
    ledger.add_record(record("a", RuntimeStage::Init, true));

    assert_eq!(ledger.records.len(), 1);
}

#[test]
fn audit_summary_counts_stages() {
    let mut ledger = AuditLedger::default();
    ledger.add_record(record("a", RuntimeStage::Init, true));
    ledger.add_record(record("b", RuntimeStage::RiskEvaluation, true));

    let summary = ledger.summarize();

    assert_eq!(summary.total_records, 2);
    assert!(summary.stages_seen.contains(&RuntimeStage::Init));
    assert!(summary.stages_seen.contains(&RuntimeStage::RiskEvaluation));
}

#[test]
fn audit_summary_detects_missing_reason_codes_if_simulated() {
    let mut ledger = AuditLedger::default();
    ledger.add_record(record("a", RuntimeStage::Init, false));

    let summary = ledger.summarize();

    assert_eq!(summary.missing_reason_code_count, 1);
}

#[test]
fn audit_ledger_fingerprint_is_deterministic() {
    let mut left = AuditLedger::default();
    let mut right = AuditLedger::default();
    for stage in [RuntimeStage::Init, RuntimeStage::RiskEvaluation] {
        left.add_record(record(&format!("left-{stage:?}"), stage, true));
        right.add_record(record(&format!("left-{stage:?}"), stage, true));
    }

    assert_eq!(left.fingerprint(), right.fingerprint());
}

#[test]
fn audit_ledger_does_not_require_wall_clock() {
    let mut ledger = AuditLedger::default();
    ledger.add_record(record("a", RuntimeStage::Init, true));

    assert!(ledger.records[0].timestamp_ms.is_none());
}
