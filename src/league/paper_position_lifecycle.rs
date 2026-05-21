use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};
use crate::ui::{PaperPositionPanel, PaperPositionSide, PaperPositionStatus, PaperPositionView};

use super::candidate_generation::{CandidateEvidenceClass, GeneratedCandidate};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperPositionLifecycleEvent {
    #[default]
    PaperOpen,
    PaperUpdate,
    PaperTargetHit,
    PaperStopHit,
    PaperExpired,
    PaperRiskClosed,
    PaperManualReviewed,
    PaperClosed,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PaperPositionLifecycleReport {
    #[serde(default)]
    pub open_positions: Vec<PaperPositionView>,
    #[serde(default)]
    pub closed_positions: Vec<PaperPositionView>,
    pub target_hit_count: usize,
    pub stop_hit_count: usize,
    pub expired_count: usize,
    pub risk_closed_count: usize,
    #[serde(default)]
    pub average_unrealized_return: Option<f64>,
    #[serde(default)]
    pub average_realized_return: Option<f64>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
    pub fingerprint: String,
}

impl PaperPositionLifecycleReport {
    pub fn stabilize(&mut self) {
        self.open_positions
            .sort_by(|left, right| left.paper_position_id.cmp(&right.paper_position_id));
        self.closed_positions
            .sort_by(|left, right| left.paper_position_id.cmp(&right.paper_position_id));
        for position in self
            .open_positions
            .iter_mut()
            .chain(self.closed_positions.iter_mut())
        {
            position.reason_codes = stable_reason_codes(&position.reason_codes);
        }
        self.reason_codes = stable_reason_codes(&self.reason_codes);
        self.fingerprint = String::new();
        self.fingerprint = stable_hash_string(&serde_json::to_string(self).unwrap_or_default());
    }

    pub fn to_text(&self) -> String {
        [
            "simulated_only_warning=paper lifecycle report is simulated only with no broker account or order ids"
                .to_string(),
            format!("open_positions={}", self.open_positions.len()),
            format!("closed_positions={}", self.closed_positions.len()),
            format!("target_hit_count={}", self.target_hit_count),
            format!("stop_hit_count={}", self.stop_hit_count),
            format!("expired_count={}", self.expired_count),
            format!("risk_closed_count={}", self.risk_closed_count),
            format!(
                "average_unrealized_return={:.6}",
                self.average_unrealized_return.unwrap_or_default()
            ),
            format!(
                "average_realized_return={:.6}",
                self.average_realized_return.unwrap_or_default()
            ),
            format!("fingerprint={}", self.fingerprint),
        ]
        .join("\n")
    }

    pub fn to_panel(&self) -> PaperPositionPanel {
        let mut panel = PaperPositionPanel {
            open_positions: self.open_positions.clone(),
            closed_positions: self.closed_positions.clone(),
            risk_closed_positions: self
                .closed_positions
                .iter()
                .filter(|position| matches!(position.status, PaperPositionStatus::RiskClosed))
                .cloned()
                .collect(),
            diagnostic_positions: self
                .open_positions
                .iter()
                .chain(self.closed_positions.iter())
                .filter(|position| matches!(position.status, PaperPositionStatus::DiagnosticOnly))
                .cloned()
                .collect(),
            reason_codes: self.reason_codes.clone(),
        };
        panel.stabilize();
        panel
    }
}

pub fn open_paper_position(candidate: &GeneratedCandidate) -> PaperPositionView {
    let entry_price = 100.0;
    let stop_price = entry_price * (1.0 - candidate.expected_drawdown.unwrap_or(0.02).max(0.005));
    let target_price = entry_price * (1.0 + candidate.expected_edge.unwrap_or(0.01).max(0.01));
    let status = if matches!(
        candidate.evidence_class,
        CandidateEvidenceClass::DiagnosticOnly
    ) {
        PaperPositionStatus::DiagnosticOnly
    } else {
        PaperPositionStatus::Open
    };
    PaperPositionView {
        paper_position_id: stable_hash_string(&format!("paper:{}", candidate.candidate_id)),
        candidate_id: candidate.candidate_id.clone(),
        symbol: candidate.symbol.clone(),
        market: format!("{:?}", candidate.market),
        side: PaperPositionSide::Long,
        entry_timestamp_ms: Some(candidate.timestamp_ms),
        entry_price: Some(entry_price),
        stop_price: Some(stop_price),
        target_price: Some(target_price),
        current_price: Some(entry_price),
        unrealized_return_pct: Some(0.0),
        realized_return_pct: None,
        status,
        source_kind: format!("{:?}", candidate.source_kind),
        reason_codes: stable_reason_codes(&[ReasonCode::PaperExecutionOnly]),
    }
}

pub fn build_paper_position_lifecycle_report(
    positions: &[PaperPositionView],
) -> PaperPositionLifecycleReport {
    let mut open_positions = Vec::new();
    let mut closed_positions = Vec::new();
    let mut target_hit_count = 0;
    let mut stop_hit_count = 0;
    let mut expired_count = 0;
    let mut risk_closed_count = 0;

    for position in positions.iter().cloned() {
        let classified = classify_position(position);
        match classified.status {
            PaperPositionStatus::Open | PaperPositionStatus::DiagnosticOnly => {
                open_positions.push(classified)
            }
            PaperPositionStatus::TargetHit => {
                target_hit_count += 1;
                closed_positions.push(classified);
            }
            PaperPositionStatus::Stopped => {
                stop_hit_count += 1;
                closed_positions.push(classified);
            }
            PaperPositionStatus::Expired => {
                expired_count += 1;
                closed_positions.push(classified);
            }
            PaperPositionStatus::RiskClosed => {
                risk_closed_count += 1;
                closed_positions.push(classified);
            }
            PaperPositionStatus::Closed => closed_positions.push(classified),
        }
    }

    let average_unrealized_return = average_of(
        open_positions
            .iter()
            .filter_map(|position| position.unrealized_return_pct)
            .collect(),
    );
    let average_realized_return = average_of(
        closed_positions
            .iter()
            .filter_map(|position| position.realized_return_pct)
            .collect(),
    );

    let mut report = PaperPositionLifecycleReport {
        open_positions,
        closed_positions,
        target_hit_count,
        stop_hit_count,
        expired_count,
        risk_closed_count,
        average_unrealized_return,
        average_realized_return,
        reason_codes: vec![
            ReasonCode::PaperExecutionOnly,
            ReasonCode::DeterministicPath,
        ],
        fingerprint: String::new(),
    };
    report.stabilize();
    report
}

fn classify_position(mut position: PaperPositionView) -> PaperPositionView {
    let entry = position.entry_price.unwrap_or(100.0);
    let current = position.current_price.unwrap_or(entry);
    let unrealized = if entry > 0.0 {
        (current - entry) / entry
    } else {
        0.0
    };
    position.unrealized_return_pct = Some(unrealized);
    match position.status {
        PaperPositionStatus::Open => {
            if let Some(target) = position.target_price {
                if current >= target {
                    position.status = PaperPositionStatus::TargetHit;
                    position.realized_return_pct = Some(unrealized);
                    return position;
                }
            }
            if let Some(stop) = position.stop_price {
                if current <= stop {
                    position.status = PaperPositionStatus::Stopped;
                    position.realized_return_pct = Some(unrealized);
                    return position;
                }
            }
            if position
                .reason_codes
                .iter()
                .any(|reason| matches!(reason, ReasonCode::TimeBarrierExpired))
            {
                position.status = PaperPositionStatus::Expired;
                position.realized_return_pct = Some(unrealized);
                return position;
            }
            if position
                .reason_codes
                .iter()
                .any(|reason| matches!(reason, ReasonCode::RiskDenied))
            {
                position.status = PaperPositionStatus::RiskClosed;
                position.realized_return_pct = Some(unrealized);
                return position;
            }
        }
        PaperPositionStatus::Closed
        | PaperPositionStatus::Stopped
        | PaperPositionStatus::TargetHit
        | PaperPositionStatus::Expired
        | PaperPositionStatus::RiskClosed
        | PaperPositionStatus::DiagnosticOnly => {
            if position.realized_return_pct.is_none() {
                position.realized_return_pct = Some(unrealized);
            }
        }
    }
    position
}

fn average_of(values: Vec<f64>) -> Option<f64> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f64>() / values.len() as f64)
    }
}
