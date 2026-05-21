use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, RuntimeMode, RuntimeStage, stable_hash_string, stable_reason_codes};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditRecord {
    pub audit_id: String,
    pub mode: RuntimeMode,
    pub stage: RuntimeStage,
    pub source_kind: String,
    pub input_fingerprint: String,
    #[serde(default)]
    pub output_fingerprint: Option<String>,
    #[serde(default)]
    pub decision_summary: Option<String>,
    #[serde(default)]
    pub risk_decision: Option<String>,
    pub reason_codes: Vec<ReasonCode>,
    #[serde(default)]
    pub timestamp_ms: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditLedger {
    pub records: Vec<AuditRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditSummary {
    pub total_records: usize,
    pub stages_seen: Vec<RuntimeStage>,
    pub decisions_seen: usize,
    pub risk_decisions_seen: usize,
    pub failures_seen: usize,
    pub missing_reason_code_count: usize,
    pub fingerprint: String,
    pub reason_codes: Vec<ReasonCode>,
}

impl AuditLedger {
    pub fn add_record(&mut self, record: AuditRecord) {
        self.records.push(record);
    }

    pub fn fingerprint(&self) -> String {
        let mut ordered = self.records.clone();
        ordered.sort_by(|left, right| {
            left.audit_id
                .cmp(&right.audit_id)
                .then_with(|| format!("{:?}", left.stage).cmp(&format!("{:?}", right.stage)))
                .then_with(|| left.input_fingerprint.cmp(&right.input_fingerprint))
        });
        stable_hash_string(
            &ordered
                .iter()
                .map(|record| {
                    format!(
                        "{}|{:?}|{:?}|{}|{}|{}|{}",
                        record.audit_id,
                        record.mode,
                        record.stage,
                        record.source_kind,
                        record.input_fingerprint,
                        record.output_fingerprint.clone().unwrap_or_default(),
                        record
                            .reason_codes
                            .iter()
                            .map(|code| format!("{code:?}"))
                            .collect::<Vec<_>>()
                            .join("|")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }

    pub fn validate_completeness(&self) -> usize {
        self.records
            .iter()
            .filter(|record| record.reason_codes.is_empty())
            .count()
    }

    pub fn summarize(&self) -> AuditSummary {
        let missing_reason_code_count = self.validate_completeness();
        let mut stages_seen = self
            .records
            .iter()
            .map(|record| record.stage)
            .collect::<Vec<_>>();
        stages_seen.sort();
        stages_seen.dedup();
        let mut reason_codes = vec![ReasonCode::AuditLedgerBuilt];
        if missing_reason_code_count > 0 {
            reason_codes.push(ReasonCode::AuditLedgerMissingReasons);
        }
        AuditSummary {
            total_records: self.records.len(),
            stages_seen,
            decisions_seen: self
                .records
                .iter()
                .filter(|record| record.decision_summary.is_some())
                .count(),
            risk_decisions_seen: self
                .records
                .iter()
                .filter(|record| record.risk_decision.is_some())
                .count(),
            failures_seen: self
                .records
                .iter()
                .filter(|record| record.stage == RuntimeStage::Failed)
                .count(),
            missing_reason_code_count,
            fingerprint: self.fingerprint(),
            reason_codes: stable_reason_codes(&reason_codes),
        }
    }
}

impl AuditSummary {
    pub fn to_text(&self) -> String {
        [
            format!("total_records={}", self.total_records),
            format!(
                "stages_seen={}",
                self.stages_seen
                    .iter()
                    .map(|stage| format!("{stage:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
            format!("decisions_seen={}", self.decisions_seen),
            format!("risk_decisions_seen={}", self.risk_decisions_seen),
            format!("failures_seen={}", self.failures_seen),
            format!(
                "missing_reason_code_count={}",
                self.missing_reason_code_count
            ),
            format!("fingerprint={}", self.fingerprint),
        ]
        .join("\n")
    }
}
