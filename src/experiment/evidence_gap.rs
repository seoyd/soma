use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::decision_router::Sprint14EvidenceInput;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvidenceChecklistItem {
    pub label: String,
    pub current: usize,
    pub required: usize,
    pub satisfied: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MinimumEvidencePlan {
    pub required_usable_datasets: usize,
    pub required_total_outcome_records: usize,
    pub required_comparable_variants: usize,
    pub additional_usable_datasets_needed: usize,
    pub additional_outcome_records_needed: usize,
    pub additional_comparable_variants_needed: usize,
    pub recommended_dataset_placeholders: Vec<String>,
    pub recommended_matrix_expansions: Vec<String>,
    pub blocked_expansion: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvidenceGapReport {
    pub study_id: String,
    pub insufficient_evidence: bool,
    pub checklist: Vec<EvidenceChecklistItem>,
    pub minimum_evidence_plan: MinimumEvidencePlan,
    pub warnings: Vec<String>,
    pub blockers: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_evidence_gap_report(input: &Sprint14EvidenceInput) -> EvidenceGapReport {
    let study_id = input
        .source_study_id
        .clone()
        .unwrap_or_else(|| "unknown-study".to_string());
    let usable_dataset_count = input.usable_dataset_count.unwrap_or(0);
    let total_outcome_records = input.total_outcome_records.unwrap_or(0);
    let comparable_variant_count = input.comparable_variant_count.unwrap_or(0);
    let required_usable_datasets = 3usize;
    let required_total_outcome_records = 20usize;
    let required_comparable_variants = 2usize;
    let checklist = vec![
        EvidenceChecklistItem {
            label: "usable_datasets".to_string(),
            current: usable_dataset_count,
            required: required_usable_datasets,
            satisfied: usable_dataset_count >= required_usable_datasets,
        },
        EvidenceChecklistItem {
            label: "total_outcome_records".to_string(),
            current: total_outcome_records,
            required: required_total_outcome_records,
            satisfied: total_outcome_records >= required_total_outcome_records,
        },
        EvidenceChecklistItem {
            label: "comparable_variants".to_string(),
            current: comparable_variant_count,
            required: required_comparable_variants,
            satisfied: comparable_variant_count >= required_comparable_variants,
        },
    ];
    let insufficient_evidence = checklist.iter().any(|item| !item.satisfied);
    let plan = MinimumEvidencePlan {
        required_usable_datasets,
        required_total_outcome_records,
        required_comparable_variants,
        additional_usable_datasets_needed: required_usable_datasets
            .saturating_sub(usable_dataset_count),
        additional_outcome_records_needed: required_total_outcome_records
            .saturating_sub(total_outcome_records),
        additional_comparable_variants_needed: required_comparable_variants
            .saturating_sub(comparable_variant_count),
        recommended_dataset_placeholders: vec![
            format!("{study_id}-local-trend-fixture"),
            format!("{study_id}-local-range-fixture"),
            format!("{study_id}-local-high-vol-fixture"),
        ],
        recommended_matrix_expansions: vec![
            "add-valid-fixture-batch".to_string(),
            "add-regime-coverage-batch".to_string(),
            "rerun-ablation-after-coverage".to_string(),
        ],
        blocked_expansion: insufficient_evidence,
        reason_codes: vec![
            ReasonCode::EvidenceGapDetected,
            ReasonCode::MinimumEvidencePlanBuilt,
        ],
    };
    EvidenceGapReport {
        study_id,
        insufficient_evidence,
        checklist,
        minimum_evidence_plan: plan,
        warnings: input.warnings.clone(),
        blockers: input.blockers.clone(),
        reason_codes: vec![ReasonCode::EvidenceGapDetected],
    }
}
