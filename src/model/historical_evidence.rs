//! Evidence-first orchestration for real historical momentum evaluation.
//!
//! This module orchestrates existing normalized snapshots and the existing
//! read-only broker. Provider-specific parsing, active voting, and execution
//! remain outside it.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use serde::{Deserialize, Serialize};

use crate::{
    core::{ReasonCode, stable_hash_string},
    data::{
        AcquisitionMarketScope, AcquisitionMode, AgentDataIntent, ConfiguredUniverse,
        DataAcquisitionBroker, DataLookback, DataPriority, DataSnapshot, DatasetKind,
        ProspectiveBlindAcquisitionReceiptV0, ProviderCapabilities, ReadOnlyMarketDataProvider,
        ReadOnlyProviderRegistry, SnapshotSourceType, build_acquisition_plan,
        historical_replay_dataset_digest_v0, snapshot_id_from_semantic_digest_v1,
        verify_prospective_blind_acquisition_receipt_v0,
    },
    league::{AgentKind, HistoricalReplayDataset},
};

use super::{
    ConstantProbabilityBaselineV0, EarliestTemporalShiftStageV0, FrozenMamba3EncoderV0,
    LinearMomentumBaselineV0, MambaRepresentationValueStatusV0, ModelAgentDeploymentStatus,
    ModelDriftStatusV0, MomentumForensicCandidateV0, MomentumLearningCampaignConfigV0,
    MomentumLearningCampaignResultV0, MomentumLearningCampaignStatusV0, MomentumSupportTraceV0,
    MomentumTemporalDiagnosticReportV0, ProbabilityCollapseRootCauseV0, ShadowSupportDecisionV0,
    SupportEnvelopeConstructionStatusV0, SupportGateApplicabilityStatusV0,
    SupportGatedMomentumSeriesVerdictV0, WarmStartLockInStatusV0,
    build_momentum_learning_windows_v0, build_momentum_temporal_diagnostic_report_v0,
    momentum_support_traces_v0, run_momentum_learning_campaign_v0,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoricalEvidenceClassV0 {
    RealHistorical,
    SyntheticTestOnly,
    Rejected,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoricalSnapshotStatusV0 {
    Ready,
    InsufficientRows,
    InvalidChronology,
    InvalidDigest,
    Mutable,
    Unsafe,
    SyntheticTestOnly,
    UnsupportedDataset,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoricalEvidencePolicyV0 {
    pub minimum_rows_per_series: usize,
    pub minimum_real_series: usize,
    /// Local replay data is real only when an owner explicitly marks its immutable ID.
    pub owner_sanitized_snapshot_ids: BTreeSet<String>,
}

impl Default for HistoricalEvidencePolicyV0 {
    fn default() -> Self {
        Self {
            minimum_rows_per_series: 128,
            minimum_real_series: 3,
            owner_sanitized_snapshot_ids: BTreeSet::new(),
        }
    }
}

impl HistoricalEvidencePolicyV0 {
    pub fn validate(&self) -> Result<(), HistoricalEvidenceErrorV0> {
        if self.minimum_rows_per_series == 0 || self.minimum_real_series == 0 {
            Err(HistoricalEvidenceErrorV0::InvalidConfig)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoricalSeriesInventoryV0 {
    pub series_id: String,
    pub symbol: String,
    pub market: AcquisitionMarketScope,
    pub snapshot_ids: Vec<String>,
    pub row_count: usize,
    pub first_timestamp_ms: Option<u64>,
    pub last_timestamp_ms: Option<u64>,
    pub provider_ids: Vec<String>,
    pub source_types: Vec<SnapshotSourceType>,
    pub immutable: bool,
    pub sanitized: bool,
    pub digest_valid: bool,
    pub evidence_class: HistoricalEvidenceClassV0,
    pub status: HistoricalSnapshotStatusV0,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RejectedSnapshotEvidenceV0 {
    pub snapshot_id: String,
    pub status: HistoricalSnapshotStatusV0,
    pub evidence_class: HistoricalEvidenceClassV0,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoricalSnapshotInventoryV0 {
    pub total_snapshots: usize,
    pub real_historical_snapshots: usize,
    pub synthetic_snapshots: usize,
    pub accepted_series: Vec<HistoricalSeriesInventoryV0>,
    pub rejected_snapshots: Vec<RejectedSnapshotEvidenceV0>,
    pub markets_present: Vec<AcquisitionMarketScope>,
    pub reason_codes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MomentumHistoricalSeriesV0 {
    pub series_id: String,
    pub symbol: String,
    pub market: AcquisitionMarketScope,
    pub snapshot_ids: Vec<String>,
    pub snapshots: Vec<DataSnapshot>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MomentumHistoricalEvidencePackV0 {
    pub pack_id: String,
    pub created_from_snapshot_ids: Vec<String>,
    pub series: Vec<MomentumHistoricalSeriesV0>,
    pub real_series_count: usize,
    pub synthetic_series_count: usize,
    pub rejected_series: Vec<HistoricalSeriesInventoryV0>,
    pub frozen: bool,
    pub digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoricalProviderGateStatusV0 {
    ExistingEvidenceSufficient,
    ApprovedProviderSelected,
    NoApprovedHistoricalProviderConfigured,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoricalHarvestConfigV0 {
    pub markets: Vec<AcquisitionMarketScope>,
    pub configured_universe: ConfiguredUniverse,
    pub minimum_rows_per_series: usize,
    pub maximum_rows_per_series: usize,
    pub provider_preference: Vec<String>,
    pub start_timestamp_ms: Option<u64>,
    pub end_timestamp_ms: Option<u64>,
    pub max_staleness_ms: u64,
    pub allow_partial_harvest: bool,
}

impl HistoricalHarvestConfigV0 {
    pub fn validate(&self) -> Result<(), HistoricalEvidenceErrorV0> {
        if self.markets.is_empty()
            || self.minimum_rows_per_series == 0
            || self.maximum_rows_per_series < self.minimum_rows_per_series
            || self
                .markets
                .iter()
                .any(|market| *market == AcquisitionMarketScope::Unknown)
            || self
                .start_timestamp_ms
                .zip(self.end_timestamp_ms)
                .is_some_and(|(start, end)| start >= end)
        {
            return Err(HistoricalEvidenceErrorV0::InvalidConfig);
        }
        if self
            .markets
            .iter()
            .all(|market| self.configured_universe.symbols_for(*market).is_empty())
        {
            return Err(HistoricalEvidenceErrorV0::InvalidConfig);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoricalProviderGateResultV0 {
    pub status: HistoricalProviderGateStatusV0,
    pub selected_provider_id: Option<String>,
    pub rejected_provider_ids: Vec<String>,
    pub reason_codes: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoricalAcquisitionStatusV0 {
    ExistingSnapshotsUsed,
    SnapshotsAcquired,
    NoApprovedHistoricalProviderConfigured,
    HistoricalAcquisitionFailed,
    InsufficientHistoricalEvidence,
    NoRealHistoricalEvidence,
    NotAttempted,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HistoricalHarvestResultV0 {
    pub status: HistoricalAcquisitionStatusV0,
    pub provider_gate: HistoricalProviderGateResultV0,
    pub acquired_snapshots: Vec<DataSnapshot>,
    pub reused_snapshots: Vec<DataSnapshot>,
    pub failure_count: usize,
    pub reason_codes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MomentumSeriesCampaignResultV0 {
    pub series_id: String,
    pub symbol: String,
    pub market: AcquisitionMarketScope,
    pub snapshot_ids: Vec<String>,
    pub campaign: MomentumLearningCampaignResultV0,
    pub evidence_status: HistoricalSnapshotStatusV0,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrossSeriesMomentumVerdictV0 {
    MambaHelped,
    LinearBaselineStronger,
    Mixed,
    DriftRisk,
    InsufficientEvidence,
    NoHistoricalEvidence,
    NoApprovedProvider,
    RejectedForSafety,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CrossSeriesMomentumGateConfigV0 {
    pub minimum_real_series: usize,
    pub minimum_sufficient_series: usize,
    pub minimum_evaluated_windows: usize,
    pub minimum_mamba_win_fraction: f32,
    pub brier_improvement_epsilon: f32,
    pub maximum_drifted_series_fraction: f32,
}

impl Default for CrossSeriesMomentumGateConfigV0 {
    fn default() -> Self {
        Self {
            minimum_real_series: 3,
            minimum_sufficient_series: 3,
            minimum_evaluated_windows: 6,
            minimum_mamba_win_fraction: 0.6,
            brier_improvement_epsilon: 1e-4,
            maximum_drifted_series_fraction: 0.34,
        }
    }
}

impl CrossSeriesMomentumGateConfigV0 {
    pub fn validate(&self) -> Result<(), HistoricalEvidenceErrorV0> {
        if self.minimum_real_series == 0
            || self.minimum_sufficient_series == 0
            || self.minimum_evaluated_windows == 0
            || !self.minimum_mamba_win_fraction.is_finite()
            || !(0.0..=1.0).contains(&self.minimum_mamba_win_fraction)
            || !self.brier_improvement_epsilon.is_finite()
            || self.brier_improvement_epsilon < 0.0
            || !self.maximum_drifted_series_fraction.is_finite()
            || !(0.0..=1.0).contains(&self.maximum_drifted_series_fraction)
        {
            Err(HistoricalEvidenceErrorV0::InvalidConfig)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CrossSeriesMomentumEvidenceV0 {
    pub evaluated_series: usize,
    pub sufficient_series: usize,
    pub total_evaluated_windows: usize,
    pub mamba_beats_constant_windows: usize,
    pub mamba_beats_linear_series: usize,
    pub linear_beats_mamba_series: usize,
    pub mixed_series: usize,
    pub insufficient_series: usize,
    pub warm_beats_cold_series: usize,
    pub cold_beats_warm_series: usize,
    pub drifted_series: usize,
    pub mean_brier_delta_vs_linear: f32,
    pub median_brier_delta_vs_linear: f32,
    pub high_confidence_error_delta_vs_linear: i64,
    pub status: CrossSeriesMomentumVerdictV0,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CrossSeriesWarmStartVerdictV0 {
    Helped,
    Failed,
    Mixed,
    InsufficientEvidence,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CrossSeriesWarmStartEvidenceV0 {
    pub compared_series: usize,
    pub warm_beats_cold_series: usize,
    pub cold_beats_warm_series: usize,
    pub tie_series: usize,
    pub status: CrossSeriesWarmStartVerdictV0,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MomentumHistoricalEvidenceReportV0 {
    pub acquisition_status: HistoricalAcquisitionStatusV0,
    pub provider_status: HistoricalProviderGateStatusV0,
    pub lines: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MomentumCampaignSufficiencyV0 {
    pub available_rows: usize,
    pub required_minimum_rows: usize,
    pub possible_windows: usize,
    pub required_windows: usize,
    pub sufficient: bool,
    pub limiting_reasons: Vec<ReasonCode>,
}

pub fn assess_momentum_campaign_sufficiency_v0(
    available_rows: usize,
    config: &MomentumLearningCampaignConfigV0,
) -> Result<MomentumCampaignSufficiencyV0, HistoricalEvidenceErrorV0> {
    config
        .validate()
        .map_err(|_| HistoricalEvidenceErrorV0::CampaignConfigurationRejected)?;
    let first_window_rows = config
        .train_rows
        .checked_add(config.purge_gap_rows)
        .and_then(|value| value.checked_add(config.validation_rows))
        .and_then(|value| value.checked_add(config.purge_gap_rows))
        .and_then(|value| value.checked_add(config.test_rows))
        .ok_or(HistoricalEvidenceErrorV0::InvalidConfig)?;
    let required_minimum_rows = config.minimum_history_rows.max(
        first_window_rows.saturating_add(
            config
                .minimum_evaluated_windows
                .saturating_sub(1)
                .saturating_mul(config.step_rows),
        ),
    );
    let possible_windows = if available_rows < config.minimum_history_rows {
        0
    } else {
        build_momentum_learning_windows_v0(config, available_rows, &["sufficiency".to_string()])
            .map(|windows| windows.len())
            .unwrap_or_default()
    };
    let sufficient = available_rows >= required_minimum_rows
        && possible_windows >= config.minimum_evaluated_windows;
    let mut limiting_reasons = Vec::new();
    if available_rows < required_minimum_rows {
        limiting_reasons.push(ReasonCode::WalkForwardInsufficientRows);
    }
    if possible_windows < config.minimum_evaluated_windows {
        limiting_reasons.push(ReasonCode::PredictionScoringInsufficientSamples);
    }
    Ok(MomentumCampaignSufficiencyV0 {
        available_rows,
        required_minimum_rows,
        possible_windows,
        required_windows: config.minimum_evaluated_windows,
        sufficient,
        limiting_reasons,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoricalEvidenceErrorV0 {
    InvalidConfig,
    CampaignConfigurationRejected,
    PackNotFrozen,
    InvalidPackDigest,
}

pub fn inventory_historical_snapshots_v0(
    snapshots: &[DataSnapshot],
    policy: &HistoricalEvidencePolicyV0,
) -> Result<HistoricalSnapshotInventoryV0, HistoricalEvidenceErrorV0> {
    policy.validate()?;
    let mut accepted = BTreeMap::<(AcquisitionMarketScope, String), Vec<DataSnapshot>>::new();
    let mut rejected = Vec::new();
    let mut real_count = 0;
    let mut synthetic_count = 0;
    for snapshot in snapshots {
        let (class, status) = classify_snapshot(snapshot, policy);
        match class {
            HistoricalEvidenceClassV0::RealHistorical
                if status == HistoricalSnapshotStatusV0::Ready =>
            {
                real_count += 1;
                accepted
                    .entry((
                        snapshot.market_scope,
                        snapshot.normalized_dataset.symbol.clone(),
                    ))
                    .or_default()
                    .push(snapshot.clone());
            }
            HistoricalEvidenceClassV0::SyntheticTestOnly => {
                synthetic_count += 1;
                rejected.push(RejectedSnapshotEvidenceV0 {
                    snapshot_id: snapshot.snapshot_id.clone(),
                    status,
                    evidence_class: class,
                });
            }
            _ => rejected.push(RejectedSnapshotEvidenceV0 {
                snapshot_id: snapshot.snapshot_id.clone(),
                status,
                evidence_class: class,
            }),
        }
    }
    let mut accepted_series = Vec::new();
    let mut rejected_series = Vec::new();
    for ((market, symbol), mut series_snapshots) in accepted {
        series_snapshots.sort_by_key(|snapshot| {
            (
                snapshot.actual_start_timestamp_ms.unwrap_or(u64::MAX),
                snapshot.snapshot_id.clone(),
            )
        });
        let row_count = series_snapshots
            .iter()
            .map(|snapshot| snapshot.row_count)
            .sum();
        let first_timestamp_ms = series_snapshots
            .iter()
            .filter_map(|snapshot| snapshot.actual_start_timestamp_ms)
            .min();
        let last_timestamp_ms = series_snapshots
            .iter()
            .filter_map(|snapshot| snapshot.actual_end_timestamp_ms)
            .max();
        let chronology_valid = series_snapshots
            .windows(2)
            .all(|pair| pair[0].actual_end_timestamp_ms < pair[1].actual_start_timestamp_ms);
        let status = if !chronology_valid {
            HistoricalSnapshotStatusV0::InvalidChronology
        } else if row_count < policy.minimum_rows_per_series {
            HistoricalSnapshotStatusV0::InsufficientRows
        } else {
            HistoricalSnapshotStatusV0::Ready
        };
        let series = HistoricalSeriesInventoryV0 {
            series_id: series_id(market, &symbol, &series_snapshots),
            symbol,
            market,
            snapshot_ids: series_snapshots
                .iter()
                .map(|snapshot| snapshot.snapshot_id.clone())
                .collect(),
            row_count,
            first_timestamp_ms,
            last_timestamp_ms,
            provider_ids: sorted_unique(
                series_snapshots
                    .iter()
                    .map(|snapshot| snapshot.provider_id.clone()),
            ),
            source_types: unique_source_types(
                series_snapshots
                    .iter()
                    .map(|snapshot| snapshot.provenance.source_type),
            ),
            immutable: true,
            sanitized: true,
            digest_valid: true,
            evidence_class: HistoricalEvidenceClassV0::RealHistorical,
            status,
        };
        if status == HistoricalSnapshotStatusV0::Ready {
            accepted_series.push(series);
        } else {
            rejected_series.push(series);
        }
    }
    accepted_series
        .sort_by(|left, right| (left.market, &left.symbol).cmp(&(right.market, &right.symbol)));
    for series in rejected_series {
        for snapshot_id in series.snapshot_ids {
            rejected.push(RejectedSnapshotEvidenceV0 {
                snapshot_id,
                status: series.status,
                evidence_class: HistoricalEvidenceClassV0::Rejected,
            });
        }
    }
    let markets_present = sorted_unique_market(accepted_series.iter().map(|series| series.market));
    Ok(HistoricalSnapshotInventoryV0 {
        total_snapshots: snapshots.len(),
        real_historical_snapshots: real_count,
        synthetic_snapshots: synthetic_count,
        accepted_series,
        rejected_snapshots: rejected,
        markets_present,
        reason_codes: vec!["historical_snapshot_inventory_built".to_string()],
    })
}

pub fn freeze_momentum_historical_evidence_pack_v0(
    snapshots: &[DataSnapshot],
    policy: &HistoricalEvidencePolicyV0,
) -> Result<
    (
        HistoricalSnapshotInventoryV0,
        MomentumHistoricalEvidencePackV0,
    ),
    HistoricalEvidenceErrorV0,
> {
    let inventory = inventory_historical_snapshots_v0(snapshots, policy)?;
    let by_id = snapshots
        .iter()
        .map(|snapshot| (snapshot.snapshot_id.clone(), snapshot.clone()))
        .collect::<BTreeMap<_, _>>();
    let series = inventory
        .accepted_series
        .iter()
        .map(|entry| MomentumHistoricalSeriesV0 {
            series_id: entry.series_id.clone(),
            symbol: entry.symbol.clone(),
            market: entry.market,
            snapshot_ids: entry.snapshot_ids.clone(),
            snapshots: entry
                .snapshot_ids
                .iter()
                .filter_map(|id| by_id.get(id).cloned())
                .collect(),
        })
        .collect::<Vec<_>>();
    let mut pack = MomentumHistoricalEvidencePackV0 {
        pack_id: format!(
            "momentum-evidence-{}",
            stable_hash_string(&pack_material(&series))
        ),
        created_from_snapshot_ids: sorted_unique(
            series.iter().flat_map(|series| series.snapshot_ids.clone()),
        ),
        real_series_count: series.len(),
        synthetic_series_count: inventory.synthetic_snapshots,
        rejected_series: inventory
            .rejected_snapshots
            .iter()
            .map(|rejected| HistoricalSeriesInventoryV0 {
                series_id: format!("rejected-{}", rejected.snapshot_id),
                symbol: String::new(),
                market: AcquisitionMarketScope::Unknown,
                snapshot_ids: vec![rejected.snapshot_id.clone()],
                row_count: 0,
                first_timestamp_ms: None,
                last_timestamp_ms: None,
                provider_ids: vec![],
                source_types: vec![],
                immutable: false,
                sanitized: false,
                digest_valid: rejected.status != HistoricalSnapshotStatusV0::InvalidDigest,
                evidence_class: rejected.evidence_class,
                status: rejected.status,
            })
            .collect(),
        frozen: true,
        digest: String::new(),
        series,
    };
    pack.digest = stable_hash_string(&pack_material(&pack.series));
    Ok((inventory, pack))
}

pub fn verify_momentum_historical_evidence_pack_v0(
    pack: &MomentumHistoricalEvidencePackV0,
) -> Result<(), HistoricalEvidenceErrorV0> {
    if !pack.frozen {
        return Err(HistoricalEvidenceErrorV0::PackNotFrozen);
    }
    if pack.digest != stable_hash_string(&pack_material(&pack.series)) {
        return Err(HistoricalEvidenceErrorV0::InvalidPackDigest);
    }
    Ok(())
}

pub fn select_approved_historical_provider_v0(
    inventory: &HistoricalSnapshotInventoryV0,
    policy: &HistoricalEvidencePolicyV0,
    config: &HistoricalHarvestConfigV0,
    registry: &ReadOnlyProviderRegistry,
) -> Result<HistoricalProviderGateResultV0, HistoricalEvidenceErrorV0> {
    policy.validate()?;
    config.validate()?;
    if inventory.accepted_series.len() >= policy.minimum_real_series {
        return Ok(HistoricalProviderGateResultV0 {
            status: HistoricalProviderGateStatusV0::ExistingEvidenceSufficient,
            selected_provider_id: None,
            rejected_provider_ids: vec![],
            reason_codes: vec!["existing_snapshot_evidence_sufficient".to_string()],
        });
    }
    let mut candidates = registry.providers.values().collect::<Vec<_>>();
    candidates.sort_by_key(|capability| provider_priority(capability, &config.provider_preference));
    let selected = candidates
        .iter()
        .find(|capability| provider_supports_harvest(capability, config));
    let rejected_provider_ids = candidates
        .iter()
        .filter(|capability| !provider_supports_harvest(capability, config))
        .map(|capability| capability.provider_id.clone())
        .collect();
    Ok(match selected {
        Some(capability) => HistoricalProviderGateResultV0 {
            status: HistoricalProviderGateStatusV0::ApprovedProviderSelected,
            selected_provider_id: Some(capability.provider_id.clone()),
            rejected_provider_ids,
            reason_codes: vec!["approved_readonly_provider_selected".to_string()],
        },
        None => HistoricalProviderGateResultV0 {
            status: HistoricalProviderGateStatusV0::NoApprovedHistoricalProviderConfigured,
            selected_provider_id: None,
            rejected_provider_ids,
            reason_codes: vec!["no_approved_historical_provider_configured".to_string()],
        },
    })
}

/// Harvests only configured daily series through the existing broker. The
/// selected provider is narrowed to the already-approved registry entry so a
/// preferred/approved gate and the actual request cannot diverge.
pub fn harvest_historical_snapshots_v0(
    inventory: &HistoricalSnapshotInventoryV0,
    policy: &HistoricalEvidencePolicyV0,
    config: &HistoricalHarvestConfigV0,
    broker: &mut DataAcquisitionBroker,
    now_ms: u64,
    provider: Option<&mut dyn ReadOnlyMarketDataProvider>,
) -> Result<HistoricalHarvestResultV0, HistoricalEvidenceErrorV0> {
    let provider_gate = select_approved_historical_provider_v0(
        inventory,
        policy,
        config,
        &broker.provider_registry,
    )?;
    match provider_gate.status {
        HistoricalProviderGateStatusV0::ExistingEvidenceSufficient => {
            return Ok(HistoricalHarvestResultV0 {
                status: HistoricalAcquisitionStatusV0::ExistingSnapshotsUsed,
                provider_gate,
                acquired_snapshots: vec![],
                reused_snapshots: vec![],
                failure_count: 0,
                reason_codes: vec!["existing_snapshots_used_without_provider_call".to_string()],
            });
        }
        HistoricalProviderGateStatusV0::NoApprovedHistoricalProviderConfigured => {
            let status = if inventory.total_snapshots > 0
                && inventory.real_historical_snapshots == 0
                && inventory.synthetic_snapshots > 0
            {
                HistoricalAcquisitionStatusV0::NoRealHistoricalEvidence
            } else if inventory.total_snapshots > 0 {
                HistoricalAcquisitionStatusV0::InsufficientHistoricalEvidence
            } else {
                HistoricalAcquisitionStatusV0::NoApprovedHistoricalProviderConfigured
            };
            return Ok(HistoricalHarvestResultV0 {
                status,
                provider_gate,
                acquired_snapshots: vec![],
                reused_snapshots: vec![],
                failure_count: 0,
                reason_codes: vec!["historical_harvest_not_started".to_string()],
            });
        }
        HistoricalProviderGateStatusV0::ApprovedProviderSelected => {}
    }
    let selected_provider_id = provider_gate
        .selected_provider_id
        .clone()
        .ok_or(HistoricalEvidenceErrorV0::InvalidConfig)?;
    let selected_capability = broker
        .provider_registry
        .providers
        .get(&selected_provider_id)
        .cloned()
        .ok_or(HistoricalEvidenceErrorV0::InvalidConfig)?;
    let mut scoped_registry = ReadOnlyProviderRegistry::default();
    scoped_registry.register(selected_capability);
    let intents = historical_harvest_intents(config);
    let plan = build_acquisition_plan(
        &intents,
        &scoped_registry,
        AcquisitionMode::ApprovedReadOnlyNetwork,
        &broker.acquisition_policy,
    );
    let execution = broker.execute_acquisition_plan(
        &plan,
        AcquisitionMode::ApprovedReadOnlyNetwork,
        now_ms,
        provider,
    );
    let failure_count = execution
        .receipts
        .iter()
        .filter(|receipt| {
            !matches!(
                receipt.status,
                crate::data::AcquisitionReceiptStatus::Acquired
                    | crate::data::AcquisitionReceiptStatus::ReusedSnapshot
            )
        })
        .count();
    let acquired_any =
        !execution.new_snapshots.is_empty() || !execution.reused_snapshots.is_empty();
    Ok(HistoricalHarvestResultV0 {
        status: if acquired_any && (config.allow_partial_harvest || failure_count == 0) {
            HistoricalAcquisitionStatusV0::SnapshotsAcquired
        } else {
            HistoricalAcquisitionStatusV0::HistoricalAcquisitionFailed
        },
        provider_gate,
        acquired_snapshots: execution.new_snapshots,
        reused_snapshots: execution.reused_snapshots,
        failure_count,
        reason_codes: execution
            .reason_codes
            .iter()
            .map(|reason| format!("{reason:?}"))
            .collect(),
    })
}

pub fn run_momentum_series_campaigns_v0(
    pack: &MomentumHistoricalEvidencePackV0,
    campaign_config: &MomentumLearningCampaignConfigV0,
    encoder: &FrozenMamba3EncoderV0,
) -> Result<Vec<MomentumSeriesCampaignResultV0>, HistoricalEvidenceErrorV0> {
    verify_momentum_historical_evidence_pack_v0(pack)?;
    let mut results = pack
        .series
        .iter()
        .map(|series| {
            let mut config = campaign_config.clone();
            config.campaign_id = format!(
                "{}-{}",
                campaign_config.campaign_id,
                stable_hash_string(&series.series_id)
            );
            let campaign = run_momentum_learning_campaign_v0(&config, &series.snapshots, encoder)
                .map_err(|_| HistoricalEvidenceErrorV0::CampaignConfigurationRejected)?;
            Ok(MomentumSeriesCampaignResultV0 {
                series_id: series.series_id.clone(),
                symbol: series.symbol.clone(),
                market: series.market,
                snapshot_ids: series.snapshot_ids.clone(),
                campaign,
                evidence_status: HistoricalSnapshotStatusV0::Ready,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    results.sort_by(|left, right| (left.market, &left.symbol).cmp(&(right.market, &right.symbol)));
    Ok(results)
}

pub fn aggregate_cross_series_momentum_evidence_v0(
    results: &[MomentumSeriesCampaignResultV0],
    gate: &CrossSeriesMomentumGateConfigV0,
    provider_status: HistoricalProviderGateStatusV0,
) -> Result<
    (
        CrossSeriesMomentumEvidenceV0,
        CrossSeriesWarmStartEvidenceV0,
    ),
    HistoricalEvidenceErrorV0,
> {
    gate.validate()?;
    if results.is_empty() {
        let status = if provider_status
            == HistoricalProviderGateStatusV0::NoApprovedHistoricalProviderConfigured
        {
            CrossSeriesMomentumVerdictV0::NoApprovedProvider
        } else {
            CrossSeriesMomentumVerdictV0::NoHistoricalEvidence
        };
        return Ok((empty_cross_series(status), empty_warm_start()));
    }
    let sufficient = results
        .iter()
        .filter(|result| {
            result.campaign.aggregate_mamba_evidence.status
                != MambaRepresentationValueStatusV0::InsufficientEvidence
        })
        .collect::<Vec<_>>();
    let evaluated_windows = results
        .iter()
        .map(|result| result.campaign.aggregate_mamba_evidence.evaluated_windows)
        .sum();
    let mamba_beats_constant_windows = sufficient
        .iter()
        .map(|result| {
            result
                .campaign
                .aggregate_mamba_evidence
                .mamba_beats_constant_count
        })
        .sum();
    let high_confidence_error_delta_vs_linear = sufficient
        .iter()
        .map(|result| {
            result
                .campaign
                .aggregate_mamba_evidence
                .high_confidence_error_delta_vs_linear
        })
        .sum();
    let mamba_wins = sufficient
        .iter()
        .filter(|result| {
            result
                .campaign
                .aggregate_mamba_evidence
                .mean_brier_delta_vs_linear
                < -gate.brier_improvement_epsilon
        })
        .count();
    let linear_wins = sufficient
        .iter()
        .filter(|result| {
            result
                .campaign
                .aggregate_mamba_evidence
                .mean_brier_delta_vs_linear
                > gate.brier_improvement_epsilon
        })
        .count();
    let mixed = sufficient.len().saturating_sub(mamba_wins + linear_wins);
    let deltas = sufficient
        .iter()
        .map(|result| {
            result
                .campaign
                .aggregate_mamba_evidence
                .mean_brier_delta_vs_linear
        })
        .collect::<Vec<_>>();
    let drifted = results
        .iter()
        .filter(|result| {
            !matches!(
                result.campaign.aggregate_drift,
                ModelDriftStatusV0::Stable | ModelDriftStatusV0::InsufficientEvidence
            )
        })
        .count();
    let safety_rejected = results
        .iter()
        .filter(|result| {
            result.campaign.status == MomentumLearningCampaignStatusV0::RejectedForSafety
        })
        .count();
    let status = if safety_rejected > 0 {
        CrossSeriesMomentumVerdictV0::RejectedForSafety
    } else if results.len() < gate.minimum_real_series
        || sufficient.len() < gate.minimum_sufficient_series
        || evaluated_windows < gate.minimum_evaluated_windows
    {
        CrossSeriesMomentumVerdictV0::InsufficientEvidence
    } else if drifted as f32 / results.len() as f32 > gate.maximum_drifted_series_fraction {
        CrossSeriesMomentumVerdictV0::DriftRisk
    } else if mamba_wins as f32 / sufficient.len() as f32 >= gate.minimum_mamba_win_fraction
        && mean(&deltas) < -gate.brier_improvement_epsilon
    {
        CrossSeriesMomentumVerdictV0::MambaHelped
    } else if linear_wins > mamba_wins && mean(&deltas) > gate.brier_improvement_epsilon {
        CrossSeriesMomentumVerdictV0::LinearBaselineStronger
    } else {
        CrossSeriesMomentumVerdictV0::Mixed
    };
    let warm = aggregate_warm_start(results);
    Ok((
        CrossSeriesMomentumEvidenceV0 {
            evaluated_series: results.len(),
            sufficient_series: sufficient.len(),
            total_evaluated_windows: evaluated_windows,
            mamba_beats_constant_windows,
            mamba_beats_linear_series: mamba_wins,
            linear_beats_mamba_series: linear_wins,
            mixed_series: mixed,
            insufficient_series: results.len() - sufficient.len(),
            warm_beats_cold_series: warm.warm_beats_cold_series,
            cold_beats_warm_series: warm.cold_beats_warm_series,
            drifted_series: drifted,
            mean_brier_delta_vs_linear: mean(&deltas),
            median_brier_delta_vs_linear: median(deltas),
            high_confidence_error_delta_vs_linear,
            status,
        },
        warm,
    ))
}

pub fn build_momentum_historical_evidence_report_v0(
    inventory: &HistoricalSnapshotInventoryV0,
    provider: &HistoricalProviderGateResultV0,
    acquisition_status: HistoricalAcquisitionStatusV0,
    results: &[MomentumSeriesCampaignResultV0],
    evidence: &CrossSeriesMomentumEvidenceV0,
    warm: &CrossSeriesWarmStartEvidenceV0,
) -> MomentumHistoricalEvidenceReportV0 {
    let mut lines = vec![
        format!("acquisition_status={acquisition_status:?}"),
        format!("provider_status={:?}", provider.status),
        format!("snapshot_inventory_total={}", inventory.total_snapshots),
        format!(
            "real_historical_snapshots={}",
            inventory.real_historical_snapshots
        ),
        format!(
            "synthetic_fixtures_excluded={}",
            inventory.synthetic_snapshots
        ),
        format!("accepted_series={}", inventory.accepted_series.len()),
        format!("rejected_snapshots={}", inventory.rejected_snapshots.len()),
        format!("cross_series_verdict={:?}", evidence.status),
        format!("warm_start_verdict={:?}", warm.status),
        format!(
            "mamba_beats_constant_windows={}",
            evidence.mamba_beats_constant_windows
        ),
        format!(
            "mamba_vs_linear_brier_delta={:.6}",
            evidence.mean_brier_delta_vs_linear
        ),
        format!(
            "high_confidence_error_delta_vs_linear={}",
            evidence.high_confidence_error_delta_vs_linear
        ),
    ];
    for series in &inventory.accepted_series {
        lines.push(format!(
            "accepted_series_detail={:?}:{}:rows={}:snapshots={}",
            series.market,
            series.symbol,
            series.row_count,
            series.snapshot_ids.join(",")
        ));
    }
    for rejected in &inventory.rejected_snapshots {
        lines.push(format!(
            "rejected_snapshot_detail={}:class={:?}:status={:?}",
            rejected.snapshot_id, rejected.evidence_class, rejected.status
        ));
    }
    for result in results {
        lines.push(format!(
            "campaign_series_detail={:?}:{}:status={:?}:windows={}:mamba_status={:?}:drift={:?}",
            result.market,
            result.symbol,
            result.campaign.status,
            result.campaign.aggregate_mamba_evidence.evaluated_windows,
            result.campaign.aggregate_mamba_evidence.status,
            result.campaign.aggregate_drift
        ));
    }
    if acquisition_status == HistoricalAcquisitionStatusV0::SnapshotsAcquired {
        lines.push("historical_evidence_acquired_through_readonly_broker".to_string());
    }
    if provider.status == HistoricalProviderGateStatusV0::NoApprovedHistoricalProviderConfigured {
        lines.push("no_approved_historical_provider_configured".to_string());
    }
    lines.extend([
        "momentum_model=ShadowOnly".to_string(),
        "no_profitability_claim".to_string(),
        "no_live_trading_readiness".to_string(),
        "official_mamba3_conformance=blocked".to_string(),
        "next_required_evidence=approved_daily_ohlcv_snapshot_series".to_string(),
    ]);
    MomentumHistoricalEvidenceReportV0 {
        acquisition_status,
        provider_status: provider.status,
        lines,
    }
}

fn classify_snapshot(
    snapshot: &DataSnapshot,
    policy: &HistoricalEvidencePolicyV0,
) -> (HistoricalEvidenceClassV0, HistoricalSnapshotStatusV0) {
    if snapshot.provenance.source_type == SnapshotSourceType::Mock {
        return (
            HistoricalEvidenceClassV0::SyntheticTestOnly,
            HistoricalSnapshotStatusV0::SyntheticTestOnly,
        );
    }
    if !matches!(
        snapshot.dataset_kind,
        DatasetKind::DailyOhlcv | DatasetKind::AdjustedDailyOhlcv
    ) {
        return (
            HistoricalEvidenceClassV0::Rejected,
            HistoricalSnapshotStatusV0::UnsupportedDataset,
        );
    }
    if !snapshot.read_only
        || !snapshot.sanitized
        || !snapshot.provenance.sanitized
        || !snapshot.provenance.credential_free
        || !snapshot.quality_summary.accepted
        || !snapshot
            .reason_codes
            .iter()
            .any(|reason| matches!(reason, crate::core::ReasonCode::DataSnapshotImmutable))
    {
        return (
            HistoricalEvidenceClassV0::Rejected,
            HistoricalSnapshotStatusV0::Mutable,
        );
    }
    if !snapshot_digest_valid(snapshot) {
        return (
            HistoricalEvidenceClassV0::Rejected,
            HistoricalSnapshotStatusV0::InvalidDigest,
        );
    }
    if snapshot
        .normalized_dataset
        .rows
        .windows(2)
        .any(|pair| pair[0].timestamp_ms >= pair[1].timestamp_ms)
    {
        return (
            HistoricalEvidenceClassV0::Rejected,
            HistoricalSnapshotStatusV0::InvalidChronology,
        );
    }
    if !snapshot_rows_are_valid(snapshot) || snapshot_contains_unsafe_text(snapshot) {
        return (
            HistoricalEvidenceClassV0::Rejected,
            HistoricalSnapshotStatusV0::Unsafe,
        );
    }
    let approved = snapshot.provenance.source_type == SnapshotSourceType::ApprovedReadOnlyProvider;
    let owner_local = snapshot.provenance.source_type == SnapshotSourceType::LocalSnapshotReplay
        && policy
            .owner_sanitized_snapshot_ids
            .contains(&snapshot.snapshot_id);
    if (!approved && !owner_local) || snapshot_contains_unsafe_text(snapshot) {
        return (
            HistoricalEvidenceClassV0::Rejected,
            HistoricalSnapshotStatusV0::Unsafe,
        );
    }
    (
        HistoricalEvidenceClassV0::RealHistorical,
        HistoricalSnapshotStatusV0::Ready,
    )
}

fn snapshot_digest_valid(snapshot: &DataSnapshot) -> bool {
    historical_replay_dataset_digest_v0(&snapshot.normalized_dataset) == snapshot.content_digest
}

fn snapshot_contains_unsafe_text(snapshot: &DataSnapshot) -> bool {
    let text = format!(
        "{} {} {} {}",
        snapshot.provider_id,
        snapshot.provenance.provider_id,
        snapshot.normalized_dataset.symbol,
        snapshot.normalized_dataset.source
    )
    .to_ascii_lowercase();
    [
        "authorization",
        "bearer",
        "account",
        "order",
        "api_key",
        "secret",
        "token",
        "raw_response",
    ]
    .iter()
    .any(|marker| text.contains(marker))
}

fn provider_supports_harvest(
    capability: &ProviderCapabilities,
    config: &HistoricalHarvestConfigV0,
) -> bool {
    capability.enabled
        && capability.read_only
        && capability.approved_for_network
        && !capability.mock_only
        && capability
            .supported_cadences
            .iter()
            .any(|cadence| cadence == "1d")
        && capability.supported_dataset_kinds.iter().any(|kind| {
            matches!(
                kind,
                DatasetKind::DailyOhlcv | DatasetKind::AdjustedDailyOhlcv
            )
        })
        && config.markets.iter().any(|market| {
            capability.supported_markets.contains(market)
                && !config.configured_universe.symbols_for(*market).is_empty()
        })
        && capability.maximum_lookback_bars >= config.minimum_rows_per_series
}

fn provider_priority(capability: &ProviderCapabilities, preference: &[String]) -> (usize, String) {
    (
        preference
            .iter()
            .position(|id| id == &capability.provider_id)
            .unwrap_or(usize::MAX),
        capability.provider_id.clone(),
    )
}

fn historical_harvest_intents(config: &HistoricalHarvestConfigV0) -> Vec<AgentDataIntent> {
    let mut intents = Vec::new();
    let mut markets = config.markets.clone();
    markets.sort();
    markets.dedup();
    for market in markets {
        let mut symbols = config.configured_universe.symbols_for(market);
        symbols.sort();
        symbols.dedup();
        for symbol in symbols {
            intents.push(AgentDataIntent {
                agent_id: format!(
                    "momentum_historical_harvest-{}",
                    stable_hash_string(&format!("{market:?}:{symbol}"))
                ),
                agent_kind: AgentKind::MomentumTrendFast,
                market_scope: market,
                symbols: vec![symbol],
                required_datasets: vec![DatasetKind::DailyOhlcv],
                optional_datasets: vec![],
                lookback: DataLookback {
                    bars: config.maximum_rows_per_series,
                    start_timestamp_ms: config.start_timestamp_ms,
                    end_timestamp_ms: config.end_timestamp_ms,
                },
                target_cadence: "1d".to_string(),
                max_staleness_ms: config.max_staleness_ms,
                priority: DataPriority::Required,
                reason_codes: vec![],
            });
        }
    }
    intents
}

fn series_id(market: AcquisitionMarketScope, symbol: &str, snapshots: &[DataSnapshot]) -> String {
    format!(
        "series-{}",
        stable_hash_string(&format!(
            "{:?}:{}:{}",
            market,
            symbol,
            snapshots
                .iter()
                .map(|snapshot| snapshot.snapshot_id.as_str())
                .collect::<Vec<_>>()
                .join(",")
        ))
    )
}

fn pack_material(series: &[MomentumHistoricalSeriesV0]) -> String {
    series
        .iter()
        .map(|series| {
            format!(
                "{:?}:{}:{}:{}",
                series.market,
                series.symbol,
                series.series_id,
                series.snapshot_ids.join(",")
            )
        })
        .collect::<Vec<_>>()
        .join("|")
}

fn unique_source_types(
    values: impl Iterator<Item = SnapshotSourceType>,
) -> Vec<SnapshotSourceType> {
    let mut unique = Vec::new();
    for value in values {
        if !unique.contains(&value) {
            unique.push(value);
        }
    }
    unique
}

fn snapshot_rows_are_valid(snapshot: &DataSnapshot) -> bool {
    snapshot.row_count == snapshot.normalized_dataset.rows.len()
        && snapshot.quality_summary.row_count == snapshot.row_count
        && snapshot.symbols.len() == 1
        && snapshot.symbols[0] == snapshot.normalized_dataset.symbol
        && snapshot.normalized_dataset.rows.iter().all(|row| {
            row.symbol == snapshot.normalized_dataset.symbol
                && row.open.is_finite()
                && row.high.is_finite()
                && row.low.is_finite()
                && row.close.is_finite()
                && row.volume.is_finite()
                && row.open > 0.0
                && row.low > 0.0
                && row.high >= row.open.max(row.close)
                && row.low <= row.open.min(row.close)
                && row.volume >= 0.0
                && row
                    .trade_value
                    .is_none_or(|value| value.is_finite() && value >= 0.0)
        })
}

fn aggregate_warm_start(
    results: &[MomentumSeriesCampaignResultV0],
) -> CrossSeriesWarmStartEvidenceV0 {
    let values = results
        .iter()
        .filter_map(|result| result.campaign.warm_start_evidence.as_ref())
        .collect::<Vec<_>>();
    let mut warm = 0;
    let mut cold = 0;
    let mut ties = 0;
    for value in &values {
        if value.warm_beats_cold_count > value.cold_beats_warm_count {
            warm += 1;
        } else if value.cold_beats_warm_count > value.warm_beats_cold_count {
            cold += 1;
        } else {
            ties += 1;
        }
    }
    let status = if values.is_empty() {
        CrossSeriesWarmStartVerdictV0::InsufficientEvidence
    } else if warm > cold {
        CrossSeriesWarmStartVerdictV0::Helped
    } else if cold > warm {
        CrossSeriesWarmStartVerdictV0::Failed
    } else {
        CrossSeriesWarmStartVerdictV0::Mixed
    };
    CrossSeriesWarmStartEvidenceV0 {
        compared_series: values.len(),
        warm_beats_cold_series: warm,
        cold_beats_warm_series: cold,
        tie_series: ties,
        status,
    }
}

fn empty_cross_series(status: CrossSeriesMomentumVerdictV0) -> CrossSeriesMomentumEvidenceV0 {
    CrossSeriesMomentumEvidenceV0 {
        evaluated_series: 0,
        sufficient_series: 0,
        total_evaluated_windows: 0,
        mamba_beats_constant_windows: 0,
        mamba_beats_linear_series: 0,
        linear_beats_mamba_series: 0,
        mixed_series: 0,
        insufficient_series: 0,
        warm_beats_cold_series: 0,
        cold_beats_warm_series: 0,
        drifted_series: 0,
        mean_brier_delta_vs_linear: 0.0,
        median_brier_delta_vs_linear: 0.0,
        high_confidence_error_delta_vs_linear: 0,
        status,
    }
}

fn empty_warm_start() -> CrossSeriesWarmStartEvidenceV0 {
    CrossSeriesWarmStartEvidenceV0 {
        compared_series: 0,
        warm_beats_cold_series: 0,
        cold_beats_warm_series: 0,
        tie_series: 0,
        status: CrossSeriesWarmStartVerdictV0::InsufficientEvidence,
    }
}

fn sorted_unique(values: impl Iterator<Item = String>) -> Vec<String> {
    let mut values = values.collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn sorted_unique_market(
    values: impl Iterator<Item = AcquisitionMarketScope>,
) -> Vec<AcquisitionMarketScope> {
    let mut values = values.collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

/// Classification for immutable historical evidence.  These labels are
/// deliberately conservative: a row that informed any research decision can
/// never later be represented as a pristine prospective holdout row.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EvidenceUsageClassV0 {
    ConsumedForTraining,
    ConsumedForValidation,
    ConsumedForTest,
    ConsumedForDiagnostics,
    ConsumedCounterfactual,
    DevelopmentEligible,
    ProspectiveHoldoutReserved,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoricalTimestampRangeV0 {
    pub start_timestamp_ms: u64,
    pub end_timestamp_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoricalEvidenceUsageRecordV0 {
    pub range: HistoricalTimestampRangeV0,
    pub usage_classes: Vec<EvidenceUsageClassV0>,
    pub campaign_ids: Vec<String>,
    pub model_version_ids: Vec<String>,
    pub labels_accessed: bool,
    pub counterfactual_accessed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoricalEvidenceUsageLedgerV0 {
    pub ledger_version: String,
    pub series_id: String,
    pub source_snapshot_ids: Vec<String>,
    pub usages: Vec<HistoricalEvidenceUsageRecordV0>,
    pub maximum_consumed_timestamp_ms: u64,
    pub ledger_digest: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BtcHistoricalExpansionStatusV0 {
    #[default]
    ExistingEvidenceOnly,
    BackfillNotAuthorized,
    BackfillPreflightBlocked,
    BackfillRequestBudgetRejected,
    BackfillFailed,
    ExpandedSnapshotAccepted,
    ExpandedSnapshotRejected,
    InsufficientHistoricalRows,
}

pub fn build_historical_evidence_usage_ledger_v0(
    snapshot: &DataSnapshot,
    campaigns: &[MomentumLearningCampaignResultV0],
) -> Result<HistoricalEvidenceUsageLedgerV0, HistoricalEvidenceErrorV0> {
    if !snapshot_rows_are_valid(snapshot)
        || snapshot.content_digest
            != historical_replay_dataset_digest_v0(&snapshot.normalized_dataset)
    {
        return Err(HistoricalEvidenceErrorV0::InvalidConfig);
    }
    let mut usages = Vec::new();
    for campaign in campaigns {
        let model_version_ids = sorted_unique(
            campaign
                .generated_versions
                .iter()
                .map(|version| version.model_version_id.clone()),
        );
        for result in &campaign.windows {
            usages.push(usage_from_index_range(
                snapshot,
                &result.window.train_range,
                vec![
                    EvidenceUsageClassV0::ConsumedForTraining,
                    EvidenceUsageClassV0::DevelopmentEligible,
                ],
                &campaign.campaign_id,
                &model_version_ids,
                true,
                false,
            )?);
            usages.push(usage_from_index_range(
                snapshot,
                &result.window.validation_range,
                vec![
                    EvidenceUsageClassV0::ConsumedForValidation,
                    EvidenceUsageClassV0::ConsumedForDiagnostics,
                    EvidenceUsageClassV0::DevelopmentEligible,
                ],
                &campaign.campaign_id,
                &model_version_ids,
                true,
                false,
            )?);
            usages.push(usage_from_index_range(
                snapshot,
                &result.window.test_range,
                vec![
                    EvidenceUsageClassV0::ConsumedForTest,
                    EvidenceUsageClassV0::ConsumedForDiagnostics,
                    EvidenceUsageClassV0::ConsumedCounterfactual,
                    EvidenceUsageClassV0::DevelopmentEligible,
                ],
                &campaign.campaign_id,
                &model_version_ids,
                true,
                true,
            )?);
        }
    }
    if !snapshot.normalized_dataset.rows.is_empty() {
        usages.push(HistoricalEvidenceUsageRecordV0 {
            range: full_snapshot_range(snapshot)?,
            usage_classes: vec![
                EvidenceUsageClassV0::ConsumedForDiagnostics,
                EvidenceUsageClassV0::DevelopmentEligible,
            ],
            campaign_ids: vec![],
            model_version_ids: vec![],
            labels_accessed: false,
            counterfactual_accessed: false,
        });
    }
    let usages = normalize_usage_records(usages)?;
    let maximum_consumed_timestamp_ms = usages
        .iter()
        .map(|usage| usage.range.end_timestamp_ms)
        .max()
        .ok_or(HistoricalEvidenceErrorV0::InvalidConfig)?;
    let source_snapshot_ids = vec![snapshot.snapshot_id.clone()];
    let ledger_digest = stable_hash_string(&usage_ledger_material(
        &snapshot.normalized_dataset.symbol,
        &source_snapshot_ids,
        &usages,
    ));
    Ok(HistoricalEvidenceUsageLedgerV0 {
        ledger_version: "historical-evidence-usage-v0".to_string(),
        series_id: format!(
            "{:?}:{}",
            snapshot.market_scope, snapshot.normalized_dataset.symbol
        ),
        source_snapshot_ids,
        usages,
        maximum_consumed_timestamp_ms,
        ledger_digest,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemporalRegimeSegmentationPolicyV0 {
    EqualLengthChronological,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BtcHistoricalRegimeConfigV0 {
    pub minimum_regimes: usize,
    pub regime_rows: usize,
    pub inter_regime_gap_rows: usize,
    pub minimum_campaign_windows_per_regime: usize,
    pub segmentation_policy: TemporalRegimeSegmentationPolicyV0,
}

impl BtcHistoricalRegimeConfigV0 {
    pub fn validate(&self) -> Result<(), HistoricalEvidenceErrorV0> {
        if self.minimum_regimes == 0
            || self.regime_rows == 0
            || self.minimum_campaign_windows_per_regime == 0
        {
            Err(HistoricalEvidenceErrorV0::InvalidConfig)
        } else {
            Ok(())
        }
    }

    pub fn digest(&self) -> String {
        stable_hash_string(&format!(
            "{:?}:{}:{}:{}:{}",
            self.segmentation_policy,
            self.minimum_regimes,
            self.regime_rows,
            self.inter_regime_gap_rows,
            self.minimum_campaign_windows_per_regime,
        ))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BtcRegimeSegmentationStatusV0 {
    Ready,
    #[default]
    InsufficientRows,
    InvalidChronology,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BtcHistoricalRegimeV0 {
    pub regime_id: String,
    pub start_row_index: usize,
    pub end_row_index_exclusive: usize,
    pub start_timestamp_ms: u64,
    pub end_timestamp_ms: u64,
    pub row_count: usize,
    pub source_snapshot_id: String,
    pub usage_class: EvidenceUsageClassV0,
    pub segmentation_config_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BtcHistoricalRegimeSegmentationV0 {
    pub status: BtcRegimeSegmentationStatusV0,
    pub regimes: Vec<BtcHistoricalRegimeV0>,
    pub incomplete_rows: usize,
    pub segmentation_config_digest: String,
}

pub fn segment_btc_historical_regimes_v0(
    snapshot: &DataSnapshot,
    config: &BtcHistoricalRegimeConfigV0,
) -> Result<BtcHistoricalRegimeSegmentationV0, HistoricalEvidenceErrorV0> {
    config.validate()?;
    let rows = &snapshot.normalized_dataset.rows;
    if rows
        .windows(2)
        .any(|pair| pair[0].timestamp_ms >= pair[1].timestamp_ms)
    {
        return Ok(BtcHistoricalRegimeSegmentationV0 {
            status: BtcRegimeSegmentationStatusV0::InvalidChronology,
            regimes: vec![],
            incomplete_rows: rows.len(),
            segmentation_config_digest: config.digest(),
        });
    }
    let mut regimes = Vec::new();
    let mut start = 0usize;
    while start
        .checked_add(config.regime_rows)
        .is_some_and(|end| end <= rows.len())
    {
        let end = start + config.regime_rows;
        let first = &rows[start];
        let last = &rows[end - 1];
        regimes.push(BtcHistoricalRegimeV0 {
            regime_id: format!("btc-regime-{:03}", regimes.len() + 1),
            start_row_index: start,
            end_row_index_exclusive: end,
            start_timestamp_ms: first.timestamp_ms,
            end_timestamp_ms: last.timestamp_ms,
            row_count: config.regime_rows,
            source_snapshot_id: snapshot.snapshot_id.clone(),
            usage_class: EvidenceUsageClassV0::DevelopmentEligible,
            segmentation_config_digest: config.digest(),
        });
        if end == rows.len() {
            start = end;
            break;
        }
        start = match end.checked_add(config.inter_regime_gap_rows) {
            Some(next) if next < rows.len() => next,
            _ => {
                start = end;
                break;
            }
        };
    }
    let status = if regimes.len() >= config.minimum_regimes {
        BtcRegimeSegmentationStatusV0::Ready
    } else {
        BtcRegimeSegmentationStatusV0::InsufficientRows
    };
    Ok(BtcHistoricalRegimeSegmentationV0 {
        status,
        incomplete_rows: rows.len().saturating_sub(start),
        regimes,
        segmentation_config_digest: config.digest(),
    })
}

pub fn freeze_btc_historical_regime_packs_v0(
    snapshot: &DataSnapshot,
    segmentation: &BtcHistoricalRegimeSegmentationV0,
    policy: &HistoricalEvidencePolicyV0,
) -> Result<Vec<(BtcHistoricalRegimeV0, MomentumHistoricalEvidencePackV0)>, HistoricalEvidenceErrorV0>
{
    if segmentation.status != BtcRegimeSegmentationStatusV0::Ready {
        return Ok(vec![]);
    }
    let mut previous_end = 0usize;
    let mut packs = Vec::new();
    for regime in &segmentation.regimes {
        if regime.start_row_index < previous_end
            || regime.end_row_index_exclusive > snapshot.normalized_dataset.rows.len()
            || regime.start_row_index >= regime.end_row_index_exclusive
        {
            return Err(HistoricalEvidenceErrorV0::InvalidConfig);
        }
        previous_end = regime.end_row_index_exclusive;
        let regime_snapshot = snapshot_for_regime(snapshot, regime)?;
        let (_, pack) = freeze_momentum_historical_evidence_pack_v0(&[regime_snapshot], policy)?;
        verify_momentum_historical_evidence_pack_v0(&pack)?;
        packs.push((regime.clone(), pack));
    }
    Ok(packs)
}

#[derive(Clone, Debug, PartialEq)]
pub struct BtcTemporalRegimeEvidenceResultV0 {
    pub regime_id: String,
    pub row_count: usize,
    pub campaign_windows: usize,
    pub no_signal_windows: usize,
    pub selected_checkpoint_windows: usize,
    pub in_support_windows: usize,
    pub out_of_support_windows: usize,
    pub support_unavailable_windows: usize,
    pub validation_in_support_windows: usize,
    pub validation_out_of_support_windows: usize,
    pub validation_insufficient_windows: usize,
    pub validation_gate_unavailable_windows: usize,
    pub support_traces: Vec<MomentumSupportTraceV0>,
    pub earliest_shift_stage: EarliestTemporalShiftStageV0,
    pub temporal_root_cause: ProbabilityCollapseRootCauseV0,
    pub frozen_representation_breach_count: usize,
    pub warm_start_status: WarmStartLockInStatusV0,
    pub abstention_count: usize,
    pub accepted_predictive_versions: usize,
    pub final_verdict: SupportGatedMomentumSeriesVerdictV0,
    pub reason_codes: Vec<String>,
    pub campaign_config_digest: String,
    pub encoder_parameter_digest: String,
    pub report_digest: String,
}

/// A redacted, deterministic reference to one sealed chronological regime.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BtcTemporalRegimeRefV0 {
    pub regime_id: String,
    pub chronological_rank: usize,
    pub row_count: usize,
    pub range_digest: String,
    pub pack_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegimeExecutionHealthV0 {
    Completed,
    EvidenceLoadFailure,
    PackVerificationFailure,
    ConfigDigestMismatch,
    BackendUnavailable,
    NumericalFailure,
    CampaignRuntimeFailure,
    DiagnosticRuntimeFailure,
    ReportConstructionFailure,
    NondeterministicReplay,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegimeDiagnosticCompletenessV0 {
    Complete,
    PartialInsufficientSamples,
    PartialNoSelectedCheckpoint,
    PartialSupportGateUnavailable,
    MissingRequiredMetric,
    MissingRequiredStatus,
    InconsistentDiagnosticState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegimeModelEvidenceOutcomeV0 {
    NoUsableValidationSignal,
    ValidationSignalButOutOfSupport,
    InSupportUsableSignal,
    FrozenRepresentationShiftRisk,
    FeatureShiftRisk,
    SequenceShiftRisk,
    LogitShiftRisk,
    ProbabilityShiftRisk,
    WarmStartLockInRisk,
    LinearBaselineStronger,
    MixedEvidence,
    InsufficientEvidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegimeOperationalShadowResultV0 {
    ShadowPredictionResearchOnly,
    ShadowAbstainNoSignal,
    ShadowAbstainOutOfSupport,
    ShadowAbstainSupportUnavailable,
    ShadowAbstainInsufficientEvidence,
    ShadowAbstainDiagnosticFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegimeExecutionStageV0 {
    PackLoad,
    PackDigestVerification,
    RowChronologyVerification,
    CampaignConfiguration,
    FeatureExtraction,
    TrainOnlyNormalization,
    SequenceConstruction,
    WindowConstruction,
    CandidateRegistration,
    CandidateTraining,
    CheckpointTrajectory,
    ValidationSignalGate,
    CheckpointSelection,
    TestSealDecision,
    TestEvaluation,
    TemporalSupportGate,
    TemporalShiftDiagnostics,
    WarmColdDiagnostics,
    OperationalShadowResult,
    ModelVersionConstruction,
    RegimeReportConstruction,
    RegimeReportDigest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegimeExecutionStageStatusV0 {
    Completed,
    CompletedNoSignal,
    CompletedAbstained,
    NotApplicable,
    NotExecutedAfterFailure,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegimeExecutionStageResultV0 {
    pub stage: RegimeExecutionStageV0,
    pub status: RegimeExecutionStageStatusV0,
    pub reason_codes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegimeExecutionTraceV0 {
    pub regime_id: String,
    pub stages: Vec<RegimeExecutionStageResultV0>,
    pub execution_health: RegimeExecutionHealthV0,
    pub trace_digest: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BtcTemporalRegimeClosedResultV0 {
    pub regime: BtcTemporalRegimeRefV0,
    pub campaign_window_count: usize,
    pub execution_health: RegimeExecutionHealthV0,
    pub diagnostic_completeness: RegimeDiagnosticCompletenessV0,
    pub model_evidence_outcome: RegimeModelEvidenceOutcomeV0,
    pub operational_shadow_result: RegimeOperationalShadowResultV0,
    pub no_signal_windows: usize,
    pub selected_checkpoint_windows: usize,
    pub test_sealed_windows: usize,
    pub support_envelope_ready_windows: usize,
    pub support_gate_applicable_windows: usize,
    pub in_support_windows: usize,
    pub out_of_support_windows: usize,
    pub support_unavailable_windows: usize,
    pub validation_in_support_windows: usize,
    pub validation_out_of_support_windows: usize,
    pub support_insufficient_windows: usize,
    pub support_gate_unavailable_windows: usize,
    pub test_in_support_windows: usize,
    pub test_out_of_support_windows: usize,
    pub dominant_support_outcome: RegimeDominantSupportOutcomeV0,
    pub first_breach_metric: Option<String>,
    pub support_traces: Vec<MomentumSupportTraceV0>,
    pub earliest_shift_stage: Option<EarliestTemporalShiftStageV0>,
    pub temporal_root_cause: Option<ProbabilityCollapseRootCauseV0>,
    pub warm_start_status: WarmStartLockInStatusV0,
    pub abstention_count: usize,
    pub accepted_predictive_versions: usize,
    pub reason_codes: Vec<String>,
    pub execution_trace: RegimeExecutionTraceV0,
    pub report_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegimeDominantSupportOutcomeV0 {
    ValidationInSupport,
    ValidationOutOfSupport,
    TestOutOfSupport,
    SupportGateUnavailable,
    InsufficientSupportEvidence,
    NoUsableValidationSignal,
    Mixed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BtcCrossRegimeSupportStatusV0 {
    RepeatedSupportQualifiedEvidence,
    SparseSupportQualifiedEvidence,
    NoSupportQualifiedEvidence,
    ValidationOutOfSupportAcrossRegimes,
    SupportGateUnavailableAcrossRegimes,
    MixedSupportEvidence,
    PredominantlyNoUsableValidationSignal,
    InsufficientSupportEvidence,
    DiagnosticFailure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BtcCrossRegimeSupportEvidenceV0 {
    pub configured_regimes: usize,
    pub technically_completed_regimes: usize,
    pub total_campaign_windows: usize,
    pub selected_checkpoint_windows: usize,
    pub support_envelope_ready_windows: usize,
    pub support_gate_applicable_windows: usize,
    pub validation_in_support_windows: usize,
    pub validation_out_of_support_windows: usize,
    pub support_insufficient_windows: usize,
    pub support_gate_unavailable_windows: usize,
    pub test_in_support_windows: usize,
    pub test_out_of_support_windows: usize,
    pub accepted_predictive_versions: usize,
    pub operational_abstentions: usize,
    pub support_status: BtcCrossRegimeSupportStatusV0,
    pub representation_status: BtcCrossRegimeRepresentationStatusV0,
    pub report_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegimeModelDigestV0 {
    pub regime_id: String,
    pub campaign_config_digest: String,
    pub encoder_parameter_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrossRegimeModelFreezeProofV0 {
    pub regime_model_digests: Vec<RegimeModelDigestV0>,
    pub all_equal: bool,
    pub mismatch_fields: Vec<String>,
    pub proof_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrossRegimeDiagnosticFailureRootCauseV0 {
    ModelConfigDigestMismatch,
    MissingRequiredMetric,
    PerRegimeReportDigestFailure,
    CrossRegimeAggregationInvariantFailure,
    UnsupportedOutcomeMapping,
    NondeterministicReplay,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BtcCrossRegimeClosedEvidenceV0 {
    pub configured_regimes: usize,
    pub technically_completed_regimes: usize,
    pub technically_failed_regimes: usize,
    pub sufficient_regimes: usize,
    pub total_campaign_windows: usize,
    pub no_signal_windows: usize,
    pub selected_checkpoint_windows: usize,
    pub in_support_windows: usize,
    pub out_of_support_windows: usize,
    pub support_unavailable_windows: usize,
    pub frozen_representation_shift_regimes: usize,
    pub feature_shift_regimes: usize,
    pub sequence_shift_regimes: usize,
    pub logit_shift_regimes: usize,
    pub probability_shift_regimes: usize,
    pub accepted_predictive_versions: usize,
    pub operational_abstentions: usize,
    pub diagnostic_failure_root_cause: Option<CrossRegimeDiagnosticFailureRootCauseV0>,
    pub status: BtcCrossRegimeRepresentationStatusV0,
    pub report_digest: String,
}

pub fn run_btc_historical_regime_campaigns_v0(
    packs: &[(BtcHistoricalRegimeV0, MomentumHistoricalEvidencePackV0)],
    campaign_config: &MomentumLearningCampaignConfigV0,
    encoder: &FrozenMamba3EncoderV0,
) -> Result<Vec<BtcTemporalRegimeEvidenceResultV0>, HistoricalEvidenceErrorV0> {
    let campaign_config_digest = campaign_config.digest();
    let encoder_parameter_digest = encoder.parameter_digest();
    let mut evidence = Vec::new();
    for (regime, pack) in packs {
        let results = run_momentum_series_campaigns_v0(pack, campaign_config, encoder)?;
        let result = results
            .first()
            .ok_or(HistoricalEvidenceErrorV0::CampaignConfigurationRejected)?;
        let report = build_momentum_temporal_diagnostic_report_v0(
            &result.campaign,
            regime.row_count,
            &pack.digest,
        );
        evidence.push(regime_evidence_from_report(
            regime,
            &report,
            momentum_support_traces_v0(&result.campaign),
            campaign_config_digest.clone(),
            encoder_parameter_digest.clone(),
        ));
    }
    evidence.sort_by(|left, right| left.regime_id.cmp(&right.regime_id));
    Ok(evidence)
}

pub fn build_cross_regime_model_freeze_proof_v0(
    results: &[BtcTemporalRegimeEvidenceResultV0],
) -> CrossRegimeModelFreezeProofV0 {
    let mut regime_model_digests = results
        .iter()
        .map(|result| RegimeModelDigestV0 {
            regime_id: result.regime_id.clone(),
            campaign_config_digest: result.campaign_config_digest.clone(),
            encoder_parameter_digest: result.encoder_parameter_digest.clone(),
        })
        .collect::<Vec<_>>();
    regime_model_digests.sort_by(|left, right| left.regime_id.cmp(&right.regime_id));
    let first = regime_model_digests.first();
    let mut mismatch_fields = Vec::new();
    if regime_model_digests.iter().any(|digest| {
        first.is_some_and(|reference| {
            digest.campaign_config_digest != reference.campaign_config_digest
        })
    }) {
        mismatch_fields.push("campaign_config_digest".to_string());
    }
    if regime_model_digests.iter().any(|digest| {
        first.is_some_and(|reference| {
            digest.encoder_parameter_digest != reference.encoder_parameter_digest
        })
    }) {
        mismatch_fields.push("encoder_parameter_digest".to_string());
    }
    let all_equal = mismatch_fields.is_empty();
    let proof_digest = stable_hash_string(&format!(
        "{}:{}:{}",
        all_equal,
        mismatch_fields.join(":"),
        regime_model_digests
            .iter()
            .map(|digest| format!(
                "{}:{}:{}",
                digest.regime_id, digest.campaign_config_digest, digest.encoder_parameter_digest
            ))
            .collect::<Vec<_>>()
            .join(":"),
    ));
    CrossRegimeModelFreezeProofV0 {
        regime_model_digests,
        all_equal,
        mismatch_fields,
        proof_digest,
    }
}

pub fn close_btc_temporal_regime_result_v0(
    result: &BtcTemporalRegimeEvidenceResultV0,
    regime: BtcTemporalRegimeRefV0,
) -> BtcTemporalRegimeClosedResultV0 {
    let diagnostic_completeness = if result.selected_checkpoint_windows == 0 {
        RegimeDiagnosticCompletenessV0::PartialNoSelectedCheckpoint
    } else if result.support_unavailable_windows > 0 {
        RegimeDiagnosticCompletenessV0::PartialSupportGateUnavailable
    } else {
        RegimeDiagnosticCompletenessV0::Complete
    };
    let model_evidence_outcome = match result.final_verdict {
        SupportGatedMomentumSeriesVerdictV0::NoUsableValidationSignal => {
            RegimeModelEvidenceOutcomeV0::NoUsableValidationSignal
        }
        SupportGatedMomentumSeriesVerdictV0::TemporalOutOfSupportAbstention => {
            RegimeModelEvidenceOutcomeV0::ValidationSignalButOutOfSupport
        }
        SupportGatedMomentumSeriesVerdictV0::FrozenRepresentationShiftRisk => {
            RegimeModelEvidenceOutcomeV0::FrozenRepresentationShiftRisk
        }
        SupportGatedMomentumSeriesVerdictV0::WarmStartLockInRisk => {
            RegimeModelEvidenceOutcomeV0::WarmStartLockInRisk
        }
        SupportGatedMomentumSeriesVerdictV0::InSupportUsableSignalButLinearStrongerOnThisSeries => {
            RegimeModelEvidenceOutcomeV0::LinearBaselineStronger
        }
        SupportGatedMomentumSeriesVerdictV0::InSupportUsableSignalAndMambaHelpedOnThisSeries => {
            RegimeModelEvidenceOutcomeV0::InSupportUsableSignal
        }
        SupportGatedMomentumSeriesVerdictV0::InSupportMixedEvidence => {
            RegimeModelEvidenceOutcomeV0::MixedEvidence
        }
        SupportGatedMomentumSeriesVerdictV0::InsufficientEvidence
        | SupportGatedMomentumSeriesVerdictV0::CampaignFailed => {
            RegimeModelEvidenceOutcomeV0::InsufficientEvidence
        }
    };
    let operational_shadow_result = if result.selected_checkpoint_windows == 0 {
        RegimeOperationalShadowResultV0::ShadowAbstainNoSignal
    } else if result.support_unavailable_windows > 0 {
        RegimeOperationalShadowResultV0::ShadowAbstainSupportUnavailable
    } else if result.out_of_support_windows > 0 {
        RegimeOperationalShadowResultV0::ShadowAbstainOutOfSupport
    } else if result.in_support_windows > 0 {
        RegimeOperationalShadowResultV0::ShadowPredictionResearchOnly
    } else {
        RegimeOperationalShadowResultV0::ShadowAbstainInsufficientEvidence
    };
    let first_breach_metric = result
        .support_traces
        .iter()
        .filter_map(|trace| trace.validation.first_breach_metric)
        .map(|metric| format!("{metric:?}"))
        .next();
    let support_envelope_ready_windows = result
        .support_traces
        .iter()
        .filter(|trace| {
            trace.envelope.construction_status == SupportEnvelopeConstructionStatusV0::Ready
        })
        .count();
    let support_gate_applicable_windows = result
        .support_traces
        .iter()
        .filter(|trace| {
            trace.validation.gate_applicability == SupportGateApplicabilityStatusV0::Applicable
        })
        .count();
    let no_signal = result.selected_checkpoint_windows == 0;
    let dominant_support_outcome = if no_signal {
        RegimeDominantSupportOutcomeV0::NoUsableValidationSignal
    } else if result.validation_in_support_windows > 0
        && (result.validation_out_of_support_windows > 0 || result.out_of_support_windows > 0)
    {
        RegimeDominantSupportOutcomeV0::Mixed
    } else if result.validation_out_of_support_windows > 0 {
        RegimeDominantSupportOutcomeV0::ValidationOutOfSupport
    } else if result.out_of_support_windows > 0 {
        RegimeDominantSupportOutcomeV0::TestOutOfSupport
    } else if result.validation_gate_unavailable_windows > 0 {
        RegimeDominantSupportOutcomeV0::SupportGateUnavailable
    } else if result.validation_insufficient_windows > 0 {
        RegimeDominantSupportOutcomeV0::InsufficientSupportEvidence
    } else if result.validation_in_support_windows > 0 {
        RegimeDominantSupportOutcomeV0::ValidationInSupport
    } else {
        RegimeDominantSupportOutcomeV0::Mixed
    };
    let support_abstained = !no_signal
        && operational_shadow_result
            != RegimeOperationalShadowResultV0::ShadowPredictionResearchOnly;
    let stages = [
        RegimeExecutionStageV0::PackLoad,
        RegimeExecutionStageV0::PackDigestVerification,
        RegimeExecutionStageV0::RowChronologyVerification,
        RegimeExecutionStageV0::CampaignConfiguration,
        RegimeExecutionStageV0::FeatureExtraction,
        RegimeExecutionStageV0::TrainOnlyNormalization,
        RegimeExecutionStageV0::SequenceConstruction,
        RegimeExecutionStageV0::WindowConstruction,
        RegimeExecutionStageV0::CandidateRegistration,
        RegimeExecutionStageV0::CandidateTraining,
        RegimeExecutionStageV0::CheckpointTrajectory,
        RegimeExecutionStageV0::ValidationSignalGate,
        RegimeExecutionStageV0::CheckpointSelection,
        RegimeExecutionStageV0::TestSealDecision,
        RegimeExecutionStageV0::TestEvaluation,
        RegimeExecutionStageV0::TemporalSupportGate,
        RegimeExecutionStageV0::TemporalShiftDiagnostics,
        RegimeExecutionStageV0::WarmColdDiagnostics,
        RegimeExecutionStageV0::OperationalShadowResult,
        RegimeExecutionStageV0::ModelVersionConstruction,
        RegimeExecutionStageV0::RegimeReportConstruction,
        RegimeExecutionStageV0::RegimeReportDigest,
    ]
    .into_iter()
    .map(|stage| {
        let status = match stage {
            RegimeExecutionStageV0::ValidationSignalGate if no_signal => {
                RegimeExecutionStageStatusV0::CompletedNoSignal
            }
            RegimeExecutionStageV0::CheckpointSelection
            | RegimeExecutionStageV0::TestEvaluation
            | RegimeExecutionStageV0::TemporalSupportGate
            | RegimeExecutionStageV0::TemporalShiftDiagnostics
            | RegimeExecutionStageV0::ModelVersionConstruction
                if no_signal =>
            {
                RegimeExecutionStageStatusV0::NotApplicable
            }
            RegimeExecutionStageV0::TemporalSupportGate
            | RegimeExecutionStageV0::OperationalShadowResult
                if support_abstained =>
            {
                RegimeExecutionStageStatusV0::CompletedAbstained
            }
            RegimeExecutionStageV0::ModelVersionConstruction if support_abstained => {
                RegimeExecutionStageStatusV0::NotApplicable
            }
            _ => RegimeExecutionStageStatusV0::Completed,
        };
        RegimeExecutionStageResultV0 {
            stage,
            status,
            reason_codes: Vec::new(),
        }
    })
    .collect::<Vec<_>>();
    let trace_digest = stable_hash_string(&format!(
        "{}:{:?}:{}",
        result.regime_id,
        RegimeExecutionHealthV0::Completed,
        stages
            .iter()
            .map(|stage| format!("{:?}:{:?}", stage.stage, stage.status))
            .collect::<Vec<_>>()
            .join(":"),
    ));
    let execution_trace = RegimeExecutionTraceV0 {
        regime_id: result.regime_id.clone(),
        stages,
        execution_health: RegimeExecutionHealthV0::Completed,
        trace_digest,
    };
    let accepted_predictive_versions = if operational_shadow_result
        == RegimeOperationalShadowResultV0::ShadowPredictionResearchOnly
    {
        result.accepted_predictive_versions
    } else {
        0
    };
    let mut reason_codes = result.reason_codes.clone();
    if result.accepted_predictive_versions > accepted_predictive_versions {
        reason_codes.push("accepted_predictive_version_absent_by_policy".to_string());
    }
    reason_codes.sort();
    reason_codes.dedup();
    let report_digest = stable_hash_string(&format!(
        "{}:{}:{:?}:{:?}:{:?}:{:?}:{}:{}:{}",
        result.report_digest,
        execution_trace.trace_digest,
        diagnostic_completeness,
        model_evidence_outcome,
        operational_shadow_result,
        dominant_support_outcome,
        result.selected_checkpoint_windows,
        result.in_support_windows,
        reason_codes.join(":"),
    ));
    BtcTemporalRegimeClosedResultV0 {
        regime,
        campaign_window_count: result.campaign_windows,
        execution_health: RegimeExecutionHealthV0::Completed,
        diagnostic_completeness,
        model_evidence_outcome,
        operational_shadow_result,
        no_signal_windows: result.no_signal_windows,
        selected_checkpoint_windows: result.selected_checkpoint_windows,
        test_sealed_windows: result
            .campaign_windows
            .saturating_sub(result.selected_checkpoint_windows),
        support_envelope_ready_windows,
        support_gate_applicable_windows,
        in_support_windows: result.in_support_windows,
        out_of_support_windows: result.out_of_support_windows,
        support_unavailable_windows: result.support_unavailable_windows,
        validation_in_support_windows: result.validation_in_support_windows,
        validation_out_of_support_windows: result.validation_out_of_support_windows,
        support_insufficient_windows: result.validation_insufficient_windows,
        support_gate_unavailable_windows: result.validation_gate_unavailable_windows,
        test_in_support_windows: result.in_support_windows,
        test_out_of_support_windows: result.out_of_support_windows,
        dominant_support_outcome,
        first_breach_metric,
        support_traces: result.support_traces.clone(),
        earliest_shift_stage: (!no_signal).then_some(result.earliest_shift_stage),
        temporal_root_cause: (!no_signal).then_some(result.temporal_root_cause),
        warm_start_status: result.warm_start_status,
        abstention_count: result.abstention_count,
        accepted_predictive_versions,
        reason_codes,
        execution_trace,
        report_digest,
    }
}

pub fn validate_btc_temporal_regime_closed_result_v0(
    result: &BtcTemporalRegimeClosedResultV0,
) -> Result<(), CrossRegimeDiagnosticFailureRootCauseV0> {
    if result.selected_checkpoint_windows > result.campaign_window_count
        || result.in_support_windows > result.selected_checkpoint_windows
        || result.out_of_support_windows > result.selected_checkpoint_windows
        || result.support_unavailable_windows > result.selected_checkpoint_windows
        || result.accepted_predictive_versions > result.in_support_windows
        || result.execution_trace.stages.len() != 22
        || result.report_digest.is_empty()
    {
        return Err(CrossRegimeDiagnosticFailureRootCauseV0::MissingRequiredMetric);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BtcCrossRegimeRepresentationStatusV0 {
    RecurrentFrozenRepresentationShift,
    RecentRegimeSpecificRepresentationShift,
    HistoricalRegimeSpecificRepresentationShift,
    MixedShiftStages,
    PredominantlyNoUsableSignal,
    SparseInSupportEvidence,
    StableAcrossAvailableRegimes,
    InsufficientRegimes,
    DiagnosticFailure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BtcCrossRegimeEvidenceV0 {
    pub total_regimes: usize,
    pub valid_regimes: usize,
    pub insufficient_regimes: usize,
    pub total_windows: usize,
    pub no_signal_windows: usize,
    pub in_support_windows: usize,
    pub out_of_support_windows: usize,
    pub frozen_representation_shift_regimes: usize,
    pub feature_shift_regimes: usize,
    pub sequence_shift_regimes: usize,
    pub logit_shift_regimes: usize,
    pub probability_shift_regimes: usize,
    pub stable_regimes: usize,
    pub accepted_predictive_versions: usize,
    pub status: BtcCrossRegimeRepresentationStatusV0,
    pub report_digest: String,
}

pub fn aggregate_btc_cross_regime_evidence_v0(
    results: &[BtcTemporalRegimeEvidenceResultV0],
    config: &BtcHistoricalRegimeConfigV0,
) -> Result<BtcCrossRegimeEvidenceV0, HistoricalEvidenceErrorV0> {
    config.validate()?;
    let mut ordered = results.to_vec();
    ordered.sort_by(|left, right| left.regime_id.cmp(&right.regime_id));
    let valid = ordered
        .iter()
        .filter(|result| result.campaign_windows >= config.minimum_campaign_windows_per_regime)
        .collect::<Vec<_>>();
    let stage_count = |stage| {
        valid
            .iter()
            .filter(|result| result.earliest_shift_stage == stage)
            .count()
    };
    let frozen = stage_count(EarliestTemporalShiftStageV0::FrozenRepresentations);
    let feature = stage_count(EarliestTemporalShiftStageV0::RawFeatures)
        + stage_count(EarliestTemporalShiftStageV0::NormalizedFeatures);
    let sequence = stage_count(EarliestTemporalShiftStageV0::Sequences);
    let logit = stage_count(EarliestTemporalShiftStageV0::Logits)
        + stage_count(EarliestTemporalShiftStageV0::RepresentationScale);
    let probability = stage_count(EarliestTemporalShiftStageV0::Probabilities)
        + stage_count(EarliestTemporalShiftStageV0::OutcomesOnly);
    let stable = valid
        .iter()
        .filter(|result| result.in_support_windows > 0 && result.out_of_support_windows == 0)
        .count();
    let status = if valid.len() < config.minimum_regimes {
        BtcCrossRegimeRepresentationStatusV0::InsufficientRegimes
    } else if frozen >= config.minimum_regimes && feature == 0 && sequence == 0 {
        BtcCrossRegimeRepresentationStatusV0::RecurrentFrozenRepresentationShift
    } else if valid.iter().all(|result| result.no_signal_windows > 0)
        && valid
            .iter()
            .map(|result| result.no_signal_windows)
            .sum::<usize>()
            * 2
            >= valid
                .iter()
                .map(|result| result.campaign_windows)
                .sum::<usize>()
    {
        BtcCrossRegimeRepresentationStatusV0::PredominantlyNoUsableSignal
    } else if [frozen, feature, sequence, logit, probability]
        .iter()
        .filter(|count| **count > 0)
        .count()
        > 1
    {
        BtcCrossRegimeRepresentationStatusV0::MixedShiftStages
    } else if stable == valid.len() {
        BtcCrossRegimeRepresentationStatusV0::StableAcrossAvailableRegimes
    } else if valid
        .iter()
        .map(|result| result.in_support_windows)
        .sum::<usize>()
        == 0
    {
        BtcCrossRegimeRepresentationStatusV0::SparseInSupportEvidence
    } else {
        BtcCrossRegimeRepresentationStatusV0::DiagnosticFailure
    };
    let total_windows = ordered.iter().map(|result| result.campaign_windows).sum();
    let no_signal_windows = ordered.iter().map(|result| result.no_signal_windows).sum();
    let in_support_windows = ordered.iter().map(|result| result.in_support_windows).sum();
    let out_of_support_windows = ordered
        .iter()
        .map(|result| result.out_of_support_windows)
        .sum();
    let accepted_predictive_versions = ordered
        .iter()
        .map(|result| result.accepted_predictive_versions)
        .sum();
    let report_digest = stable_hash_string(&format!(
        "{:?}:{}:{}:{}:{}:{}:{}:{}",
        status,
        ordered.len(),
        valid.len(),
        total_windows,
        frozen,
        feature,
        sequence,
        ordered
            .iter()
            .map(|result| result.report_digest.as_str())
            .collect::<Vec<_>>()
            .join(":"),
    ));
    Ok(BtcCrossRegimeEvidenceV0 {
        total_regimes: ordered.len(),
        valid_regimes: valid.len(),
        insufficient_regimes: ordered.len().saturating_sub(valid.len()),
        total_windows,
        no_signal_windows,
        in_support_windows,
        out_of_support_windows,
        frozen_representation_shift_regimes: frozen,
        feature_shift_regimes: feature,
        sequence_shift_regimes: sequence,
        logit_shift_regimes: logit,
        probability_shift_regimes: probability,
        stable_regimes: stable,
        accepted_predictive_versions,
        status,
        report_digest,
    })
}

pub fn aggregate_btc_cross_regime_closed_evidence_v0(
    results: &[BtcTemporalRegimeClosedResultV0],
    config: &BtcHistoricalRegimeConfigV0,
    freeze_proof: &CrossRegimeModelFreezeProofV0,
) -> BtcCrossRegimeClosedEvidenceV0 {
    let mut ordered = results.to_vec();
    ordered.sort_by(|left, right| {
        left.regime
            .chronological_rank
            .cmp(&right.regime.chronological_rank)
            .then_with(|| left.regime.regime_id.cmp(&right.regime.regime_id))
    });
    let completed = ordered
        .iter()
        .filter(|result| result.execution_health == RegimeExecutionHealthV0::Completed)
        .collect::<Vec<_>>();
    let sufficient = completed
        .iter()
        .filter(|result| result.campaign_window_count >= config.minimum_campaign_windows_per_regime)
        .collect::<Vec<_>>();
    let invalid = ordered
        .iter()
        .any(|result| validate_btc_temporal_regime_closed_result_v0(result).is_err());
    let technical_failure = !freeze_proof.all_equal || invalid || completed.len() != ordered.len();
    let stage_count = |outcome| {
        sufficient
            .iter()
            .filter(|result| result.model_evidence_outcome == outcome)
            .count()
    };
    let frozen = stage_count(RegimeModelEvidenceOutcomeV0::FrozenRepresentationShiftRisk);
    let feature = stage_count(RegimeModelEvidenceOutcomeV0::FeatureShiftRisk);
    let sequence = stage_count(RegimeModelEvidenceOutcomeV0::SequenceShiftRisk);
    let logit = stage_count(RegimeModelEvidenceOutcomeV0::LogitShiftRisk);
    let probability = stage_count(RegimeModelEvidenceOutcomeV0::ProbabilityShiftRisk);
    let total_windows = ordered
        .iter()
        .map(|result| result.campaign_window_count)
        .sum::<usize>();
    let no_signal_windows = ordered
        .iter()
        .map(|result| result.no_signal_windows)
        .sum::<usize>();
    let selected_checkpoint_windows = ordered
        .iter()
        .map(|result| result.selected_checkpoint_windows)
        .sum::<usize>();
    let in_support_windows = ordered
        .iter()
        .map(|result| result.in_support_windows)
        .sum::<usize>();
    let out_of_support_windows = ordered
        .iter()
        .map(|result| result.out_of_support_windows)
        .sum::<usize>();
    let support_unavailable_windows = ordered
        .iter()
        .map(|result| result.support_unavailable_windows)
        .sum::<usize>();
    let accepted_predictive_versions = ordered
        .iter()
        .map(|result| result.accepted_predictive_versions)
        .sum::<usize>();
    let operational_abstentions = ordered
        .iter()
        .map(|result| result.abstention_count)
        .sum::<usize>();
    let (status, diagnostic_failure_root_cause) = if technical_failure {
        (
            BtcCrossRegimeRepresentationStatusV0::DiagnosticFailure,
            Some(if !freeze_proof.all_equal {
                CrossRegimeDiagnosticFailureRootCauseV0::ModelConfigDigestMismatch
            } else if invalid {
                CrossRegimeDiagnosticFailureRootCauseV0::MissingRequiredMetric
            } else {
                CrossRegimeDiagnosticFailureRootCauseV0::CrossRegimeAggregationInvariantFailure
            }),
        )
    } else if sufficient.len() < config.minimum_regimes {
        (
            BtcCrossRegimeRepresentationStatusV0::InsufficientRegimes,
            None,
        )
    } else if frozen >= config.minimum_regimes && feature == 0 && sequence == 0 {
        (
            BtcCrossRegimeRepresentationStatusV0::RecurrentFrozenRepresentationShift,
            None,
        )
    } else if frozen + feature + sequence + logit + probability > 1 {
        (BtcCrossRegimeRepresentationStatusV0::MixedShiftStages, None)
    } else if total_windows > 0 && no_signal_windows.saturating_mul(2) >= total_windows {
        (
            BtcCrossRegimeRepresentationStatusV0::PredominantlyNoUsableSignal,
            None,
        )
    } else if in_support_windows == 0 || support_unavailable_windows > 0 {
        (
            BtcCrossRegimeRepresentationStatusV0::SparseInSupportEvidence,
            None,
        )
    } else {
        (
            BtcCrossRegimeRepresentationStatusV0::StableAcrossAvailableRegimes,
            None,
        )
    };
    let report_digest = stable_hash_string(&format!(
        "{:?}:{:?}:{}:{}:{}:{}",
        status,
        diagnostic_failure_root_cause,
        freeze_proof.proof_digest,
        total_windows,
        no_signal_windows,
        ordered
            .iter()
            .map(|result| result.report_digest.as_str())
            .collect::<Vec<_>>()
            .join(":"),
    ));
    BtcCrossRegimeClosedEvidenceV0 {
        configured_regimes: config.minimum_regimes,
        technically_completed_regimes: completed.len(),
        technically_failed_regimes: ordered.len().saturating_sub(completed.len()),
        sufficient_regimes: sufficient.len(),
        total_campaign_windows: total_windows,
        no_signal_windows,
        selected_checkpoint_windows,
        in_support_windows,
        out_of_support_windows,
        support_unavailable_windows,
        frozen_representation_shift_regimes: frozen,
        feature_shift_regimes: feature,
        sequence_shift_regimes: sequence,
        logit_shift_regimes: logit,
        probability_shift_regimes: probability,
        accepted_predictive_versions,
        operational_abstentions,
        diagnostic_failure_root_cause,
        status,
        report_digest,
    }
}

pub fn aggregate_btc_cross_regime_support_evidence_v0(
    results: &[BtcTemporalRegimeClosedResultV0],
    config: &BtcHistoricalRegimeConfigV0,
    representation_status: BtcCrossRegimeRepresentationStatusV0,
) -> BtcCrossRegimeSupportEvidenceV0 {
    let sum =
        |f: fn(&BtcTemporalRegimeClosedResultV0) -> usize| results.iter().map(f).sum::<usize>();
    let technically_completed_regimes = results
        .iter()
        .filter(|result| result.execution_health == RegimeExecutionHealthV0::Completed)
        .count();
    let total_campaign_windows = sum(|result| result.campaign_window_count);
    let selected_checkpoint_windows = sum(|result| result.selected_checkpoint_windows);
    let support_envelope_ready_windows = sum(|result| result.support_envelope_ready_windows);
    let support_gate_applicable_windows = sum(|result| result.support_gate_applicable_windows);
    let validation_in_support_windows = sum(|result| result.validation_in_support_windows);
    let validation_out_of_support_windows = sum(|result| result.validation_out_of_support_windows);
    let support_insufficient_windows = sum(|result| result.support_insufficient_windows);
    let support_gate_unavailable_windows = sum(|result| result.support_gate_unavailable_windows);
    let test_in_support_windows = sum(|result| result.test_in_support_windows);
    let test_out_of_support_windows = sum(|result| result.test_out_of_support_windows);
    let accepted_predictive_versions = sum(|result| result.accepted_predictive_versions);
    let operational_abstentions = sum(|result| result.abstention_count);
    let support_status = if technically_completed_regimes != results.len() {
        BtcCrossRegimeSupportStatusV0::DiagnosticFailure
    } else if total_campaign_windows > 0
        && sum(|result| result.no_signal_windows) * 2 >= total_campaign_windows
    {
        BtcCrossRegimeSupportStatusV0::PredominantlyNoUsableValidationSignal
    } else if accepted_predictive_versions >= config.minimum_regimes
        && test_in_support_windows >= config.minimum_regimes
    {
        BtcCrossRegimeSupportStatusV0::RepeatedSupportQualifiedEvidence
    } else if accepted_predictive_versions > 0 && test_in_support_windows > 0 {
        BtcCrossRegimeSupportStatusV0::SparseSupportQualifiedEvidence
    } else if validation_out_of_support_windows > 0
        && validation_out_of_support_windows >= validation_in_support_windows
    {
        BtcCrossRegimeSupportStatusV0::ValidationOutOfSupportAcrossRegimes
    } else if support_gate_unavailable_windows > 0
        && support_gate_unavailable_windows >= support_gate_applicable_windows
    {
        BtcCrossRegimeSupportStatusV0::SupportGateUnavailableAcrossRegimes
    } else if support_insufficient_windows > 0 {
        BtcCrossRegimeSupportStatusV0::InsufficientSupportEvidence
    } else if validation_in_support_windows > 0 || test_out_of_support_windows > 0 {
        BtcCrossRegimeSupportStatusV0::MixedSupportEvidence
    } else {
        BtcCrossRegimeSupportStatusV0::NoSupportQualifiedEvidence
    };
    let report_digest = stable_hash_string(&format!(
        "{:?}:{:?}:{}:{}:{}:{}:{}:{}",
        support_status,
        representation_status,
        total_campaign_windows,
        validation_in_support_windows,
        validation_out_of_support_windows,
        test_in_support_windows,
        accepted_predictive_versions,
        results
            .iter()
            .map(|result| result.report_digest.as_str())
            .collect::<Vec<_>>()
            .join(":"),
    ));
    BtcCrossRegimeSupportEvidenceV0 {
        configured_regimes: config.minimum_regimes,
        technically_completed_regimes,
        total_campaign_windows,
        selected_checkpoint_windows,
        support_envelope_ready_windows,
        support_gate_applicable_windows,
        validation_in_support_windows,
        validation_out_of_support_windows,
        support_insufficient_windows,
        support_gate_unavailable_windows,
        test_in_support_windows,
        test_out_of_support_windows,
        accepted_predictive_versions,
        operational_abstentions,
        support_status,
        representation_status,
        report_digest,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProspectiveHoldoutStatusV0 {
    #[default]
    PolicyNotDefined,
    PolicySealedNoFutureRows,
    FutureRowsAccumulating,
    ReadyForOneTimeEvaluation,
    OpenedForOneTimeEvaluation,
    InvalidatedByEarlyAccess,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProspectiveHoldoutPolicyConfigV0 {
    pub minimum_future_rows: usize,
    pub required_future_windows: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProspectiveHoldoutManifestV0 {
    pub manifest_version: String,
    pub series_id: String,
    pub cutoff_exclusive_timestamp_ms: u64,
    pub minimum_future_rows: usize,
    pub required_future_windows: usize,
    pub policy_config_digest: String,
    pub status: ProspectiveHoldoutStatusV0,
    pub opened: bool,
    pub labels_accessed: bool,
    pub manifest_digest: String,
}

pub fn seal_prospective_holdout_v0(
    ledger: &HistoricalEvidenceUsageLedgerV0,
    policy: &ProspectiveHoldoutPolicyConfigV0,
    future_timestamps: &[u64],
) -> Result<ProspectiveHoldoutManifestV0, HistoricalEvidenceErrorV0> {
    if policy.minimum_future_rows == 0 || policy.required_future_windows == 0 {
        return Err(HistoricalEvidenceErrorV0::InvalidConfig);
    }
    if future_timestamps
        .iter()
        .any(|timestamp| *timestamp <= ledger.maximum_consumed_timestamp_ms)
    {
        return Err(HistoricalEvidenceErrorV0::InvalidConfig);
    }
    let status = if future_timestamps.is_empty() {
        ProspectiveHoldoutStatusV0::PolicySealedNoFutureRows
    } else if future_timestamps.len() < policy.minimum_future_rows {
        ProspectiveHoldoutStatusV0::FutureRowsAccumulating
    } else {
        ProspectiveHoldoutStatusV0::ReadyForOneTimeEvaluation
    };
    let policy_config_digest = stable_hash_string(&format!(
        "{}:{}",
        policy.minimum_future_rows, policy.required_future_windows
    ));
    let mut manifest = ProspectiveHoldoutManifestV0 {
        manifest_version: "prospective-holdout-v0".to_string(),
        series_id: ledger.series_id.clone(),
        cutoff_exclusive_timestamp_ms: ledger.maximum_consumed_timestamp_ms,
        minimum_future_rows: policy.minimum_future_rows,
        required_future_windows: policy.required_future_windows,
        policy_config_digest,
        status,
        opened: false,
        labels_accessed: false,
        manifest_digest: String::new(),
    };
    manifest.manifest_digest = holdout_manifest_digest(&manifest);
    Ok(manifest)
}

pub fn record_prospective_holdout_access_v0(
    manifest: &mut ProspectiveHoldoutManifestV0,
    opened: bool,
    labels_accessed: bool,
    model_selection_access: bool,
) {
    manifest.opened |= opened;
    manifest.labels_accessed |= labels_accessed;
    if model_selection_access || (labels_accessed && !opened) {
        manifest.status = ProspectiveHoldoutStatusV0::InvalidatedByEarlyAccess;
    } else if manifest.opened {
        manifest.status = ProspectiveHoldoutStatusV0::OpenedForOneTimeEvaluation;
    }
    manifest.manifest_digest = holdout_manifest_digest(manifest);
}

pub fn prospective_holdout_timestamp_is_eligible_v0(
    timestamp_ms: u64,
    ledger: &HistoricalEvidenceUsageLedgerV0,
    manifest: &ProspectiveHoldoutManifestV0,
) -> bool {
    timestamp_ms > ledger.maximum_consumed_timestamp_ms
        && timestamp_ms > manifest.cutoff_exclusive_timestamp_ms
        && manifest.status != ProspectiveHoldoutStatusV0::InvalidatedByEarlyAccess
}

fn usage_from_index_range(
    snapshot: &DataSnapshot,
    range: &super::IndexRangeV0,
    usage_classes: Vec<EvidenceUsageClassV0>,
    campaign_id: &str,
    model_version_ids: &[String],
    labels_accessed: bool,
    counterfactual_accessed: bool,
) -> Result<HistoricalEvidenceUsageRecordV0, HistoricalEvidenceErrorV0> {
    if range.start >= range.end || range.end > snapshot.normalized_dataset.rows.len() {
        return Err(HistoricalEvidenceErrorV0::InvalidConfig);
    }
    Ok(HistoricalEvidenceUsageRecordV0 {
        range: HistoricalTimestampRangeV0 {
            start_timestamp_ms: snapshot.normalized_dataset.rows[range.start].timestamp_ms,
            end_timestamp_ms: snapshot.normalized_dataset.rows[range.end - 1].timestamp_ms,
        },
        usage_classes,
        campaign_ids: vec![campaign_id.to_string()],
        model_version_ids: model_version_ids.to_vec(),
        labels_accessed,
        counterfactual_accessed,
    })
}

fn full_snapshot_range(
    snapshot: &DataSnapshot,
) -> Result<HistoricalTimestampRangeV0, HistoricalEvidenceErrorV0> {
    Ok(HistoricalTimestampRangeV0 {
        start_timestamp_ms: snapshot
            .normalized_dataset
            .rows
            .first()
            .ok_or(HistoricalEvidenceErrorV0::InvalidConfig)?
            .timestamp_ms,
        end_timestamp_ms: snapshot
            .normalized_dataset
            .rows
            .last()
            .ok_or(HistoricalEvidenceErrorV0::InvalidConfig)?
            .timestamp_ms,
    })
}

fn normalize_usage_records(
    mut usages: Vec<HistoricalEvidenceUsageRecordV0>,
) -> Result<Vec<HistoricalEvidenceUsageRecordV0>, HistoricalEvidenceErrorV0> {
    for usage in &mut usages {
        if usage.range.start_timestamp_ms > usage.range.end_timestamp_ms {
            return Err(HistoricalEvidenceErrorV0::InvalidConfig);
        }
        usage.usage_classes.sort();
        usage.usage_classes.dedup();
        usage.campaign_ids.sort();
        usage.campaign_ids.dedup();
        usage.model_version_ids.sort();
        usage.model_version_ids.dedup();
    }
    usages.sort_by(|left, right| {
        (
            left.range.start_timestamp_ms,
            left.range.end_timestamp_ms,
            &left.campaign_ids,
            &left.model_version_ids,
        )
            .cmp(&(
                right.range.start_timestamp_ms,
                right.range.end_timestamp_ms,
                &right.campaign_ids,
                &right.model_version_ids,
            ))
    });
    let mut normalized: Vec<HistoricalEvidenceUsageRecordV0> = Vec::new();
    for usage in usages {
        if let Some(current) = normalized.last_mut()
            && usage.range.start_timestamp_ms <= current.range.end_timestamp_ms
        {
            current.range.end_timestamp_ms = current
                .range
                .end_timestamp_ms
                .max(usage.range.end_timestamp_ms);
            current.usage_classes.extend(usage.usage_classes);
            current.usage_classes.sort();
            current.usage_classes.dedup();
            current.campaign_ids.extend(usage.campaign_ids);
            current.campaign_ids.sort();
            current.campaign_ids.dedup();
            current.model_version_ids.extend(usage.model_version_ids);
            current.model_version_ids.sort();
            current.model_version_ids.dedup();
            current.labels_accessed |= usage.labels_accessed;
            current.counterfactual_accessed |= usage.counterfactual_accessed;
        } else {
            normalized.push(usage);
        }
    }
    Ok(normalized)
}

fn usage_ledger_material(
    symbol: &str,
    snapshot_ids: &[String],
    usages: &[HistoricalEvidenceUsageRecordV0],
) -> String {
    format!(
        "{}:{}:{}",
        symbol,
        snapshot_ids.join(","),
        usages
            .iter()
            .map(|usage| {
                format!(
                    "{}-{}:{:?}:{}:{}:{}:{}",
                    usage.range.start_timestamp_ms,
                    usage.range.end_timestamp_ms,
                    usage.usage_classes,
                    usage.campaign_ids.join(","),
                    usage.model_version_ids.join(","),
                    usage.labels_accessed,
                    usage.counterfactual_accessed,
                )
            })
            .collect::<Vec<_>>()
            .join("|"),
    )
}

fn snapshot_for_regime(
    snapshot: &DataSnapshot,
    regime: &BtcHistoricalRegimeV0,
) -> Result<DataSnapshot, HistoricalEvidenceErrorV0> {
    let rows = snapshot.normalized_dataset.rows
        [regime.start_row_index..regime.end_row_index_exclusive]
        .to_vec();
    let dataset = HistoricalReplayDataset {
        symbol: snapshot.normalized_dataset.symbol.clone(),
        source: snapshot.normalized_dataset.source.clone(),
        rows,
        reason_codes: snapshot.normalized_dataset.reason_codes.clone(),
    };
    let digest = historical_replay_dataset_digest_v0(&dataset);
    let mut regime_snapshot = snapshot.clone();
    regime_snapshot.snapshot_id = snapshot_id_from_semantic_digest_v1(&digest);
    regime_snapshot.request_key = format!(
        "{}:regime:{}",
        snapshot.request_key, regime.segmentation_config_digest
    );
    regime_snapshot.requested_lookback.bars = dataset.rows.len();
    regime_snapshot.actual_start_timestamp_ms = dataset.rows.first().map(|row| row.timestamp_ms);
    regime_snapshot.actual_end_timestamp_ms = dataset.rows.last().map(|row| row.timestamp_ms);
    regime_snapshot.row_count = dataset.rows.len();
    regime_snapshot.quality_summary.row_count = regime_snapshot.row_count;
    regime_snapshot.content_digest = digest;
    regime_snapshot.normalized_dataset = dataset;
    if !snapshot_rows_are_valid(&regime_snapshot) {
        return Err(HistoricalEvidenceErrorV0::InvalidConfig);
    }
    Ok(regime_snapshot)
}

fn regime_evidence_from_report(
    regime: &BtcHistoricalRegimeV0,
    report: &MomentumTemporalDiagnosticReportV0,
    support_traces: Vec<MomentumSupportTraceV0>,
    campaign_config_digest: String,
    encoder_parameter_digest: String,
) -> BtcTemporalRegimeEvidenceResultV0 {
    BtcTemporalRegimeEvidenceResultV0 {
        regime_id: regime.regime_id.clone(),
        row_count: regime.row_count,
        campaign_windows: report.aggregate.total_windows,
        no_signal_windows: report.aggregate.no_signal_windows,
        selected_checkpoint_windows: report.aggregate.selected_checkpoint_windows,
        in_support_windows: report.aggregate.in_support_windows,
        out_of_support_windows: report.aggregate.out_of_support_windows,
        support_unavailable_windows: report.aggregate.support_gate_unavailable_windows,
        validation_in_support_windows: report.aggregate.validation_in_support_windows,
        validation_out_of_support_windows: report.aggregate.validation_out_of_support_windows,
        validation_insufficient_windows: report.aggregate.validation_insufficient_windows,
        validation_gate_unavailable_windows: report.aggregate.validation_gate_unavailable_windows,
        support_traces,
        earliest_shift_stage: report.earliest_shift_stage,
        temporal_root_cause: report.temporal_root_cause,
        frozen_representation_breach_count: report.aggregate.representation_shift_windows,
        warm_start_status: report.warm_start_status,
        abstention_count: report.aggregate.operational_abstentions,
        accepted_predictive_versions: report.aggregate.accepted_predictive_versions,
        final_verdict: report.final_verdict,
        reason_codes: report.reason_codes.clone(),
        campaign_config_digest,
        encoder_parameter_digest,
        report_digest: report.report_digest.clone(),
    }
}

fn holdout_manifest_digest(manifest: &ProspectiveHoldoutManifestV0) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}:{:?}:{}:{}",
        manifest.manifest_version,
        manifest.series_id,
        manifest.cutoff_exclusive_timestamp_ms,
        manifest.minimum_future_rows,
        manifest.required_future_windows,
        manifest.policy_config_digest,
        manifest.status,
        manifest.opened,
        manifest.labels_accessed,
    ))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProspectiveChallengeStatusV0 {
    Draft,
    Validated,
    Sealed,
    PreRegistrationCommitted,
    AwaitingFutureRows,
    Accumulating,
    AwaitingMaturity,
    ReadyForOneTimeEvaluation,
    OpenedForOneTimeEvaluation,
    Evaluated,
    Closed,
    Invalidated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProspectiveChallengeInvalidationReasonV0 {
    CandidateArtifactChanged,
    CandidateDigestMismatch,
    ComparatorChanged,
    FeatureConfigChanged,
    LabelPolicyChanged,
    SupportPolicyChanged,
    CollapsePolicyChanged,
    MetricPolicyChanged,
    EvaluationRuleChanged,
    FutureEvidenceAccessedBeforeSeal,
    FutureLabelsAccessedEarly,
    PredictionJournalModified,
    EvidenceVaultModified,
    DuplicateEvaluationAttempt,
    TechnicalIntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProspectiveRegistryTransitionReasonV0 {
    PreRegistrationCommit,
    FutureRowsUnavailable,
    FutureRowAdmitted,
    FirstEventSealed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProspectiveRegistryTransitionRecordV0 {
    pub challenge_id: String,
    pub previous_state: ProspectiveChallengeStatusV0,
    pub next_state: ProspectiveChallengeStatusV0,
    pub previous_registry_digest: String,
    pub next_registry_digest: String,
    pub capsule_digest: String,
    pub vault_digest_before: String,
    pub vault_digest_after: String,
    pub journal_digest_before: String,
    pub journal_digest_after: String,
    pub transition_reason: ProspectiveRegistryTransitionReasonV0,
    pub evidence_access_occurred: bool,
    pub labels_accessed: bool,
    pub transition_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegistryDigestProvenanceStatusV0 {
    ValidSingleState,
    ValidTransitionChain,
    StaleCommittedReport,
    StaleLocalRegistry,
    UnexpectedDigestChange,
    MissingTransitionRecord,
    CapsuleMismatch,
    RegistryCorruption,
    Ambiguous,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProspectiveRegistryDigestProvenanceV0 {
    pub capsule_digest: String,
    pub initial_sealed_registry_digest: String,
    pub pre_registration_committed_registry_digest: String,
    pub current_registry_digest: String,
    pub transition_record_count: usize,
    pub transition_chain_valid: bool,
    pub semantic_state_changes_explain_digest_changes: bool,
    pub local_state_matches_committed_registration: bool,
    pub status: RegistryDigestProvenanceStatusV0,
    pub provenance_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProspectiveChallengeFreezeFieldV0 {
    Candidate,
    Comparators,
    Cutoff,
    FeaturePolicy,
    LabelPolicy,
    SupportPolicy,
    CollapsePolicy,
    EvaluationPolicy,
    OpeningPolicy,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProspectiveChallengeFreezeProofV0 {
    pub candidate_unchanged: bool,
    pub comparators_unchanged: bool,
    pub cutoff_unchanged: bool,
    pub feature_policy_unchanged: bool,
    pub label_policy_unchanged: bool,
    pub support_policy_unchanged: bool,
    pub collapse_policy_unchanged: bool,
    pub evaluation_policy_unchanged: bool,
    pub opening_policy_unchanged: bool,
    pub all_equal: bool,
    pub mismatch_fields: Vec<ProspectiveChallengeFreezeFieldV0>,
    pub proof_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProspectiveEvidenceUsageClassV0 {
    VaultedUnopened,
    CausalContextForSealedPrediction,
    UsedForSealedPrediction,
    AwaitingLabelMaturity,
    MatureLabelSealed,
    OpenedForOneTimeEvaluation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProspectiveCausalContextBridgeV0 {
    pub challenge_id: String,
    pub historical_context_snapshot_id: String,
    pub historical_context_range_digest: String,
    pub future_vault_digest: String,
    pub required_feature_lookback_rows: usize,
    pub required_sequence_rows: usize,
    pub context_rows_used: usize,
    pub future_rows_used: usize,
    pub latest_input_timestamp_ms: u64,
    pub target_prediction_timestamp_ms: u64,
    pub bridge_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ProspectiveComparatorKindV0 {
    FrozenLinear,
    FrozenTrainingPrevalenceConstant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProspectiveShadowOutcomeV0 {
    ShadowPredictionSealed,
    ShadowAbstainOutOfSupport,
    ShadowAbstainSupportUnavailable,
    ShadowAbstainInsufficientHistory,
    ShadowAbstainNumericalFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProspectiveLabelStatusV0 {
    AwaitingFutureRows,
    MatureButSealed,
    OpenedForOneTimeEvaluation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProspectiveShadowChallengeVerdictV0 {
    MambaHelpedProspectively,
    LinearStrongerProspectively,
    ConstantBaselineNotBeaten,
    MixedProspectiveEvidence,
    PredominantlyAbstained,
    TemporalSupportFailure,
    ProbabilityCollapse,
    InsufficientMatureEvidence,
    TechnicalEvaluationFailure,
    ChallengeInvalidated,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrozenProspectiveMambaCandidateV0 {
    pub source_campaign_id: String,
    pub source_regime_id: String,
    pub source_window_id: String,
    pub source_candidate_id: String,
    pub checkpoint_epoch: usize,
    pub model_version_id: String,
    pub encoder_digest: String,
    pub representation_normalizer_digest: String,
    pub head_digest: String,
    pub feature_config_digest: String,
    pub sequence_config_digest: String,
    pub label_config_digest: String,
    pub support_policy_digest: String,
    pub collapse_policy_digest: String,
    pub backend_policy_digest: String,
    pub shadow_only: bool,
    pub head_weight_bits: Vec<u32>,
    pub head_bias_bits: u32,
    pub artifact_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrozenProspectiveComparatorV0 {
    pub kind: ProspectiveComparatorKindV0,
    pub source_training_digest: String,
    pub parameter_digest: String,
    pub feature_config_digest: String,
    pub label_config_digest: String,
    pub parameter_bits: Vec<u32>,
    pub artifact_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProspectiveEvidencePolicyV0 {
    pub finalized_daily_rows_only: bool,
    pub maximum_requests: usize,
    pub maximum_concurrency: usize,
    pub retry_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProspectivePredictionPolicyV0 {
    pub support_required: bool,
    pub hide_probabilities_before_opening: bool,
    pub hide_labels_before_opening: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProspectiveEvaluationPolicyV0 {
    pub minimum_finalized_future_rows: usize,
    pub minimum_mature_events: usize,
    pub minimum_support_qualified_events: usize,
    pub minimum_positive_labels: usize,
    pub minimum_negative_labels_for_auc: usize,
    pub maximum_technical_failure_count: usize,
    pub primary_metric: String,
    pub secondary_metrics: Vec<String>,
    pub require_brier_improvement_over_linear: bool,
    pub require_brier_improvement_over_constant: bool,
    pub require_no_severe_probability_collapse: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OneTimeOpeningPolicyV0 {
    pub minimum_mature_events: usize,
    pub minimum_support_qualified_events: usize,
    pub explicit_open_flag_required: bool,
    pub challenge_must_be_sealed: bool,
    pub capsule_digest_must_match: bool,
    pub vault_digest_must_match: bool,
    pub journal_digest_must_match: bool,
    pub second_open_forbidden: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProspectiveShadowChallengeCapsuleV0 {
    pub capsule_version: String,
    pub challenge_id: String,
    pub series_id: String,
    pub prospective_cutoff_exclusive_timestamp_ms: u64,
    pub candidate: FrozenProspectiveMambaCandidateV0,
    pub comparators: Vec<FrozenProspectiveComparatorV0>,
    pub support_policy_digest: String,
    pub feature_policy_digest: String,
    pub label_policy_digest: String,
    pub prediction_horizon: usize,
    pub evidence_policy: ProspectiveEvidencePolicyV0,
    pub prediction_policy: ProspectivePredictionPolicyV0,
    pub evaluation_policy: ProspectiveEvaluationPolicyV0,
    pub opening_policy: OneTimeOpeningPolicyV0,
    pub status: ProspectiveChallengeStatusV0,
    pub capsule_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProspectiveEvidenceRowRefV0 {
    pub timestamp_ms: u64,
    pub canonical_row_digest: String,
    pub finalized: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProspectiveEvidenceVaultV0 {
    pub vault_version: String,
    pub challenge_id: String,
    pub series_id: String,
    pub cutoff_exclusive_timestamp_ms: u64,
    pub finalized_rows: Vec<ProspectiveEvidenceRowRefV0>,
    pub first_timestamp_ms: Option<u64>,
    pub last_timestamp_ms: Option<u64>,
    pub finalized_row_count: usize,
    pub labels_derived: bool,
    pub opened: bool,
    pub vault_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SealedProbabilityV0 {
    pub ieee754_bits: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SealedComparatorPredictionV0 {
    pub comparator_artifact_digest: String,
    pub probability: SealedProbabilityV0,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProspectivePredictionEventV0 {
    pub challenge_id: String,
    pub event_id: String,
    pub prediction_timestamp_ms: u64,
    pub required_label_maturity_timestamp_ms: u64,
    pub input_evidence_digest: String,
    pub candidate_artifact_digest: String,
    pub comparator_artifact_digests: Vec<String>,
    pub support_applicability: String,
    pub support_decision: String,
    pub candidate_prediction: Option<SealedProbabilityV0>,
    pub comparator_predictions: Vec<SealedComparatorPredictionV0>,
    pub operational_outcome: ProspectiveShadowOutcomeV0,
    pub label_status: ProspectiveLabelStatusV0,
    pub event_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProspectivePredictionJournalV0 {
    pub journal_version: String,
    pub challenge_id: String,
    pub events: Vec<ProspectivePredictionEventV0>,
    pub journal_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProspectiveChallengeRegistryEntryV0 {
    pub challenge_id: String,
    pub capsule_digest: String,
    pub candidate_digest: String,
    pub comparator_digests: Vec<String>,
    pub cutoff_exclusive_timestamp_ms: u64,
    pub status: ProspectiveChallengeStatusV0,
    pub invalidation_reason: Option<ProspectiveChallengeInvalidationReasonV0>,
    pub vault_digest: String,
    pub journal_digest: String,
    pub evaluation_opened: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProspectiveChallengeRegistryV0 {
    pub registry_version: String,
    pub challenges: Vec<ProspectiveChallengeRegistryEntryV0>,
    pub registry_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProspectiveChallengeLocalStateV0 {
    pub capsule: ProspectiveShadowChallengeCapsuleV0,
    pub vault: ProspectiveEvidenceVaultV0,
    pub journal: ProspectivePredictionJournalV0,
    pub registry: ProspectiveChallengeRegistryV0,
    #[serde(default)]
    pub registry_transition_records: Vec<ProspectiveRegistryTransitionRecordV0>,
    #[serde(default)]
    pub blind_acquisition_receipts: Vec<ProspectiveBlindAcquisitionReceiptV0>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlindProspectiveChallengeStatusV0 {
    pub challenge_status: ProspectiveChallengeStatusV0,
    pub finalized_row_count: usize,
    pub eligible_prediction_event_count: usize,
    pub support_qualified_prediction_count: usize,
    pub abstention_count: usize,
    pub awaiting_label_maturity_count: usize,
    pub mature_but_sealed_label_count: usize,
    pub capsule_digest: String,
    pub vault_digest: String,
    pub journal_digest: String,
    pub registry_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProspectiveChallengeErrorV0 {
    CandidateResolutionAmbiguous,
    CandidateResolutionMissing,
    InvalidCapsule,
    InvalidTransition,
    InvalidVault,
    InvalidJournal,
    IntegrityMismatch,
    EvaluationOpeningForbidden,
    Storage,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProspectiveCandidateResolutionV0 {
    pub candidate: FrozenProspectiveMambaCandidateV0,
    pub linear: FrozenProspectiveComparatorV0,
    pub constant: FrozenProspectiveComparatorV0,
}

fn forensic_candidate_id_v0(candidate: MomentumForensicCandidateV0) -> &'static str {
    match candidate {
        MomentumForensicCandidateV0::C0Reference => "c0_reference",
        MomentumForensicCandidateV0::C1RepresentationNormalized => "c1_representation_normalized",
        MomentumForensicCandidateV0::C2PrevalenceBias => "c2_prevalence_bias",
        MomentumForensicCandidateV0::C3Combined => "c3_combined",
    }
}

fn support_policy_digest_v0(config: &MomentumLearningCampaignConfigV0) -> String {
    let gate = &config.support_gate;
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}:{}",
        gate.minimum_samples,
        gate.maximum_mean_standardized_shift.to_bits(),
        gate.maximum_dimension_standardized_shift.to_bits(),
        gate.maximum_mean_log_variance_ratio.to_bits(),
        gate.maximum_out_of_support_fraction.to_bits(),
        gate.minimum_validation_coverage.to_bits(),
        gate.comparison_epsilon.to_bits(),
    ))
}

fn collapse_policy_digest_v0(config: &MomentumLearningCampaignConfigV0) -> String {
    let policy = &config.collapse_config;
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}",
        policy.minimum_probability_stddev.to_bits(),
        policy.minimum_prediction_entropy.to_bits(),
        policy.maximum_single_side_fraction.to_bits(),
        policy.low_saturation_threshold.to_bits(),
        policy.high_saturation_threshold.to_bits(),
        policy.maximum_saturation_fraction.to_bits(),
        policy.minimum_unique_probability_bins,
        policy.comparison_epsilon.to_bits(),
        policy.minimum_samples,
    ))
}

fn sequence_policy_digest_v0(config: &MomentumLearningCampaignConfigV0) -> String {
    let policy = &config.sequence_config;
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}",
        policy.sequence_length,
        policy.prediction_horizon,
        policy.label_dead_zone.to_bits(),
        policy.stride,
        policy.include_neutral_labels,
    ))
}

fn backend_policy_digest_v0(config: &MomentumLearningCampaignConfigV0) -> String {
    let preference = match config.backend_preference {
        super::BackendPreference::Auto => "auto",
        super::BackendPreference::Cpu => "cpu",
        super::BackendPreference::Metal => "metal",
        super::BackendPreference::Cuda => "cuda",
    };
    let fallback = match config.fallback_policy {
        super::BackendFallbackPolicy::AllowCpuFallback => "allow_cpu_fallback",
        super::BackendFallbackPolicy::Strict => "strict",
    };
    stable_hash_string(&format!("{preference}:{fallback}"))
}

fn candidate_artifact_digest_v0(candidate: &FrozenProspectiveMambaCandidateV0) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        candidate.source_campaign_id,
        candidate.source_regime_id,
        candidate.source_window_id,
        candidate.source_candidate_id,
        candidate.checkpoint_epoch,
        candidate.model_version_id,
        candidate.encoder_digest,
        candidate.representation_normalizer_digest,
        candidate.head_digest,
        candidate.feature_config_digest,
        candidate.sequence_config_digest,
        candidate.label_config_digest,
        candidate.support_policy_digest,
        candidate.collapse_policy_digest,
        candidate.backend_policy_digest,
        candidate.shadow_only,
        candidate
            .head_weight_bits
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(","),
        candidate.head_bias_bits,
    ))
}

fn comparator_artifact_digest_v0(comparator: &FrozenProspectiveComparatorV0) -> String {
    let kind = match comparator.kind {
        ProspectiveComparatorKindV0::FrozenLinear => "frozen_linear",
        ProspectiveComparatorKindV0::FrozenTrainingPrevalenceConstant => {
            "frozen_training_prevalence_constant"
        }
    };
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}",
        kind,
        comparator.source_training_digest,
        comparator.parameter_digest,
        comparator.feature_config_digest,
        comparator.label_config_digest,
        comparator
            .parameter_bits
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(","),
    ))
}

fn frozen_linear_comparator_v0(
    linear: &LinearMomentumBaselineV0,
    source_training_digest: String,
    feature_config_digest: String,
    label_config_digest: String,
) -> FrozenProspectiveComparatorV0 {
    let mut comparator = FrozenProspectiveComparatorV0 {
        kind: ProspectiveComparatorKindV0::FrozenLinear,
        source_training_digest,
        parameter_digest: linear.head.parameter_digest(),
        feature_config_digest,
        label_config_digest,
        parameter_bits: linear
            .head
            .weights
            .iter()
            .map(|value| value.to_bits())
            .chain(std::iter::once(linear.head.bias.to_bits()))
            .collect(),
        artifact_digest: String::new(),
    };
    comparator.artifact_digest = comparator_artifact_digest_v0(&comparator);
    comparator
}

fn frozen_constant_comparator_v0(
    constant: &ConstantProbabilityBaselineV0,
    source_training_digest: String,
    feature_config_digest: String,
    label_config_digest: String,
) -> FrozenProspectiveComparatorV0 {
    let mut comparator = FrozenProspectiveComparatorV0 {
        kind: ProspectiveComparatorKindV0::FrozenTrainingPrevalenceConstant,
        source_training_digest,
        parameter_digest: stable_hash_string(&constant.probability.to_bits().to_string()),
        feature_config_digest,
        label_config_digest,
        parameter_bits: vec![constant.probability.to_bits()],
        artifact_digest: String::new(),
    };
    comparator.artifact_digest = comparator_artifact_digest_v0(&comparator);
    comparator
}

pub fn resolve_btc_prospective_candidate_v0(
    packs: &[(BtcHistoricalRegimeV0, MomentumHistoricalEvidencePackV0)],
    campaign_config: &MomentumLearningCampaignConfigV0,
    encoder: &FrozenMamba3EncoderV0,
) -> Result<ProspectiveCandidateResolutionV0, ProspectiveChallengeErrorV0> {
    let mut ordered = packs.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| {
        left.0
            .start_timestamp_ms
            .cmp(&right.0.start_timestamp_ms)
            .then_with(|| left.0.regime_id.cmp(&right.0.regime_id))
    });
    let mut resolutions = Vec::new();
    for (regime, pack) in ordered {
        let campaign = run_momentum_series_campaigns_v0(pack, campaign_config, encoder)
            .map_err(|_| ProspectiveChallengeErrorV0::CandidateResolutionMissing)?
            .into_iter()
            .next()
            .ok_or(ProspectiveChallengeErrorV0::CandidateResolutionMissing)?
            .campaign;
        for forensic in &campaign.collapse_forensics {
            let Some(temporal) = &forensic.temporal_generalization else {
                continue;
            };
            if temporal.validation_support_decision != ShadowSupportDecisionV0::InSupport
                || temporal.test_support_decision != ShadowSupportDecisionV0::InSupport
            {
                continue;
            }
            let (Some(selected_candidate), Some(checkpoint)) = (
                forensic.selected_candidate,
                forensic.selected_checkpoint.as_ref(),
            ) else {
                continue;
            };
            let selected = forensic
                .candidate_results
                .iter()
                .find(|result| result.candidate == selected_candidate)
                .ok_or(ProspectiveChallengeErrorV0::CandidateResolutionMissing)?;
            let window = campaign
                .windows
                .iter()
                .find(|window| window.window.window_id == forensic.window_id)
                .ok_or(ProspectiveChallengeErrorV0::CandidateResolutionMissing)?;
            let path = window
                .paths
                .iter()
                .find(|path| path.path == super::MomentumLearningPathV0::Cold)
                .or_else(|| window.paths.first())
                .ok_or(ProspectiveChallengeErrorV0::CandidateResolutionMissing)?;
            let source_training_digest = stable_hash_string(&format!(
                "{}:{}:{}:{}:{}",
                pack.digest,
                path.version.train_range.start,
                path.version.train_range.end,
                path.version.validation_range.start,
                path.version.validation_range.end,
            ));
            let source_candidate_id = forensic_candidate_id_v0(selected_candidate).to_string();
            let head_digest = selected.frozen_head.parameter_digest();
            let model_version_id = format!(
                "frozen-prospective-forensic-{}",
                stable_hash_string(&format!(
                    "{}:{}:{}:{}:{}",
                    campaign.campaign_id,
                    regime.regime_id,
                    forensic.window_id,
                    source_candidate_id,
                    head_digest,
                ))
            );
            let mut candidate = FrozenProspectiveMambaCandidateV0 {
                source_campaign_id: campaign.campaign_id.clone(),
                source_regime_id: regime.regime_id.clone(),
                source_window_id: forensic.window_id.clone(),
                source_candidate_id,
                checkpoint_epoch: checkpoint.epoch,
                model_version_id,
                encoder_digest: encoder.parameter_digest(),
                representation_normalizer_digest: forensic.representation_normalizer_digest.clone(),
                head_digest,
                feature_config_digest: window.feature_config_digest.clone(),
                sequence_config_digest: sequence_policy_digest_v0(campaign_config),
                label_config_digest: sequence_policy_digest_v0(campaign_config),
                support_policy_digest: support_policy_digest_v0(campaign_config),
                collapse_policy_digest: collapse_policy_digest_v0(campaign_config),
                backend_policy_digest: backend_policy_digest_v0(campaign_config),
                shadow_only: path.version.deployment_status
                    == ModelAgentDeploymentStatus::ShadowOnly,
                head_weight_bits: selected
                    .frozen_head
                    .weights
                    .iter()
                    .map(|value| value.to_bits())
                    .collect(),
                head_bias_bits: selected.frozen_head.bias.to_bits(),
                artifact_digest: String::new(),
            };
            candidate.artifact_digest = candidate_artifact_digest_v0(&candidate);
            if !candidate.shadow_only {
                return Err(ProspectiveChallengeErrorV0::CandidateResolutionMissing);
            }
            let linear = frozen_linear_comparator_v0(
                &path.frozen_linear_comparator,
                source_training_digest.clone(),
                candidate.feature_config_digest.clone(),
                candidate.label_config_digest.clone(),
            );
            let constant = frozen_constant_comparator_v0(
                &path.frozen_constant_comparator,
                source_training_digest,
                candidate.feature_config_digest.clone(),
                candidate.label_config_digest.clone(),
            );
            resolutions.push(ProspectiveCandidateResolutionV0 {
                candidate,
                linear,
                constant,
            });
        }
    }
    match resolutions.len() {
        0 => Err(ProspectiveChallengeErrorV0::CandidateResolutionMissing),
        1 => Ok(resolutions.remove(0)),
        _ => Err(ProspectiveChallengeErrorV0::CandidateResolutionAmbiguous),
    }
}

pub fn prospective_evaluation_policy_from_campaign_v0(
    config: &MomentumLearningCampaignConfigV0,
) -> ProspectiveEvaluationPolicyV0 {
    ProspectiveEvaluationPolicyV0 {
        minimum_finalized_future_rows: config.sequence_config.sequence_length
            + config.sequence_config.prediction_horizon
            + config.minimum_test_samples,
        minimum_mature_events: config.minimum_test_samples,
        minimum_support_qualified_events: config.minimum_test_samples,
        minimum_positive_labels: 1,
        minimum_negative_labels_for_auc: 1,
        maximum_technical_failure_count: 0,
        primary_metric: "brier_score_support_qualified_events".to_string(),
        secondary_metrics: vec![
            "reliability".to_string(),
            "resolution".to_string(),
            "uncertainty".to_string(),
            "binary_rank_auc_when_both_classes_exist".to_string(),
            "high_confidence_error_count".to_string(),
            "support_qualified_coverage".to_string(),
            "abstention_rate".to_string(),
            "test_time_out_of_support_rate".to_string(),
            "probability_diversity".to_string(),
            "collapse_status".to_string(),
        ],
        require_brier_improvement_over_linear: true,
        require_brier_improvement_over_constant: true,
        require_no_severe_probability_collapse: true,
    }
}

fn evaluation_policy_digest_v0(policy: &ProspectiveEvaluationPolicyV0) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        policy.minimum_finalized_future_rows,
        policy.minimum_mature_events,
        policy.minimum_support_qualified_events,
        policy.minimum_positive_labels,
        policy.minimum_negative_labels_for_auc,
        policy.maximum_technical_failure_count,
        policy.primary_metric,
        policy.secondary_metrics.join(","),
        policy.require_brier_improvement_over_linear,
        policy.require_brier_improvement_over_constant,
        policy.require_no_severe_probability_collapse,
    ))
}

fn opening_policy_digest_v0(policy: &OneTimeOpeningPolicyV0) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}:{}:{}",
        policy.minimum_mature_events,
        policy.minimum_support_qualified_events,
        policy.explicit_open_flag_required,
        policy.challenge_must_be_sealed,
        policy.capsule_digest_must_match,
        policy.vault_digest_must_match,
        policy.journal_digest_must_match,
        policy.second_open_forbidden,
    ))
}

fn evidence_policy_digest_v0(policy: &ProspectiveEvidencePolicyV0) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}",
        policy.finalized_daily_rows_only,
        policy.maximum_requests,
        policy.maximum_concurrency,
        policy.retry_count,
    ))
}

fn prediction_policy_digest_v0(policy: &ProspectivePredictionPolicyV0) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}",
        policy.support_required,
        policy.hide_probabilities_before_opening,
        policy.hide_labels_before_opening,
    ))
}

fn capsule_digest_v0(capsule: &ProspectiveShadowChallengeCapsuleV0) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        capsule.capsule_version,
        capsule.challenge_id,
        capsule.series_id,
        capsule.prospective_cutoff_exclusive_timestamp_ms,
        capsule.candidate.artifact_digest,
        capsule
            .comparators
            .iter()
            .map(|comparator| comparator.artifact_digest.as_str())
            .collect::<Vec<_>>()
            .join(","),
        capsule.support_policy_digest,
        capsule.feature_policy_digest,
        capsule.label_policy_digest,
        capsule.prediction_horizon,
        evidence_policy_digest_v0(&capsule.evidence_policy),
        prediction_policy_digest_v0(&capsule.prediction_policy),
        evaluation_policy_digest_v0(&capsule.evaluation_policy),
        opening_policy_digest_v0(&capsule.opening_policy),
        "sealed",
        capsule.capsule_version,
    ))
}

pub fn build_prospective_shadow_challenge_capsule_v0(
    series_id: String,
    cutoff_exclusive_timestamp_ms: u64,
    resolution: ProspectiveCandidateResolutionV0,
    prediction_horizon: usize,
    evaluation_policy: ProspectiveEvaluationPolicyV0,
) -> Result<ProspectiveShadowChallengeCapsuleV0, ProspectiveChallengeErrorV0> {
    if series_id.is_empty()
        || cutoff_exclusive_timestamp_ms == 0
        || prediction_horizon == 0
        || evaluation_policy.minimum_finalized_future_rows == 0
        || evaluation_policy.minimum_mature_events == 0
        || evaluation_policy.minimum_support_qualified_events == 0
        || !resolution.candidate.shadow_only
    {
        return Err(ProspectiveChallengeErrorV0::InvalidCapsule);
    }
    let mut comparators = vec![resolution.linear, resolution.constant];
    comparators.sort_by_key(|comparator| comparator.kind);
    if comparators.len() != 2
        || comparators[0].kind == comparators[1].kind
        || comparators
            .iter()
            .any(|comparator| comparator.artifact_digest.is_empty())
    {
        return Err(ProspectiveChallengeErrorV0::InvalidCapsule);
    }
    let challenge_id = format!(
        "btc-prospective-{}",
        stable_hash_string(&format!(
            "{}:{}:{}",
            series_id, cutoff_exclusive_timestamp_ms, resolution.candidate.artifact_digest
        ))
    );
    let mut capsule = ProspectiveShadowChallengeCapsuleV0 {
        capsule_version: "prospective-shadow-challenge-v0".to_string(),
        challenge_id,
        series_id,
        prospective_cutoff_exclusive_timestamp_ms: cutoff_exclusive_timestamp_ms,
        support_policy_digest: resolution.candidate.support_policy_digest.clone(),
        feature_policy_digest: resolution.candidate.feature_config_digest.clone(),
        label_policy_digest: resolution.candidate.label_config_digest.clone(),
        prediction_horizon,
        candidate: resolution.candidate,
        comparators,
        evidence_policy: ProspectiveEvidencePolicyV0 {
            finalized_daily_rows_only: true,
            maximum_requests: 1,
            maximum_concurrency: 1,
            retry_count: 0,
        },
        prediction_policy: ProspectivePredictionPolicyV0 {
            support_required: true,
            hide_probabilities_before_opening: true,
            hide_labels_before_opening: true,
        },
        opening_policy: OneTimeOpeningPolicyV0 {
            minimum_mature_events: evaluation_policy.minimum_mature_events,
            minimum_support_qualified_events: evaluation_policy.minimum_support_qualified_events,
            explicit_open_flag_required: true,
            challenge_must_be_sealed: true,
            capsule_digest_must_match: true,
            vault_digest_must_match: true,
            journal_digest_must_match: true,
            second_open_forbidden: true,
        },
        evaluation_policy,
        status: ProspectiveChallengeStatusV0::Sealed,
        capsule_digest: String::new(),
    };
    capsule.capsule_digest = capsule_digest_v0(&capsule);
    validate_prospective_shadow_challenge_capsule_v0(&capsule)?;
    Ok(capsule)
}

pub fn validate_prospective_shadow_challenge_capsule_v0(
    capsule: &ProspectiveShadowChallengeCapsuleV0,
) -> Result<(), ProspectiveChallengeErrorV0> {
    if capsule.status != ProspectiveChallengeStatusV0::Sealed
        || capsule.challenge_id.is_empty()
        || capsule.series_id.is_empty()
        || capsule.candidate.artifact_digest != candidate_artifact_digest_v0(&capsule.candidate)
        || capsule.comparators.len() != 2
        || capsule.comparators.iter().any(|comparator| {
            comparator.artifact_digest != comparator_artifact_digest_v0(comparator)
        })
        || capsule
            .comparators
            .windows(2)
            .any(|pair| pair[0].kind >= pair[1].kind)
        || capsule.evidence_policy.maximum_requests != 1
        || capsule.evidence_policy.maximum_concurrency != 1
        || capsule.evidence_policy.retry_count != 0
        || capsule.capsule_digest != capsule_digest_v0(capsule)
    {
        Err(ProspectiveChallengeErrorV0::InvalidCapsule)
    } else {
        Ok(())
    }
}

pub fn prospective_challenge_transition_v0(
    current: ProspectiveChallengeStatusV0,
    next: ProspectiveChallengeStatusV0,
) -> Result<(), ProspectiveChallengeErrorV0> {
    let permitted = matches!(
        (current, next),
        (
            ProspectiveChallengeStatusV0::Draft,
            ProspectiveChallengeStatusV0::Validated
        ) | (
            ProspectiveChallengeStatusV0::Validated,
            ProspectiveChallengeStatusV0::Sealed
        ) | (
            ProspectiveChallengeStatusV0::Sealed,
            ProspectiveChallengeStatusV0::PreRegistrationCommitted
        ) | (
            ProspectiveChallengeStatusV0::PreRegistrationCommitted,
            ProspectiveChallengeStatusV0::AwaitingFutureRows
        ) | (
            ProspectiveChallengeStatusV0::PreRegistrationCommitted,
            ProspectiveChallengeStatusV0::Accumulating
        ) | (
            ProspectiveChallengeStatusV0::AwaitingFutureRows,
            ProspectiveChallengeStatusV0::Accumulating
        ) | (
            ProspectiveChallengeStatusV0::PreRegistrationCommitted,
            ProspectiveChallengeStatusV0::AwaitingMaturity
        ) | (
            ProspectiveChallengeStatusV0::Accumulating,
            ProspectiveChallengeStatusV0::AwaitingMaturity
        ) | (
            ProspectiveChallengeStatusV0::AwaitingMaturity,
            ProspectiveChallengeStatusV0::ReadyForOneTimeEvaluation
        ) | (
            ProspectiveChallengeStatusV0::ReadyForOneTimeEvaluation,
            ProspectiveChallengeStatusV0::OpenedForOneTimeEvaluation
        ) | (
            ProspectiveChallengeStatusV0::OpenedForOneTimeEvaluation,
            ProspectiveChallengeStatusV0::Evaluated
        ) | (
            ProspectiveChallengeStatusV0::Evaluated,
            ProspectiveChallengeStatusV0::Closed
        ) | (_, ProspectiveChallengeStatusV0::Invalidated)
    );
    if permitted {
        Ok(())
    } else {
        Err(ProspectiveChallengeErrorV0::InvalidTransition)
    }
}

fn vault_digest_v0(vault: &ProspectiveEvidenceVaultV0) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        vault.vault_version,
        vault.challenge_id,
        vault.series_id,
        vault.cutoff_exclusive_timestamp_ms,
        vault
            .finalized_rows
            .iter()
            .map(|row| format!(
                "{}:{}:{}",
                row.timestamp_ms, row.canonical_row_digest, row.finalized
            ))
            .collect::<Vec<_>>()
            .join(","),
        vault.first_timestamp_ms.unwrap_or_default(),
        vault.last_timestamp_ms.unwrap_or_default(),
        vault.finalized_row_count,
        vault.labels_derived,
        vault.opened,
    ))
}

fn shadow_outcome_label_v0(outcome: ProspectiveShadowOutcomeV0) -> &'static str {
    match outcome {
        ProspectiveShadowOutcomeV0::ShadowPredictionSealed => "prediction_sealed",
        ProspectiveShadowOutcomeV0::ShadowAbstainOutOfSupport => "abstain_out_of_support",
        ProspectiveShadowOutcomeV0::ShadowAbstainSupportUnavailable => {
            "abstain_support_unavailable"
        }
        ProspectiveShadowOutcomeV0::ShadowAbstainInsufficientHistory => {
            "abstain_insufficient_history"
        }
        ProspectiveShadowOutcomeV0::ShadowAbstainNumericalFailure => "abstain_numerical_failure",
    }
}

fn label_status_label_v0(status: ProspectiveLabelStatusV0) -> &'static str {
    match status {
        ProspectiveLabelStatusV0::AwaitingFutureRows => "awaiting_future_rows",
        ProspectiveLabelStatusV0::MatureButSealed => "mature_but_sealed",
        ProspectiveLabelStatusV0::OpenedForOneTimeEvaluation => "opened_for_one_time_evaluation",
    }
}

fn challenge_status_label_v0(status: ProspectiveChallengeStatusV0) -> &'static str {
    match status {
        ProspectiveChallengeStatusV0::Draft => "draft",
        ProspectiveChallengeStatusV0::Validated => "validated",
        ProspectiveChallengeStatusV0::Sealed => "sealed",
        ProspectiveChallengeStatusV0::PreRegistrationCommitted => "pre_registration_committed",
        ProspectiveChallengeStatusV0::AwaitingFutureRows => "awaiting_future_rows",
        ProspectiveChallengeStatusV0::Accumulating => "accumulating",
        ProspectiveChallengeStatusV0::AwaitingMaturity => "awaiting_maturity",
        ProspectiveChallengeStatusV0::ReadyForOneTimeEvaluation => "ready_for_one_time_evaluation",
        ProspectiveChallengeStatusV0::OpenedForOneTimeEvaluation => {
            "opened_for_one_time_evaluation"
        }
        ProspectiveChallengeStatusV0::Evaluated => "evaluated",
        ProspectiveChallengeStatusV0::Closed => "closed",
        ProspectiveChallengeStatusV0::Invalidated => "invalidated",
    }
}

fn invalidation_label_v0(reason: Option<ProspectiveChallengeInvalidationReasonV0>) -> &'static str {
    match reason {
        None => "none",
        Some(ProspectiveChallengeInvalidationReasonV0::CandidateArtifactChanged) => {
            "candidate_artifact_changed"
        }
        Some(ProspectiveChallengeInvalidationReasonV0::CandidateDigestMismatch) => {
            "candidate_digest_mismatch"
        }
        Some(ProspectiveChallengeInvalidationReasonV0::ComparatorChanged) => "comparator_changed",
        Some(ProspectiveChallengeInvalidationReasonV0::FeatureConfigChanged) => {
            "feature_config_changed"
        }
        Some(ProspectiveChallengeInvalidationReasonV0::LabelPolicyChanged) => {
            "label_policy_changed"
        }
        Some(ProspectiveChallengeInvalidationReasonV0::SupportPolicyChanged) => {
            "support_policy_changed"
        }
        Some(ProspectiveChallengeInvalidationReasonV0::CollapsePolicyChanged) => {
            "collapse_policy_changed"
        }
        Some(ProspectiveChallengeInvalidationReasonV0::MetricPolicyChanged) => {
            "metric_policy_changed"
        }
        Some(ProspectiveChallengeInvalidationReasonV0::EvaluationRuleChanged) => {
            "evaluation_rule_changed"
        }
        Some(ProspectiveChallengeInvalidationReasonV0::FutureEvidenceAccessedBeforeSeal) => {
            "future_evidence_accessed_before_seal"
        }
        Some(ProspectiveChallengeInvalidationReasonV0::FutureLabelsAccessedEarly) => {
            "future_labels_accessed_early"
        }
        Some(ProspectiveChallengeInvalidationReasonV0::PredictionJournalModified) => {
            "prediction_journal_modified"
        }
        Some(ProspectiveChallengeInvalidationReasonV0::EvidenceVaultModified) => {
            "evidence_vault_modified"
        }
        Some(ProspectiveChallengeInvalidationReasonV0::DuplicateEvaluationAttempt) => {
            "duplicate_evaluation_attempt"
        }
        Some(ProspectiveChallengeInvalidationReasonV0::TechnicalIntegrityFailure) => {
            "technical_integrity_failure"
        }
    }
}

fn event_digest_v0(event: &ProspectivePredictionEventV0) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        event.challenge_id,
        event.event_id,
        event.prediction_timestamp_ms,
        event.required_label_maturity_timestamp_ms,
        event.input_evidence_digest,
        event.candidate_artifact_digest,
        event.comparator_artifact_digests.join(","),
        event.support_applicability,
        event.support_decision,
        event
            .candidate_prediction
            .as_ref()
            .map(|value| value.ieee754_bits)
            .unwrap_or_default(),
        event
            .comparator_predictions
            .iter()
            .map(|value| format!(
                "{}:{}",
                value.comparator_artifact_digest, value.probability.ieee754_bits
            ))
            .collect::<Vec<_>>()
            .join(","),
        format!(
            "{}:{}",
            shadow_outcome_label_v0(event.operational_outcome),
            label_status_label_v0(event.label_status)
        ),
    ))
}

fn journal_digest_v0(journal: &ProspectivePredictionJournalV0) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}",
        journal.journal_version,
        journal.challenge_id,
        journal
            .events
            .iter()
            .map(|event| event.event_digest.as_str())
            .collect::<Vec<_>>()
            .join(","),
    ))
}

fn registry_digest_v0(registry: &ProspectiveChallengeRegistryV0) -> String {
    stable_hash_string(&format!(
        "{}:{}",
        registry.registry_version,
        registry
            .challenges
            .iter()
            .map(|entry| format!(
                "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
                entry.challenge_id,
                entry.capsule_digest,
                entry.candidate_digest,
                entry.comparator_digests.join(","),
                entry.cutoff_exclusive_timestamp_ms,
                challenge_status_label_v0(entry.status),
                invalidation_label_v0(entry.invalidation_reason),
                entry.vault_digest,
                entry.journal_digest,
                entry.evaluation_opened,
            ))
            .collect::<Vec<_>>()
            .join("|"),
    ))
}

fn registry_transition_reason_label_v0(
    reason: ProspectiveRegistryTransitionReasonV0,
) -> &'static str {
    match reason {
        ProspectiveRegistryTransitionReasonV0::PreRegistrationCommit => "pre_registration_commit",
        ProspectiveRegistryTransitionReasonV0::FutureRowsUnavailable => "future_rows_unavailable",
        ProspectiveRegistryTransitionReasonV0::FutureRowAdmitted => "future_row_admitted",
        ProspectiveRegistryTransitionReasonV0::FirstEventSealed => "first_event_sealed",
    }
}

fn registry_transition_digest_v0(record: &ProspectiveRegistryTransitionRecordV0) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        record.challenge_id,
        challenge_status_label_v0(record.previous_state),
        challenge_status_label_v0(record.next_state),
        record.previous_registry_digest,
        record.next_registry_digest,
        record.capsule_digest,
        record.vault_digest_before,
        record.vault_digest_after,
        record.journal_digest_before,
        record.journal_digest_after,
        registry_transition_reason_label_v0(record.transition_reason),
        record.evidence_access_occurred,
        record.labels_accessed,
    ))
}

fn registry_with_state_v0(
    state: &ProspectiveChallengeLocalStateV0,
    status: ProspectiveChallengeStatusV0,
) -> Result<ProspectiveChallengeRegistryV0, ProspectiveChallengeErrorV0> {
    let mut registry = state.registry.clone();
    let entry = registry
        .challenges
        .first_mut()
        .ok_or(ProspectiveChallengeErrorV0::IntegrityMismatch)?;
    entry.status = status;
    registry.registry_digest = registry_digest_v0(&registry);
    Ok(registry)
}

fn append_registry_transition_record_v0(
    state: &mut ProspectiveChallengeLocalStateV0,
    previous_state: ProspectiveChallengeStatusV0,
    previous_registry_digest: String,
    vault_digest_before: String,
    journal_digest_before: String,
    transition_reason: ProspectiveRegistryTransitionReasonV0,
    evidence_access_occurred: bool,
) -> Result<(), ProspectiveChallengeErrorV0> {
    let entry = state
        .registry
        .challenges
        .first()
        .ok_or(ProspectiveChallengeErrorV0::IntegrityMismatch)?;
    let mut record = ProspectiveRegistryTransitionRecordV0 {
        challenge_id: state.capsule.challenge_id.clone(),
        previous_state,
        next_state: entry.status,
        previous_registry_digest,
        next_registry_digest: state.registry.registry_digest.clone(),
        capsule_digest: state.capsule.capsule_digest.clone(),
        vault_digest_before,
        vault_digest_after: state.vault.vault_digest.clone(),
        journal_digest_before,
        journal_digest_after: state.journal.journal_digest.clone(),
        transition_reason,
        evidence_access_occurred,
        labels_accessed: false,
        transition_digest: String::new(),
    };
    record.transition_digest = registry_transition_digest_v0(&record);
    if let Some(existing) = state.registry_transition_records.last() {
        if existing.transition_digest == record.transition_digest {
            return Ok(());
        }
    }
    state.registry_transition_records.push(record);
    Ok(())
}

fn expected_initial_registry_digest_v0(
    state: &ProspectiveChallengeLocalStateV0,
) -> Result<String, ProspectiveChallengeErrorV0> {
    Ok(registry_with_state_v0(state, ProspectiveChallengeStatusV0::Sealed)?.registry_digest)
}

fn registry_transition_chain_is_valid_v0(
    state: &ProspectiveChallengeLocalStateV0,
    initial_registry_digest: &str,
) -> bool {
    let Some(entry) = state.registry.challenges.first() else {
        return false;
    };
    let mut previous_state = ProspectiveChallengeStatusV0::Sealed;
    let mut previous_digest = initial_registry_digest;
    let mut previous_vault_digest = state
        .registry_transition_records
        .first()
        .map(|record| record.vault_digest_before.clone())
        .unwrap_or_else(|| state.vault.vault_digest.clone());
    let mut previous_journal_digest = state
        .registry_transition_records
        .first()
        .map(|record| record.journal_digest_before.clone())
        .unwrap_or_else(|| state.journal.journal_digest.clone());
    for record in &state.registry_transition_records {
        if record.challenge_id != state.capsule.challenge_id
            || record.capsule_digest != state.capsule.capsule_digest
            || record.previous_state != previous_state
            || record.previous_registry_digest != previous_digest
            || record.vault_digest_before != previous_vault_digest
            || record.journal_digest_before != previous_journal_digest
            || record.labels_accessed
            || record.transition_digest != registry_transition_digest_v0(record)
            || prospective_challenge_transition_v0(record.previous_state, record.next_state)
                .is_err()
        {
            return false;
        }
        previous_state = record.next_state;
        previous_digest = &record.next_registry_digest;
        previous_vault_digest = record.vault_digest_after.clone();
        previous_journal_digest = record.journal_digest_after.clone();
    }
    previous_state == entry.status
        && previous_digest == state.registry.registry_digest
        && previous_vault_digest == state.vault.vault_digest
        && previous_journal_digest == state.journal.journal_digest
}

pub fn build_prospective_challenge_freeze_proof_v0(
    state: &ProspectiveChallengeLocalStateV0,
    expected_capsule: &ProspectiveShadowChallengeCapsuleV0,
) -> ProspectiveChallengeFreezeProofV0 {
    let capsule = &state.capsule;
    let candidate_unchanged = capsule.candidate == expected_capsule.candidate;
    let comparators_unchanged = capsule.comparators == expected_capsule.comparators;
    let cutoff_unchanged = capsule.prospective_cutoff_exclusive_timestamp_ms
        == expected_capsule.prospective_cutoff_exclusive_timestamp_ms;
    let feature_policy_unchanged =
        capsule.feature_policy_digest == expected_capsule.feature_policy_digest;
    let label_policy_unchanged =
        capsule.label_policy_digest == expected_capsule.label_policy_digest;
    let support_policy_unchanged =
        capsule.support_policy_digest == expected_capsule.support_policy_digest;
    let collapse_policy_unchanged = capsule.candidate.collapse_policy_digest
        == expected_capsule.candidate.collapse_policy_digest;
    let evaluation_policy_unchanged =
        capsule.evaluation_policy == expected_capsule.evaluation_policy;
    let opening_policy_unchanged = capsule.opening_policy == expected_capsule.opening_policy;
    let mut mismatch_fields = Vec::new();
    for (matches, field) in [
        (
            candidate_unchanged,
            ProspectiveChallengeFreezeFieldV0::Candidate,
        ),
        (
            comparators_unchanged,
            ProspectiveChallengeFreezeFieldV0::Comparators,
        ),
        (cutoff_unchanged, ProspectiveChallengeFreezeFieldV0::Cutoff),
        (
            feature_policy_unchanged,
            ProspectiveChallengeFreezeFieldV0::FeaturePolicy,
        ),
        (
            label_policy_unchanged,
            ProspectiveChallengeFreezeFieldV0::LabelPolicy,
        ),
        (
            support_policy_unchanged,
            ProspectiveChallengeFreezeFieldV0::SupportPolicy,
        ),
        (
            collapse_policy_unchanged,
            ProspectiveChallengeFreezeFieldV0::CollapsePolicy,
        ),
        (
            evaluation_policy_unchanged,
            ProspectiveChallengeFreezeFieldV0::EvaluationPolicy,
        ),
        (
            opening_policy_unchanged,
            ProspectiveChallengeFreezeFieldV0::OpeningPolicy,
        ),
    ] {
        if !matches {
            mismatch_fields.push(field);
        }
    }
    let all_equal = mismatch_fields.is_empty()
        && capsule.capsule_digest == expected_capsule.capsule_digest
        && validate_prospective_shadow_challenge_capsule_v0(capsule).is_ok()
        && validate_prospective_shadow_challenge_capsule_v0(expected_capsule).is_ok();
    let proof_digest = stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        capsule.capsule_digest,
        candidate_unchanged,
        comparators_unchanged,
        cutoff_unchanged,
        feature_policy_unchanged,
        label_policy_unchanged,
        support_policy_unchanged,
        collapse_policy_unchanged,
        evaluation_policy_unchanged,
        opening_policy_unchanged,
        mismatch_fields
            .iter()
            .map(|field| format!("{field:?}"))
            .collect::<Vec<_>>()
            .join(","),
    ));
    ProspectiveChallengeFreezeProofV0 {
        candidate_unchanged,
        comparators_unchanged,
        cutoff_unchanged,
        feature_policy_unchanged,
        label_policy_unchanged,
        support_policy_unchanged,
        collapse_policy_unchanged,
        evaluation_policy_unchanged,
        opening_policy_unchanged,
        all_equal,
        mismatch_fields,
        proof_digest,
    }
}

pub fn compute_prospective_registry_digest_provenance_v0(
    state: &ProspectiveChallengeLocalStateV0,
) -> Result<ProspectiveRegistryDigestProvenanceV0, ProspectiveChallengeErrorV0> {
    validate_prospective_challenge_local_state_v0(state)?;
    let initial_sealed_registry_digest = expected_initial_registry_digest_v0(state)?;
    let current_registry_digest = state.registry.registry_digest.clone();
    let committed_digest = state
        .registry_transition_records
        .iter()
        .find(|record| {
            record.previous_state == ProspectiveChallengeStatusV0::Sealed
                && record.next_state == ProspectiveChallengeStatusV0::PreRegistrationCommitted
        })
        .map(|record| record.next_registry_digest.clone())
        .unwrap_or_default();
    let local_state_matches_committed_registration =
        state.registry.challenges.first().is_some_and(|entry| {
            entry.status == ProspectiveChallengeStatusV0::PreRegistrationCommitted
                && state.vault.finalized_row_count == 0
                && state.journal.events.is_empty()
        });
    let transition_chain_valid = !state.registry_transition_records.is_empty()
        && registry_transition_chain_is_valid_v0(state, &initial_sealed_registry_digest);
    let semantic_state_changes_explain_digest_changes = transition_chain_valid
        && committed_digest != initial_sealed_registry_digest
        && current_registry_digest
            == state
                .registry_transition_records
                .last()
                .map(|record| record.next_registry_digest.clone())
                .unwrap_or_default();
    let status = if state.capsule.capsule_digest.is_empty() {
        RegistryDigestProvenanceStatusV0::CapsuleMismatch
    } else if state.registry_transition_records.is_empty() {
        RegistryDigestProvenanceStatusV0::MissingTransitionRecord
    } else if transition_chain_valid && semantic_state_changes_explain_digest_changes {
        RegistryDigestProvenanceStatusV0::ValidTransitionChain
    } else {
        RegistryDigestProvenanceStatusV0::UnexpectedDigestChange
    };
    let provenance_digest = stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}",
        state.capsule.capsule_digest,
        initial_sealed_registry_digest,
        committed_digest,
        current_registry_digest,
        state.registry_transition_records.len(),
        transition_chain_valid,
        semantic_state_changes_explain_digest_changes,
        local_state_matches_committed_registration,
        format!("{status:?}"),
    ));
    Ok(ProspectiveRegistryDigestProvenanceV0 {
        capsule_digest: state.capsule.capsule_digest.clone(),
        initial_sealed_registry_digest,
        pre_registration_committed_registry_digest: committed_digest,
        current_registry_digest,
        transition_record_count: state.registry_transition_records.len(),
        transition_chain_valid,
        semantic_state_changes_explain_digest_changes,
        local_state_matches_committed_registration,
        status,
        provenance_digest,
    })
}

pub fn seal_missing_pre_registration_transition_v0(
    state: &mut ProspectiveChallengeLocalStateV0,
    expected_capsule: &ProspectiveShadowChallengeCapsuleV0,
) -> Result<ProspectiveRegistryDigestProvenanceV0, ProspectiveChallengeErrorV0> {
    validate_prospective_challenge_local_state_v0(state)?;
    let proof = build_prospective_challenge_freeze_proof_v0(state, expected_capsule);
    let entry = state
        .registry
        .challenges
        .first()
        .ok_or(ProspectiveChallengeErrorV0::IntegrityMismatch)?;
    if !proof.all_equal
        || entry.status != ProspectiveChallengeStatusV0::PreRegistrationCommitted
        || !state.vault.finalized_rows.is_empty()
        || !state.journal.events.is_empty()
    {
        return Err(ProspectiveChallengeErrorV0::IntegrityMismatch);
    }
    if state.registry_transition_records.is_empty() {
        let initial_digest = expected_initial_registry_digest_v0(state)?;
        let current_digest = state.registry.registry_digest.clone();
        let mut record = ProspectiveRegistryTransitionRecordV0 {
            challenge_id: state.capsule.challenge_id.clone(),
            previous_state: ProspectiveChallengeStatusV0::Sealed,
            next_state: ProspectiveChallengeStatusV0::PreRegistrationCommitted,
            previous_registry_digest: initial_digest,
            next_registry_digest: current_digest,
            capsule_digest: state.capsule.capsule_digest.clone(),
            vault_digest_before: state.vault.vault_digest.clone(),
            vault_digest_after: state.vault.vault_digest.clone(),
            journal_digest_before: state.journal.journal_digest.clone(),
            journal_digest_after: state.journal.journal_digest.clone(),
            transition_reason: ProspectiveRegistryTransitionReasonV0::PreRegistrationCommit,
            evidence_access_occurred: false,
            labels_accessed: false,
            transition_digest: String::new(),
        };
        record.transition_digest = registry_transition_digest_v0(&record);
        state.registry_transition_records.push(record);
    }
    compute_prospective_registry_digest_provenance_v0(state)
}

pub fn build_prospective_causal_context_bridge_v0(
    state: &ProspectiveChallengeLocalStateV0,
    historical_context_snapshot_id: String,
    historical_context_range_digest: String,
    historical_rows_available: usize,
    future_rows_used: usize,
    target_prediction_timestamp_ms: u64,
) -> Result<ProspectiveCausalContextBridgeV0, ProspectiveChallengeErrorV0> {
    validate_prospective_challenge_local_state_v0(state)?;
    let required_sequence_rows = state.capsule.prediction_horizon;
    let required_feature_lookback_rows = required_sequence_rows.saturating_sub(1);
    let total_required = required_feature_lookback_rows.saturating_add(required_sequence_rows);
    let context_rows_used = historical_rows_available.min(total_required);
    if historical_context_snapshot_id.is_empty()
        || historical_context_range_digest.is_empty()
        || target_prediction_timestamp_ms <= state.vault.cutoff_exclusive_timestamp_ms
        || future_rows_used > state.vault.finalized_row_count
        || context_rows_used.saturating_add(future_rows_used) < total_required
        || state
            .vault
            .finalized_rows
            .iter()
            .skip(future_rows_used)
            .any(|row| row.timestamp_ms <= target_prediction_timestamp_ms)
    {
        return Err(ProspectiveChallengeErrorV0::InvalidVault);
    }
    let latest_input_timestamp_ms = state
        .vault
        .finalized_rows
        .get(future_rows_used.saturating_sub(1))
        .map(|row| row.timestamp_ms)
        .unwrap_or(state.vault.cutoff_exclusive_timestamp_ms);
    if latest_input_timestamp_ms > target_prediction_timestamp_ms {
        return Err(ProspectiveChallengeErrorV0::InvalidVault);
    }
    let bridge_digest = stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}",
        state.capsule.challenge_id,
        historical_context_snapshot_id,
        historical_context_range_digest,
        state.vault.vault_digest,
        required_feature_lookback_rows,
        required_sequence_rows,
        context_rows_used,
        future_rows_used,
        target_prediction_timestamp_ms,
    ));
    Ok(ProspectiveCausalContextBridgeV0 {
        challenge_id: state.capsule.challenge_id.clone(),
        historical_context_snapshot_id,
        historical_context_range_digest,
        future_vault_digest: state.vault.vault_digest.clone(),
        required_feature_lookback_rows,
        required_sequence_rows,
        context_rows_used,
        future_rows_used,
        latest_input_timestamp_ms,
        target_prediction_timestamp_ms,
        bridge_digest,
    })
}

pub fn new_prospective_challenge_local_state_v0(
    capsule: ProspectiveShadowChallengeCapsuleV0,
) -> Result<ProspectiveChallengeLocalStateV0, ProspectiveChallengeErrorV0> {
    validate_prospective_shadow_challenge_capsule_v0(&capsule)?;
    let mut vault = ProspectiveEvidenceVaultV0 {
        vault_version: "prospective-evidence-vault-v0".to_string(),
        challenge_id: capsule.challenge_id.clone(),
        series_id: capsule.series_id.clone(),
        cutoff_exclusive_timestamp_ms: capsule.prospective_cutoff_exclusive_timestamp_ms,
        finalized_rows: vec![],
        first_timestamp_ms: None,
        last_timestamp_ms: None,
        finalized_row_count: 0,
        labels_derived: false,
        opened: false,
        vault_digest: String::new(),
    };
    vault.vault_digest = vault_digest_v0(&vault);
    let mut journal = ProspectivePredictionJournalV0 {
        journal_version: "prospective-prediction-journal-v0".to_string(),
        challenge_id: capsule.challenge_id.clone(),
        events: vec![],
        journal_digest: String::new(),
    };
    journal.journal_digest = journal_digest_v0(&journal);
    let mut registry = ProspectiveChallengeRegistryV0 {
        registry_version: "prospective-challenge-registry-v0".to_string(),
        challenges: vec![ProspectiveChallengeRegistryEntryV0 {
            challenge_id: capsule.challenge_id.clone(),
            capsule_digest: capsule.capsule_digest.clone(),
            candidate_digest: capsule.candidate.artifact_digest.clone(),
            comparator_digests: capsule
                .comparators
                .iter()
                .map(|comparator| comparator.artifact_digest.clone())
                .collect(),
            cutoff_exclusive_timestamp_ms: capsule.prospective_cutoff_exclusive_timestamp_ms,
            status: ProspectiveChallengeStatusV0::Sealed,
            invalidation_reason: None,
            vault_digest: vault.vault_digest.clone(),
            journal_digest: journal.journal_digest.clone(),
            evaluation_opened: false,
        }],
        registry_digest: String::new(),
    };
    registry.registry_digest = registry_digest_v0(&registry);
    Ok(ProspectiveChallengeLocalStateV0 {
        capsule,
        vault,
        journal,
        registry,
        registry_transition_records: vec![],
        blind_acquisition_receipts: vec![],
    })
}

pub fn record_prospective_blind_acquisition_receipt_v0(
    state: &mut ProspectiveChallengeLocalStateV0,
    receipt: ProspectiveBlindAcquisitionReceiptV0,
) -> Result<(), ProspectiveChallengeErrorV0> {
    validate_prospective_challenge_local_state_v0(state)?;
    if receipt.challenge_id != state.capsule.challenge_id
        || receipt.request_count > state.capsule.evidence_policy.maximum_requests
        || receipt.request_count > 1
        || receipt.receipt_digest.is_empty()
        || state
            .blind_acquisition_receipts
            .iter()
            .any(|existing| existing.receipt_digest != receipt.receipt_digest)
    {
        return Err(ProspectiveChallengeErrorV0::IntegrityMismatch);
    }
    if state
        .blind_acquisition_receipts
        .iter()
        .any(|existing| existing.receipt_digest == receipt.receipt_digest)
    {
        return Ok(());
    }
    state.blind_acquisition_receipts.push(receipt);
    Ok(())
}

pub fn append_prospective_vault_row_v0(
    state: &mut ProspectiveChallengeLocalStateV0,
    row: ProspectiveEvidenceRowRefV0,
) -> Result<(), ProspectiveChallengeErrorV0> {
    validate_prospective_challenge_local_state_v0(state)?;
    let entry = state
        .registry
        .challenges
        .first()
        .ok_or(ProspectiveChallengeErrorV0::InvalidVault)?;
    if !matches!(
        entry.status,
        ProspectiveChallengeStatusV0::PreRegistrationCommitted
            | ProspectiveChallengeStatusV0::AwaitingFutureRows
    ) || !row.finalized
        || row.timestamp_ms <= state.vault.cutoff_exclusive_timestamp_ms
        || state
            .vault
            .finalized_rows
            .iter()
            .any(|existing| existing.timestamp_ms >= row.timestamp_ms)
        || row.canonical_row_digest.is_empty()
    {
        return Err(ProspectiveChallengeErrorV0::InvalidVault);
    }
    let previous_state = entry.status;
    let previous_registry_digest = state.registry.registry_digest.clone();
    let previous_vault_digest = state.vault.vault_digest.clone();
    let previous_journal_digest = state.journal.journal_digest.clone();
    state.vault.finalized_rows.push(row);
    state.vault.first_timestamp_ms = state
        .vault
        .finalized_rows
        .first()
        .map(|value| value.timestamp_ms);
    state.vault.last_timestamp_ms = state
        .vault
        .finalized_rows
        .last()
        .map(|value| value.timestamp_ms);
    state.vault.finalized_row_count = state.vault.finalized_rows.len();
    state.vault.vault_digest = vault_digest_v0(&state.vault);
    let entry = state
        .registry
        .challenges
        .first_mut()
        .ok_or(ProspectiveChallengeErrorV0::InvalidVault)?;
    if entry.status != ProspectiveChallengeStatusV0::Accumulating {
        prospective_challenge_transition_v0(
            entry.status,
            ProspectiveChallengeStatusV0::Accumulating,
        )?;
        entry.status = ProspectiveChallengeStatusV0::Accumulating;
    }
    sync_registry_digests_v0(state)?;
    append_registry_transition_record_v0(
        state,
        previous_state,
        previous_registry_digest,
        previous_vault_digest,
        previous_journal_digest,
        ProspectiveRegistryTransitionReasonV0::FutureRowAdmitted,
        true,
    )?;
    Ok(())
}

pub fn append_prospective_prediction_event_v0(
    state: &mut ProspectiveChallengeLocalStateV0,
    mut event: ProspectivePredictionEventV0,
) -> Result<(), ProspectiveChallengeErrorV0> {
    validate_prospective_challenge_local_state_v0(state)?;
    let entry = state
        .registry
        .challenges
        .first()
        .ok_or(ProspectiveChallengeErrorV0::InvalidJournal)?;
    if !matches!(
        entry.status,
        ProspectiveChallengeStatusV0::PreRegistrationCommitted
            | ProspectiveChallengeStatusV0::Accumulating
            | ProspectiveChallengeStatusV0::AwaitingMaturity
    ) || event.challenge_id != state.capsule.challenge_id
        || event.prediction_timestamp_ms <= state.vault.cutoff_exclusive_timestamp_ms
        || event.required_label_maturity_timestamp_ms <= event.prediction_timestamp_ms
        || event.candidate_artifact_digest != state.capsule.candidate.artifact_digest
        || event.comparator_artifact_digests
            != state
                .capsule
                .comparators
                .iter()
                .map(|comparator| comparator.artifact_digest.clone())
                .collect::<Vec<_>>()
        || state
            .vault
            .last_timestamp_ms
            .is_some_and(|timestamp| timestamp >= event.required_label_maturity_timestamp_ms)
        || event.label_status != ProspectiveLabelStatusV0::AwaitingFutureRows
        || (state.capsule.prediction_policy.support_required
            && event.operational_outcome == ProspectiveShadowOutcomeV0::ShadowPredictionSealed
            && (event.support_applicability != "applicable"
                || event.support_decision != "in_support"
                || event.candidate_prediction.is_none()))
    {
        return Err(ProspectiveChallengeErrorV0::InvalidJournal);
    }
    event.comparator_predictions.sort_by(|left, right| {
        left.comparator_artifact_digest
            .cmp(&right.comparator_artifact_digest)
    });
    event.event_digest = event_digest_v0(&event);
    if let Some(existing) = state
        .journal
        .events
        .iter()
        .find(|existing| existing.event_id == event.event_id)
    {
        return if existing.event_digest == event.event_digest {
            Ok(())
        } else {
            Err(ProspectiveChallengeErrorV0::InvalidJournal)
        };
    }
    if state
        .journal
        .events
        .iter()
        .any(|existing| existing.prediction_timestamp_ms == event.prediction_timestamp_ms)
    {
        return Err(ProspectiveChallengeErrorV0::InvalidJournal);
    }
    let previous_state = entry.status;
    let previous_registry_digest = state.registry.registry_digest.clone();
    let previous_vault_digest = state.vault.vault_digest.clone();
    let previous_journal_digest = state.journal.journal_digest.clone();
    state.journal.events.push(event);
    state.journal.events.sort_by(|left, right| {
        left.prediction_timestamp_ms
            .cmp(&right.prediction_timestamp_ms)
    });
    state.journal.journal_digest = journal_digest_v0(&state.journal);
    let entry = state
        .registry
        .challenges
        .first_mut()
        .ok_or(ProspectiveChallengeErrorV0::InvalidJournal)?;
    prospective_challenge_transition_v0(
        entry.status,
        ProspectiveChallengeStatusV0::AwaitingMaturity,
    )?;
    entry.status = ProspectiveChallengeStatusV0::AwaitingMaturity;
    sync_registry_digests_v0(state)?;
    append_registry_transition_record_v0(
        state,
        previous_state,
        previous_registry_digest,
        previous_vault_digest,
        previous_journal_digest,
        ProspectiveRegistryTransitionReasonV0::FirstEventSealed,
        true,
    )?;
    Ok(())
}

pub fn confirm_prospective_pre_registration_v0(
    state: &mut ProspectiveChallengeLocalStateV0,
) -> Result<(), ProspectiveChallengeErrorV0> {
    validate_prospective_challenge_local_state_v0(state)?;
    let previous_registry_digest = state.registry.registry_digest.clone();
    let previous_vault_digest = state.vault.vault_digest.clone();
    let previous_journal_digest = state.journal.journal_digest.clone();
    let entry = state
        .registry
        .challenges
        .first_mut()
        .ok_or(ProspectiveChallengeErrorV0::IntegrityMismatch)?;
    let previous_state = entry.status;
    prospective_challenge_transition_v0(
        entry.status,
        ProspectiveChallengeStatusV0::PreRegistrationCommitted,
    )?;
    entry.status = ProspectiveChallengeStatusV0::PreRegistrationCommitted;
    sync_registry_digests_v0(state)?;
    append_registry_transition_record_v0(
        state,
        previous_state,
        previous_registry_digest,
        previous_vault_digest,
        previous_journal_digest,
        ProspectiveRegistryTransitionReasonV0::PreRegistrationCommit,
        false,
    )
}

pub fn mark_prospective_awaiting_future_rows_v0(
    state: &mut ProspectiveChallengeLocalStateV0,
) -> Result<(), ProspectiveChallengeErrorV0> {
    validate_prospective_challenge_local_state_v0(state)?;
    let previous_registry_digest = state.registry.registry_digest.clone();
    let previous_vault_digest = state.vault.vault_digest.clone();
    let previous_journal_digest = state.journal.journal_digest.clone();
    let entry = state
        .registry
        .challenges
        .first_mut()
        .ok_or(ProspectiveChallengeErrorV0::IntegrityMismatch)?;
    if entry.status == ProspectiveChallengeStatusV0::AwaitingFutureRows {
        return Ok(());
    }
    let previous_state = entry.status;
    prospective_challenge_transition_v0(
        entry.status,
        ProspectiveChallengeStatusV0::AwaitingFutureRows,
    )?;
    entry.status = ProspectiveChallengeStatusV0::AwaitingFutureRows;
    sync_registry_digests_v0(state)?;
    append_registry_transition_record_v0(
        state,
        previous_state,
        previous_registry_digest,
        previous_vault_digest,
        previous_journal_digest,
        ProspectiveRegistryTransitionReasonV0::FutureRowsUnavailable,
        false,
    )
}

pub fn invalidate_prospective_challenge_v0(
    state: &mut ProspectiveChallengeLocalStateV0,
    reason: ProspectiveChallengeInvalidationReasonV0,
) -> Result<(), ProspectiveChallengeErrorV0> {
    validate_prospective_challenge_local_state_v0(state)?;
    let entry = state
        .registry
        .challenges
        .first_mut()
        .ok_or(ProspectiveChallengeErrorV0::IntegrityMismatch)?;
    if entry.status == ProspectiveChallengeStatusV0::Invalidated {
        return Err(ProspectiveChallengeErrorV0::InvalidTransition);
    }
    entry.status = ProspectiveChallengeStatusV0::Invalidated;
    entry.invalidation_reason = Some(reason);
    sync_registry_digests_v0(state)
}

fn sync_registry_digests_v0(
    state: &mut ProspectiveChallengeLocalStateV0,
) -> Result<(), ProspectiveChallengeErrorV0> {
    let entry = state
        .registry
        .challenges
        .first_mut()
        .ok_or(ProspectiveChallengeErrorV0::IntegrityMismatch)?;
    entry.vault_digest = state.vault.vault_digest.clone();
    entry.journal_digest = state.journal.journal_digest.clone();
    state.registry.registry_digest = registry_digest_v0(&state.registry);
    Ok(())
}

pub fn validate_prospective_challenge_local_state_v0(
    state: &ProspectiveChallengeLocalStateV0,
) -> Result<(), ProspectiveChallengeErrorV0> {
    validate_prospective_shadow_challenge_capsule_v0(&state.capsule)?;
    let Some(entry) = state.registry.challenges.first() else {
        return Err(ProspectiveChallengeErrorV0::IntegrityMismatch);
    };
    if state.registry.challenges.len() != 1
        || state.vault.challenge_id != state.capsule.challenge_id
        || state.journal.challenge_id != state.capsule.challenge_id
        || entry.challenge_id != state.capsule.challenge_id
        || entry.capsule_digest != state.capsule.capsule_digest
        || entry.candidate_digest != state.capsule.candidate.artifact_digest
        || entry.comparator_digests
            != state
                .capsule
                .comparators
                .iter()
                .map(|comparator| comparator.artifact_digest.clone())
                .collect::<Vec<_>>()
        || state.vault.vault_digest != vault_digest_v0(&state.vault)
        || state.journal.journal_digest != journal_digest_v0(&state.journal)
        || entry.vault_digest != state.vault.vault_digest
        || entry.journal_digest != state.journal.journal_digest
        || state.registry.registry_digest != registry_digest_v0(&state.registry)
        || state.vault.labels_derived
        || state.vault.opened != entry.evaluation_opened
        || (entry.evaluation_opened
            && entry.status != ProspectiveChallengeStatusV0::OpenedForOneTimeEvaluation)
        || state
            .vault
            .finalized_rows
            .windows(2)
            .any(|pair| pair[0].timestamp_ms >= pair[1].timestamp_ms)
        || state.vault.finalized_rows.iter().any(|row| {
            !row.finalized || row.timestamp_ms <= state.vault.cutoff_exclusive_timestamp_ms
        })
        || state
            .journal
            .events
            .iter()
            .any(|event| event.event_digest != event_digest_v0(event))
        || state.blind_acquisition_receipts.len() > 1
        || state.blind_acquisition_receipts.iter().any(|receipt| {
            receipt.challenge_id != state.capsule.challenge_id
                || !verify_prospective_blind_acquisition_receipt_v0(receipt)
        })
    {
        Err(ProspectiveChallengeErrorV0::IntegrityMismatch)
    } else {
        Ok(())
    }
}

pub fn blind_prospective_challenge_status_v0(
    state: &ProspectiveChallengeLocalStateV0,
) -> Result<BlindProspectiveChallengeStatusV0, ProspectiveChallengeErrorV0> {
    validate_prospective_challenge_local_state_v0(state)?;
    let entry = state
        .registry
        .challenges
        .first()
        .ok_or(ProspectiveChallengeErrorV0::IntegrityMismatch)?;
    Ok(BlindProspectiveChallengeStatusV0 {
        challenge_status: entry.status,
        finalized_row_count: state.vault.finalized_row_count,
        eligible_prediction_event_count: state.journal.events.len(),
        support_qualified_prediction_count: state
            .journal
            .events
            .iter()
            .filter(|event| {
                event.operational_outcome == ProspectiveShadowOutcomeV0::ShadowPredictionSealed
            })
            .count(),
        abstention_count: state
            .journal
            .events
            .iter()
            .filter(|event| {
                event.operational_outcome != ProspectiveShadowOutcomeV0::ShadowPredictionSealed
            })
            .count(),
        awaiting_label_maturity_count: state
            .journal
            .events
            .iter()
            .filter(|event| event.label_status == ProspectiveLabelStatusV0::AwaitingFutureRows)
            .count(),
        mature_but_sealed_label_count: state
            .journal
            .events
            .iter()
            .filter(|event| event.label_status == ProspectiveLabelStatusV0::MatureButSealed)
            .count(),
        capsule_digest: state.capsule.capsule_digest.clone(),
        vault_digest: state.vault.vault_digest.clone(),
        journal_digest: state.journal.journal_digest.clone(),
        registry_digest: state.registry.registry_digest.clone(),
    })
}

pub fn refresh_prospective_label_maturity_v0(
    state: &mut ProspectiveChallengeLocalStateV0,
) -> Result<(), ProspectiveChallengeErrorV0> {
    validate_prospective_challenge_local_state_v0(state)?;
    let last_finalized_timestamp = state.vault.last_timestamp_ms;
    for event in &mut state.journal.events {
        if event.label_status == ProspectiveLabelStatusV0::AwaitingFutureRows
            && last_finalized_timestamp
                .is_some_and(|timestamp| timestamp >= event.required_label_maturity_timestamp_ms)
        {
            event.label_status = ProspectiveLabelStatusV0::MatureButSealed;
            event.event_digest = event_digest_v0(event);
        }
    }
    state.journal.journal_digest = journal_digest_v0(&state.journal);
    sync_registry_digests_v0(state)
}

pub fn open_prospective_challenge_once_v0(
    state: &mut ProspectiveChallengeLocalStateV0,
    explicit_open_flag: bool,
) -> Result<(), ProspectiveChallengeErrorV0> {
    validate_prospective_challenge_local_state_v0(state)?;
    let mature_events = state
        .journal
        .events
        .iter()
        .filter(|event| event.label_status == ProspectiveLabelStatusV0::MatureButSealed)
        .count();
    let support_events = state
        .journal
        .events
        .iter()
        .filter(|event| {
            event.operational_outcome == ProspectiveShadowOutcomeV0::ShadowPredictionSealed
        })
        .count();
    let entry = state
        .registry
        .challenges
        .first_mut()
        .ok_or(ProspectiveChallengeErrorV0::IntegrityMismatch)?;
    if !explicit_open_flag
        || entry.status != ProspectiveChallengeStatusV0::ReadyForOneTimeEvaluation
        || entry.evaluation_opened
        || mature_events < state.capsule.opening_policy.minimum_mature_events
        || support_events
            < state
                .capsule
                .opening_policy
                .minimum_support_qualified_events
    {
        return Err(ProspectiveChallengeErrorV0::EvaluationOpeningForbidden);
    }
    entry.status = ProspectiveChallengeStatusV0::OpenedForOneTimeEvaluation;
    entry.evaluation_opened = true;
    state.vault.opened = true;
    for event in &mut state.journal.events {
        if event.label_status == ProspectiveLabelStatusV0::MatureButSealed {
            event.label_status = ProspectiveLabelStatusV0::OpenedForOneTimeEvaluation;
            event.event_digest = event_digest_v0(event);
        }
    }
    state.journal.journal_digest = journal_digest_v0(&state.journal);
    sync_registry_digests_v0(state)
}

pub fn write_prospective_challenge_local_state_v0(
    path: &Path,
    state: &ProspectiveChallengeLocalStateV0,
) -> Result<(), ProspectiveChallengeErrorV0> {
    validate_prospective_challenge_local_state_v0(state)?;
    let parent = path.parent().ok_or(ProspectiveChallengeErrorV0::Storage)?;
    fs::create_dir_all(parent).map_err(|_| ProspectiveChallengeErrorV0::Storage)?;
    let encoded = serde_json::to_vec(state).map_err(|_| ProspectiveChallengeErrorV0::Storage)?;
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, encoded).map_err(|_| ProspectiveChallengeErrorV0::Storage)?;
    fs::rename(&temporary, path).map_err(|_| ProspectiveChallengeErrorV0::Storage)
}

pub fn read_prospective_challenge_local_state_v0(
    path: &Path,
) -> Result<ProspectiveChallengeLocalStateV0, ProspectiveChallengeErrorV0> {
    let encoded = fs::read(path).map_err(|_| ProspectiveChallengeErrorV0::Storage)?;
    let state = serde_json::from_slice::<ProspectiveChallengeLocalStateV0>(&encoded)
        .map_err(|_| ProspectiveChallengeErrorV0::Storage)?;
    validate_prospective_challenge_local_state_v0(&state)?;
    Ok(state)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BtcProspectiveChallengePreparationV0 {
    pub holdout: ProspectiveHoldoutManifestV0,
    pub capsule: ProspectiveShadowChallengeCapsuleV0,
    pub ledger_digest: String,
    pub regime_pack_count: usize,
}

pub fn prepare_btc_prospective_challenge_v0(
    snapshot: &DataSnapshot,
    campaign_config: &MomentumLearningCampaignConfigV0,
) -> Result<BtcProspectiveChallengePreparationV0, ProspectiveChallengeErrorV0> {
    if snapshot.content_digest != historical_replay_dataset_digest_v0(&snapshot.normalized_dataset)
        || !snapshot_rows_are_valid(snapshot)
    {
        return Err(ProspectiveChallengeErrorV0::IntegrityMismatch);
    }
    let sufficiency = assess_momentum_campaign_sufficiency_v0(snapshot.row_count, campaign_config)
        .map_err(|_| ProspectiveChallengeErrorV0::InvalidCapsule)?;
    if !sufficiency.sufficient {
        return Err(ProspectiveChallengeErrorV0::CandidateResolutionMissing);
    }
    let regime_config = BtcHistoricalRegimeConfigV0 {
        minimum_regimes: 2,
        regime_rows: sufficiency.required_minimum_rows,
        inter_regime_gap_rows: campaign_config.purge_gap_rows,
        minimum_campaign_windows_per_regime: campaign_config.minimum_evaluated_windows,
        segmentation_policy: TemporalRegimeSegmentationPolicyV0::EqualLengthChronological,
    };
    let segmentation = segment_btc_historical_regimes_v0(snapshot, &regime_config)
        .map_err(|_| ProspectiveChallengeErrorV0::IntegrityMismatch)?;
    if segmentation.status != BtcRegimeSegmentationStatusV0::Ready
        || segmentation.regimes.len() != regime_config.minimum_regimes
    {
        return Err(ProspectiveChallengeErrorV0::CandidateResolutionMissing);
    }
    let packs = freeze_btc_historical_regime_packs_v0(
        snapshot,
        &segmentation,
        &HistoricalEvidencePolicyV0::default(),
    )
    .map_err(|_| ProspectiveChallengeErrorV0::IntegrityMismatch)?;
    if packs.len() != segmentation.regimes.len()
        || packs
            .iter()
            .any(|(_, pack)| verify_momentum_historical_evidence_pack_v0(pack).is_err())
    {
        return Err(ProspectiveChallengeErrorV0::IntegrityMismatch);
    }
    let encoder = super::frozen_mamba3_encoder_from_seed_v0(
        &campaign_config.feature_config,
        campaign_config.campaign_seed,
        campaign_config.backend_preference,
        campaign_config.fallback_policy,
    )
    .map_err(|_| ProspectiveChallengeErrorV0::CandidateResolutionMissing)?;
    let resolution = resolve_btc_prospective_candidate_v0(&packs, campaign_config, &encoder)?;
    let ledger = build_historical_evidence_usage_ledger_v0(snapshot, &[])
        .map_err(|_| ProspectiveChallengeErrorV0::IntegrityMismatch)?;
    let evaluation_policy = prospective_evaluation_policy_from_campaign_v0(campaign_config);
    let holdout = seal_prospective_holdout_v0(
        &ledger,
        &ProspectiveHoldoutPolicyConfigV0 {
            minimum_future_rows: evaluation_policy.minimum_finalized_future_rows,
            required_future_windows: campaign_config.minimum_evaluated_windows,
        },
        &[],
    )
    .map_err(|_| ProspectiveChallengeErrorV0::InvalidCapsule)?;
    let capsule = build_prospective_shadow_challenge_capsule_v0(
        ledger.series_id.clone(),
        holdout.cutoff_exclusive_timestamp_ms,
        resolution,
        campaign_config.sequence_config.prediction_horizon,
        evaluation_policy,
    )?;
    Ok(BtcProspectiveChallengePreparationV0 {
        holdout,
        capsule,
        ledger_digest: ledger.ledger_digest,
        regime_pack_count: packs.len(),
    })
}

fn mean(values: &[f32]) -> f32 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f32>() / values.len() as f32
    }
}

fn median(mut values: Vec<f32>) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(f32::total_cmp);
    let middle = values.len() / 2;
    if values.len() % 2 == 0 {
        (values[middle - 1] + values[middle]) / 2.0
    } else {
        values[middle]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        core::ReasonCode,
        data::{
            AcquisitionPolicy, DataLookback, MockReadOnlyProvider, ReadOnlyProviderResponse,
            SnapshotProvenance, SnapshotQualitySummary,
        },
        league::{HistoricalOhlcvRow, HistoricalReplayDataset},
    };

    fn snapshot(id: &str, source: SnapshotSourceType, rows: usize) -> DataSnapshot {
        let dataset = HistoricalReplayDataset {
            symbol: id.to_string(),
            source: "approved-sanitized-history".to_string(),
            rows: (0..rows)
                .map(|index| HistoricalOhlcvRow {
                    symbol: id.to_string(),
                    timestamp_ms: index as u64 + 1,
                    open: 10.0 + index as f64,
                    high: 11.0 + index as f64,
                    low: 9.0 + index as f64,
                    close: 10.5 + index as f64,
                    volume: 100.0,
                    trade_value: None,
                })
                .collect(),
            reason_codes: vec![],
        };
        DataSnapshot {
            snapshot_id: id.to_string(),
            request_key: id.to_string(),
            provider_id: "provider".to_string(),
            dataset_kind: DatasetKind::DailyOhlcv,
            market_scope: AcquisitionMarketScope::UsStocks,
            symbols: vec![id.to_string()],
            requested_lookback: DataLookback {
                bars: rows,
                start_timestamp_ms: Some(1),
                end_timestamp_ms: Some(rows as u64),
            },
            actual_start_timestamp_ms: Some(1),
            actual_end_timestamp_ms: Some(rows as u64),
            fetched_at_ms: 1,
            normalized_at_ms: 1,
            schema_version: 1,
            row_count: rows,
            quality_summary: SnapshotQualitySummary {
                accepted: true,
                row_count: rows,
                reason_codes: vec![],
            },
            content_digest: historical_replay_dataset_digest_v0(&dataset),
            sanitized: true,
            read_only: true,
            normalized_dataset: dataset,
            provenance: SnapshotProvenance {
                provider_id: "provider".to_string(),
                acquisition_request_id: "request".to_string(),
                fetch_receipt_id: "receipt".to_string(),
                source_type: source,
                sanitized: true,
                credential_free: true,
                reason_codes: vec![],
            },
            reason_codes: vec![ReasonCode::DataSnapshotImmutable],
        }
    }

    fn harvest_config(symbols: Vec<String>) -> HistoricalHarvestConfigV0 {
        HistoricalHarvestConfigV0 {
            markets: vec![AcquisitionMarketScope::UsStocks],
            configured_universe: ConfiguredUniverse {
                symbols_by_market: BTreeMap::from([(AcquisitionMarketScope::UsStocks, symbols)]),
            },
            minimum_rows_per_series: 128,
            maximum_rows_per_series: 256,
            provider_preference: vec!["approved".to_string()],
            start_timestamp_ms: Some(1),
            end_timestamp_ms: Some(999),
            max_staleness_ms: 86_400_000,
            allow_partial_harvest: false,
        }
    }

    fn approved_capabilities() -> ProviderCapabilities {
        ProviderCapabilities {
            provider_id: "approved".to_string(),
            supported_markets: vec![AcquisitionMarketScope::UsStocks],
            supported_dataset_kinds: vec![DatasetKind::DailyOhlcv],
            supported_cadences: vec!["1d".to_string()],
            maximum_lookback_bars: 256,
            requires_credentials: false,
            read_only: true,
            enabled: true,
            approved_for_network: true,
            mock_only: false,
            reason_codes: vec![],
        }
    }

    fn approved_response(rows: usize) -> ReadOnlyProviderResponse {
        let sample = snapshot("AAA", SnapshotSourceType::ApprovedReadOnlyProvider, rows);
        ReadOnlyProviderResponse {
            request_id: String::new(),
            provider_id: String::new(),
            fetched_at_ms: 10,
            content_type: "application/x-soma-normalized-dataset".to_string(),
            reported_content_bytes: serde_json::to_string(&sample.normalized_dataset)
                .unwrap()
                .len(),
            normalized_dataset: sample.normalized_dataset,
            reason_codes: vec![],
        }
    }

    #[test]
    fn inventory_excludes_mock_snapshots_from_real_evidence() {
        let real = snapshot("REAL", SnapshotSourceType::ApprovedReadOnlyProvider, 128);
        let mock = snapshot("MOCK", SnapshotSourceType::Mock, 128);
        let inventory = inventory_historical_snapshots_v0(
            &[real, mock],
            &HistoricalEvidencePolicyV0::default(),
        )
        .unwrap();
        assert_eq!(inventory.real_historical_snapshots, 1);
        assert_eq!(inventory.synthetic_snapshots, 1);
        assert_eq!(inventory.accepted_series.len(), 1);
    }

    #[test]
    fn provider_gate_reports_no_configured_provider() {
        let inventory =
            inventory_historical_snapshots_v0(&[], &HistoricalEvidencePolicyV0::default()).unwrap();
        let config = harvest_config(vec!["ABC".to_string()]);
        let gate = select_approved_historical_provider_v0(
            &inventory,
            &HistoricalEvidencePolicyV0::default(),
            &config,
            &ReadOnlyProviderRegistry::default(),
        )
        .unwrap();
        assert_eq!(
            gate.status,
            HistoricalProviderGateStatusV0::NoApprovedHistoricalProviderConfigured
        );
    }

    #[test]
    fn frozen_pack_is_deterministic_and_detects_mutation() {
        let real = snapshot("REAL", SnapshotSourceType::ApprovedReadOnlyProvider, 128);
        let (_, pack) = freeze_momentum_historical_evidence_pack_v0(
            &[real],
            &HistoricalEvidencePolicyV0::default(),
        )
        .unwrap();
        assert!(verify_momentum_historical_evidence_pack_v0(&pack).is_ok());
        let mut changed = pack;
        changed.series[0].symbol.push('X');
        assert_eq!(
            verify_momentum_historical_evidence_pack_v0(&changed),
            Err(HistoricalEvidenceErrorV0::InvalidPackDigest)
        );
    }

    #[test]
    fn empty_cross_series_is_honest_about_missing_provider() {
        let (evidence, warm) = aggregate_cross_series_momentum_evidence_v0(
            &[],
            &CrossSeriesMomentumGateConfigV0::default(),
            HistoricalProviderGateStatusV0::NoApprovedHistoricalProviderConfigured,
        )
        .unwrap();
        assert_eq!(
            evidence.status,
            CrossSeriesMomentumVerdictV0::NoApprovedProvider
        );
        assert_eq!(
            warm.status,
            CrossSeriesWarmStartVerdictV0::InsufficientEvidence
        );
    }

    #[test]
    fn inventory_rejects_mutable_corrupt_and_unsafe_snapshots() {
        let mut mutable = snapshot("MUTABLE", SnapshotSourceType::ApprovedReadOnlyProvider, 128);
        mutable.read_only = false;
        let mut corrupt = snapshot("CORRUPT", SnapshotSourceType::ApprovedReadOnlyProvider, 128);
        corrupt.content_digest.push('x');
        let mut unsafe_snapshot =
            snapshot("UNSAFE", SnapshotSourceType::ApprovedReadOnlyProvider, 128);
        unsafe_snapshot.normalized_dataset.source = "bearer token".to_string();
        unsafe_snapshot.content_digest =
            historical_replay_dataset_digest_v0(&unsafe_snapshot.normalized_dataset);
        let inventory = inventory_historical_snapshots_v0(
            &[mutable, corrupt, unsafe_snapshot],
            &HistoricalEvidencePolicyV0::default(),
        )
        .unwrap();
        assert_eq!(inventory.accepted_series.len(), 0);
        assert!(
            inventory
                .rejected_snapshots
                .iter()
                .any(|entry| entry.status == HistoricalSnapshotStatusV0::Mutable)
        );
        assert!(
            inventory
                .rejected_snapshots
                .iter()
                .any(|entry| entry.status == HistoricalSnapshotStatusV0::InvalidDigest)
        );
        assert!(
            inventory
                .rejected_snapshots
                .iter()
                .any(|entry| entry.status == HistoricalSnapshotStatusV0::Unsafe)
        );
    }

    #[test]
    fn harvest_uses_only_the_approved_broker_provider() {
        let inventory =
            inventory_historical_snapshots_v0(&[], &HistoricalEvidencePolicyV0::default()).unwrap();
        let mut registry = ReadOnlyProviderRegistry::default();
        registry.register(approved_capabilities());
        let mut policy = AcquisitionPolicy::default();
        policy.allow_approved_readonly_network = true;
        let mut broker = DataAcquisitionBroker::new(registry, policy);
        let mut provider = MockReadOnlyProvider {
            capabilities: approved_capabilities(),
            default_response: Some(approved_response(128)),
            default_failure: None,
            requests: vec![],
        };
        let harvest = harvest_historical_snapshots_v0(
            &inventory,
            &HistoricalEvidencePolicyV0::default(),
            &harvest_config(vec!["AAA".to_string()]),
            &mut broker,
            10,
            Some(&mut provider),
        )
        .unwrap();
        assert_eq!(
            harvest.status,
            HistoricalAcquisitionStatusV0::SnapshotsAcquired,
            "{:?}",
            harvest.reason_codes
        );
        assert_eq!(harvest.acquired_snapshots.len(), 1);
        assert_eq!(provider.requests.len(), 1);
        assert_eq!(provider.requests[0].symbols, vec!["AAA".to_string()]);
        assert_eq!(provider.requests[0].cadence, "1d");
    }

    #[test]
    fn sufficient_existing_evidence_skips_the_provider_call() {
        let snapshots = [
            snapshot("AAA", SnapshotSourceType::ApprovedReadOnlyProvider, 128),
            snapshot("BBB", SnapshotSourceType::ApprovedReadOnlyProvider, 128),
            snapshot("CCC", SnapshotSourceType::ApprovedReadOnlyProvider, 128),
        ];
        let inventory =
            inventory_historical_snapshots_v0(&snapshots, &HistoricalEvidencePolicyV0::default())
                .unwrap();
        let mut broker = DataAcquisitionBroker::new(
            ReadOnlyProviderRegistry::default(),
            AcquisitionPolicy::default(),
        );
        let harvest = harvest_historical_snapshots_v0(
            &inventory,
            &HistoricalEvidencePolicyV0::default(),
            &harvest_config(vec!["AAA".to_string()]),
            &mut broker,
            10,
            None,
        )
        .unwrap();
        assert_eq!(
            harvest.status,
            HistoricalAcquisitionStatusV0::ExistingSnapshotsUsed
        );
    }

    #[test]
    fn report_exposes_synthetic_exclusion_and_shadow_boundaries() {
        let inventory = inventory_historical_snapshots_v0(
            &[snapshot("MOCK", SnapshotSourceType::Mock, 128)],
            &HistoricalEvidencePolicyV0::default(),
        )
        .unwrap();
        let provider = HistoricalProviderGateResultV0 {
            status: HistoricalProviderGateStatusV0::NoApprovedHistoricalProviderConfigured,
            selected_provider_id: None,
            rejected_provider_ids: vec![],
            reason_codes: vec![],
        };
        let (evidence, warm) = aggregate_cross_series_momentum_evidence_v0(
            &[],
            &CrossSeriesMomentumGateConfigV0::default(),
            provider.status,
        )
        .unwrap();
        let report = build_momentum_historical_evidence_report_v0(
            &inventory,
            &provider,
            HistoricalAcquisitionStatusV0::NoRealHistoricalEvidence,
            &[],
            &evidence,
            &warm,
        );
        assert!(
            report
                .lines
                .contains(&"synthetic_fixtures_excluded=1".to_string())
        );
        assert!(
            report
                .lines
                .contains(&"momentum_model=ShadowOnly".to_string())
        );
        assert!(
            report
                .lines
                .contains(&"no_live_trading_readiness".to_string())
        );
    }

    #[test]
    fn campaign_sufficiency_uses_existing_window_requirements() {
        let config = MomentumLearningCampaignConfigV0::default();
        let insufficient = assess_momentum_campaign_sufficiency_v0(128, &config).unwrap();
        assert!(!insufficient.sufficient);
        assert!(insufficient.required_minimum_rows > insufficient.available_rows);

        let sufficient =
            assess_momentum_campaign_sufficiency_v0(insufficient.required_minimum_rows, &config)
                .unwrap();
        assert!(sufficient.sufficient);
        assert!(sufficient.possible_windows >= sufficient.required_windows);
    }

    #[test]
    fn evidence_usage_ledger_is_deterministic_and_seals_all_observed_rows() {
        let snapshot = snapshot("BTC", SnapshotSourceType::ApprovedReadOnlyProvider, 12);
        let first = build_historical_evidence_usage_ledger_v0(&snapshot, &[]).unwrap();
        let second = build_historical_evidence_usage_ledger_v0(&snapshot, &[]).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.maximum_consumed_timestamp_ms, 12);
        assert!(first.usages.iter().all(|usage| {
            !usage.campaign_ids.iter().any(|value| value.contains('/'))
                && !usage
                    .model_version_ids
                    .iter()
                    .any(|value| value.contains('/'))
        }));
    }

    #[test]
    fn segmentation_is_equal_length_non_overlapping_and_gap_preserving() {
        let snapshot = snapshot("BTC", SnapshotSourceType::ApprovedReadOnlyProvider, 12);
        let config = BtcHistoricalRegimeConfigV0 {
            minimum_regimes: 3,
            regime_rows: 3,
            inter_regime_gap_rows: 1,
            minimum_campaign_windows_per_regime: 1,
            segmentation_policy: TemporalRegimeSegmentationPolicyV0::EqualLengthChronological,
        };
        let first = segment_btc_historical_regimes_v0(&snapshot, &config).unwrap();
        let second = segment_btc_historical_regimes_v0(&snapshot, &config).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.status, BtcRegimeSegmentationStatusV0::Ready);
        assert_eq!(first.regimes.len(), 3);
        assert_eq!(first.regimes[0].end_row_index_exclusive, 3);
        assert_eq!(first.regimes[1].start_row_index, 4);
        assert_eq!(first.regimes[2].start_row_index, 8);
        assert!(
            first
                .regimes
                .windows(2)
                .all(|pair| pair[0].end_row_index_exclusive <= pair[1].start_row_index)
        );
    }

    #[test]
    fn insufficient_regimes_never_freeze_packs_or_claim_recurrence() {
        let snapshot = snapshot("BTC", SnapshotSourceType::ApprovedReadOnlyProvider, 5);
        let config = BtcHistoricalRegimeConfigV0 {
            minimum_regimes: 2,
            regime_rows: 3,
            inter_regime_gap_rows: 0,
            minimum_campaign_windows_per_regime: 1,
            segmentation_policy: TemporalRegimeSegmentationPolicyV0::EqualLengthChronological,
        };
        let segmentation = segment_btc_historical_regimes_v0(&snapshot, &config).unwrap();
        assert_eq!(
            segmentation.status,
            BtcRegimeSegmentationStatusV0::InsufficientRows
        );
        assert!(
            freeze_btc_historical_regime_packs_v0(
                &snapshot,
                &segmentation,
                &HistoricalEvidencePolicyV0::default(),
            )
            .unwrap()
            .is_empty()
        );
        let aggregate = aggregate_btc_cross_regime_evidence_v0(&[], &config).unwrap();
        assert_eq!(
            aggregate.status,
            BtcCrossRegimeRepresentationStatusV0::InsufficientRegimes
        );
    }

    fn regime_result(
        regime_id: &str,
        no_signal_windows: usize,
        selected_checkpoint_windows: usize,
        in_support_windows: usize,
        out_of_support_windows: usize,
    ) -> BtcTemporalRegimeEvidenceResultV0 {
        BtcTemporalRegimeEvidenceResultV0 {
            regime_id: regime_id.to_string(),
            row_count: 8,
            campaign_windows: 2,
            no_signal_windows,
            selected_checkpoint_windows,
            in_support_windows,
            out_of_support_windows,
            support_unavailable_windows: 0,
            validation_in_support_windows: 0,
            validation_out_of_support_windows: 0,
            validation_insufficient_windows: 0,
            validation_gate_unavailable_windows: 0,
            support_traces: vec![],
            earliest_shift_stage: EarliestTemporalShiftStageV0::InsufficientEvidence,
            temporal_root_cause: ProbabilityCollapseRootCauseV0::Unknown,
            frozen_representation_breach_count: 0,
            warm_start_status: WarmStartLockInStatusV0::WarmAndColdBothNoSignal,
            abstention_count: no_signal_windows,
            accepted_predictive_versions: 0,
            final_verdict: SupportGatedMomentumSeriesVerdictV0::NoUsableValidationSignal,
            reason_codes: vec!["no_usable_validation_signal".to_string()],
            campaign_config_digest: "campaign".to_string(),
            encoder_parameter_digest: "encoder".to_string(),
            report_digest: format!("report-{regime_id}"),
        }
    }

    fn regime_reference(regime_id: &str, rank: usize) -> BtcTemporalRegimeRefV0 {
        BtcTemporalRegimeRefV0 {
            regime_id: regime_id.to_string(),
            chronological_rank: rank,
            row_count: 8,
            range_digest: format!("range-{regime_id}"),
            pack_digest: format!("pack-{regime_id}"),
        }
    }

    #[test]
    fn closed_result_allows_mixed_no_signal_and_selected_checkpoint_windows() {
        let raw = regime_result("mixed-window-counts", 1, 1, 1, 0);
        let closed = close_btc_temporal_regime_result_v0(&raw, regime_reference(&raw.regime_id, 0));
        assert_eq!(closed.no_signal_windows, 1);
        assert_eq!(closed.selected_checkpoint_windows, 1);
        assert!(validate_btc_temporal_regime_closed_result_v0(&closed).is_ok());
    }

    #[test]
    fn legacy_fallback_reproduces_diagnostic_failure_for_valid_unmapped_results() {
        let config = BtcHistoricalRegimeConfigV0 {
            minimum_regimes: 2,
            regime_rows: 8,
            inter_regime_gap_rows: 0,
            minimum_campaign_windows_per_regime: 1,
            segmentation_policy: TemporalRegimeSegmentationPolicyV0::EqualLengthChronological,
        };
        let first = regime_result("older", 0, 1, 1, 1);
        let second = regime_result("newer", 0, 1, 0, 1);
        assert_eq!(
            aggregate_btc_cross_regime_evidence_v0(&[first, second], &config)
                .unwrap()
                .status,
            BtcCrossRegimeRepresentationStatusV0::DiagnosticFailure
        );
    }

    #[test]
    fn sealed_no_signal_regimes_are_completed_and_aggregate_honestly() {
        let config = BtcHistoricalRegimeConfigV0 {
            minimum_regimes: 2,
            regime_rows: 8,
            inter_regime_gap_rows: 0,
            minimum_campaign_windows_per_regime: 1,
            segmentation_policy: TemporalRegimeSegmentationPolicyV0::EqualLengthChronological,
        };
        let raw = vec![
            regime_result("older", 2, 0, 0, 0),
            regime_result("newer", 2, 0, 0, 0),
        ];
        let proof = build_cross_regime_model_freeze_proof_v0(&raw);
        let closed = raw
            .iter()
            .enumerate()
            .map(|(rank, result)| {
                close_btc_temporal_regime_result_v0(
                    result,
                    regime_reference(&result.regime_id, rank),
                )
            })
            .collect::<Vec<_>>();
        assert!(closed.iter().all(|result| {
            result.execution_health == RegimeExecutionHealthV0::Completed
                && result.diagnostic_completeness
                    == RegimeDiagnosticCompletenessV0::PartialNoSelectedCheckpoint
                && result.model_evidence_outcome
                    == RegimeModelEvidenceOutcomeV0::NoUsableValidationSignal
                && result.operational_shadow_result
                    == RegimeOperationalShadowResultV0::ShadowAbstainNoSignal
                && validate_btc_temporal_regime_closed_result_v0(result).is_ok()
        }));
        let aggregate = aggregate_btc_cross_regime_closed_evidence_v0(&closed, &config, &proof);
        assert_eq!(
            aggregate.status,
            BtcCrossRegimeRepresentationStatusV0::PredominantlyNoUsableSignal
        );
        assert!(aggregate.diagnostic_failure_root_cause.is_none());
    }

    #[test]
    fn freeze_mismatch_is_a_real_diagnostic_failure() {
        let config = BtcHistoricalRegimeConfigV0 {
            minimum_regimes: 2,
            regime_rows: 8,
            inter_regime_gap_rows: 0,
            minimum_campaign_windows_per_regime: 1,
            segmentation_policy: TemporalRegimeSegmentationPolicyV0::EqualLengthChronological,
        };
        let mut second = regime_result("newer", 2, 0, 0, 0);
        second.campaign_config_digest = "different-campaign".to_string();
        let raw = vec![regime_result("older", 2, 0, 0, 0), second];
        let proof = build_cross_regime_model_freeze_proof_v0(&raw);
        let closed = raw
            .iter()
            .enumerate()
            .map(|(rank, result)| {
                close_btc_temporal_regime_result_v0(
                    result,
                    regime_reference(&result.regime_id, rank),
                )
            })
            .collect::<Vec<_>>();
        let aggregate = aggregate_btc_cross_regime_closed_evidence_v0(&closed, &config, &proof);
        assert_eq!(
            aggregate.status,
            BtcCrossRegimeRepresentationStatusV0::DiagnosticFailure
        );
        assert_eq!(
            aggregate.diagnostic_failure_root_cause,
            Some(CrossRegimeDiagnosticFailureRootCauseV0::ModelConfigDigestMismatch)
        );
    }

    #[test]
    fn zero_support_qualified_paths_do_not_aggregate_as_sparse_support() {
        let config = BtcHistoricalRegimeConfigV0 {
            minimum_regimes: 2,
            regime_rows: 8,
            inter_regime_gap_rows: 0,
            minimum_campaign_windows_per_regime: 1,
            segmentation_policy: TemporalRegimeSegmentationPolicyV0::EqualLengthChronological,
        };
        let raw = vec![
            regime_result("older", 0, 1, 0, 0),
            regime_result("newer", 0, 1, 0, 0),
        ];
        let closed = raw
            .iter()
            .enumerate()
            .map(|(rank, result)| {
                close_btc_temporal_regime_result_v0(
                    result,
                    regime_reference(&result.regime_id, rank),
                )
            })
            .collect::<Vec<_>>();
        let support = aggregate_btc_cross_regime_support_evidence_v0(
            &closed,
            &config,
            BtcCrossRegimeRepresentationStatusV0::SparseInSupportEvidence,
        );
        assert_eq!(
            support.support_status,
            BtcCrossRegimeSupportStatusV0::NoSupportQualifiedEvidence
        );
    }

    #[test]
    fn prospective_holdout_uses_strictly_later_rows_and_invalidates_early_access() {
        let snapshot = snapshot("BTC", SnapshotSourceType::ApprovedReadOnlyProvider, 12);
        let ledger = build_historical_evidence_usage_ledger_v0(&snapshot, &[]).unwrap();
        let policy = ProspectiveHoldoutPolicyConfigV0 {
            minimum_future_rows: 4,
            required_future_windows: 1,
        };
        let mut manifest = seal_prospective_holdout_v0(&ledger, &policy, &[]).unwrap();
        assert_eq!(
            manifest.status,
            ProspectiveHoldoutStatusV0::PolicySealedNoFutureRows
        );
        assert!(!prospective_holdout_timestamp_is_eligible_v0(
            12, &ledger, &manifest
        ));
        assert!(prospective_holdout_timestamp_is_eligible_v0(
            13, &ledger, &manifest
        ));
        record_prospective_holdout_access_v0(&mut manifest, false, true, true);
        assert_eq!(
            manifest.status,
            ProspectiveHoldoutStatusV0::InvalidatedByEarlyAccess
        );
        assert!(!prospective_holdout_timestamp_is_eligible_v0(
            13, &ledger, &manifest
        ));
    }

    fn prospective_test_resolution() -> ProspectiveCandidateResolutionV0 {
        let mut candidate = FrozenProspectiveMambaCandidateV0 {
            source_campaign_id: "campaign".to_string(),
            source_regime_id: "regime".to_string(),
            source_window_id: "window".to_string(),
            source_candidate_id: "c0_reference".to_string(),
            checkpoint_epoch: 1,
            model_version_id: "frozen-version".to_string(),
            encoder_digest: "encoder".to_string(),
            representation_normalizer_digest: "normalizer".to_string(),
            head_digest: "head".to_string(),
            feature_config_digest: "feature".to_string(),
            sequence_config_digest: "sequence".to_string(),
            label_config_digest: "label".to_string(),
            support_policy_digest: "support".to_string(),
            collapse_policy_digest: "collapse".to_string(),
            backend_policy_digest: "backend".to_string(),
            shadow_only: true,
            head_weight_bits: vec![0.1f32.to_bits()],
            head_bias_bits: 0.2f32.to_bits(),
            artifact_digest: String::new(),
        };
        candidate.artifact_digest = candidate_artifact_digest_v0(&candidate);
        let linear = FrozenProspectiveComparatorV0 {
            kind: ProspectiveComparatorKindV0::FrozenLinear,
            source_training_digest: "training".to_string(),
            parameter_digest: "linear".to_string(),
            feature_config_digest: "feature".to_string(),
            label_config_digest: "label".to_string(),
            parameter_bits: vec![0.3f32.to_bits()],
            artifact_digest: String::new(),
        };
        let constant = FrozenProspectiveComparatorV0 {
            kind: ProspectiveComparatorKindV0::FrozenTrainingPrevalenceConstant,
            source_training_digest: "training".to_string(),
            parameter_digest: "constant".to_string(),
            feature_config_digest: "feature".to_string(),
            label_config_digest: "label".to_string(),
            parameter_bits: vec![0.5f32.to_bits()],
            artifact_digest: String::new(),
        };
        let mut linear = linear;
        linear.artifact_digest = comparator_artifact_digest_v0(&linear);
        let mut constant = constant;
        constant.artifact_digest = comparator_artifact_digest_v0(&constant);
        ProspectiveCandidateResolutionV0 {
            candidate,
            linear,
            constant,
        }
    }

    fn prospective_test_state() -> ProspectiveChallengeLocalStateV0 {
        let capsule = build_prospective_shadow_challenge_capsule_v0(
            "BTC".to_string(),
            10,
            prospective_test_resolution(),
            1,
            ProspectiveEvaluationPolicyV0 {
                minimum_finalized_future_rows: 4,
                minimum_mature_events: 2,
                minimum_support_qualified_events: 2,
                minimum_positive_labels: 1,
                minimum_negative_labels_for_auc: 1,
                maximum_technical_failure_count: 0,
                primary_metric: "brier".to_string(),
                secondary_metrics: vec!["auc".to_string()],
                require_brier_improvement_over_linear: true,
                require_brier_improvement_over_constant: true,
                require_no_severe_probability_collapse: true,
            },
        )
        .unwrap();
        new_prospective_challenge_local_state_v0(capsule).unwrap()
    }

    #[test]
    fn prospective_capsule_is_deterministic_and_rejects_mutation() {
        let first = prospective_test_state();
        let second = prospective_test_state();
        assert_eq!(first.capsule.capsule_digest, second.capsule.capsule_digest);
        assert!(validate_prospective_challenge_local_state_v0(&first).is_ok());
        let mut changed = first;
        changed.capsule.candidate.head_bias_bits ^= 1;
        assert_eq!(
            validate_prospective_challenge_local_state_v0(&changed),
            Err(ProspectiveChallengeErrorV0::InvalidCapsule)
        );
    }

    #[test]
    fn prospective_state_machine_and_vault_fail_closed() {
        assert_eq!(
            prospective_challenge_transition_v0(
                ProspectiveChallengeStatusV0::Draft,
                ProspectiveChallengeStatusV0::Accumulating,
            ),
            Err(ProspectiveChallengeErrorV0::InvalidTransition)
        );
        let mut state = prospective_test_state();
        assert!(confirm_prospective_pre_registration_v0(&mut state).is_ok());
        assert_eq!(
            append_prospective_vault_row_v0(
                &mut state,
                ProspectiveEvidenceRowRefV0 {
                    timestamp_ms: 10,
                    canonical_row_digest: "row".to_string(),
                    finalized: true,
                },
            ),
            Err(ProspectiveChallengeErrorV0::InvalidVault)
        );
        append_prospective_vault_row_v0(
            &mut state,
            ProspectiveEvidenceRowRefV0 {
                timestamp_ms: 11,
                canonical_row_digest: "row-11".to_string(),
                finalized: true,
            },
        )
        .unwrap();
        assert_eq!(
            blind_prospective_challenge_status_v0(&state)
                .unwrap()
                .finalized_row_count,
            1
        );
    }

    #[test]
    fn prospective_prediction_journal_seals_bits_without_label_access() {
        let mut state = prospective_test_state();
        confirm_prospective_pre_registration_v0(&mut state).unwrap();
        let comparator_artifact_digests = state
            .capsule
            .comparators
            .iter()
            .map(|comparator| comparator.artifact_digest.clone())
            .collect::<Vec<_>>();
        let challenge_id = state.capsule.challenge_id.clone();
        let candidate_artifact_digest = state.capsule.candidate.artifact_digest.clone();
        let event = ProspectivePredictionEventV0 {
            challenge_id,
            event_id: "event-11".to_string(),
            prediction_timestamp_ms: 11,
            required_label_maturity_timestamp_ms: 12,
            input_evidence_digest: "input".to_string(),
            candidate_artifact_digest,
            comparator_artifact_digests: comparator_artifact_digests.clone(),
            support_applicability: "applicable".to_string(),
            support_decision: "in_support".to_string(),
            candidate_prediction: Some(SealedProbabilityV0 {
                ieee754_bits: 0.7f32.to_bits(),
            }),
            comparator_predictions: comparator_artifact_digests
                .iter()
                .map(|digest| SealedComparatorPredictionV0 {
                    comparator_artifact_digest: digest.clone(),
                    probability: SealedProbabilityV0 {
                        ieee754_bits: 0.5f32.to_bits(),
                    },
                })
                .collect(),
            operational_outcome: ProspectiveShadowOutcomeV0::ShadowPredictionSealed,
            label_status: ProspectiveLabelStatusV0::AwaitingFutureRows,
            event_digest: String::new(),
        };
        append_prospective_prediction_event_v0(&mut state, event).unwrap();
        let blind = blind_prospective_challenge_status_v0(&state).unwrap();
        assert_eq!(blind.support_qualified_prediction_count, 1);
        assert_eq!(blind.awaiting_label_maturity_count, 1);
        assert_eq!(
            open_prospective_challenge_once_v0(&mut state, false),
            Err(ProspectiveChallengeErrorV0::EvaluationOpeningForbidden)
        );
    }

    #[test]
    fn prospective_local_state_round_trip_reopens_and_verifies_atomically() {
        let state = prospective_test_state();
        let path = std::env::temp_dir().join(format!(
            "soma-prospective-state-{}.json",
            state.capsule.capsule_digest
        ));
        let _ = std::fs::remove_file(&path);
        write_prospective_challenge_local_state_v0(&path, &state).unwrap();
        assert_eq!(
            read_prospective_challenge_local_state_v0(&path).unwrap(),
            state
        );
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn prospective_registry_provenance_explains_the_pre_registration_transition() {
        let mut state = prospective_test_state();
        let expected_capsule = state.capsule.clone();
        assert_eq!(
            compute_prospective_registry_digest_provenance_v0(&state)
                .unwrap()
                .status,
            RegistryDigestProvenanceStatusV0::MissingTransitionRecord
        );
        confirm_prospective_pre_registration_v0(&mut state).unwrap();
        let proof = build_prospective_challenge_freeze_proof_v0(&state, &expected_capsule);
        assert!(proof.all_equal);
        let provenance = compute_prospective_registry_digest_provenance_v0(&state).unwrap();
        assert_eq!(
            provenance.status,
            RegistryDigestProvenanceStatusV0::ValidTransitionChain
        );
        assert_ne!(
            provenance.initial_sealed_registry_digest,
            provenance.pre_registration_committed_registry_digest
        );
        assert_eq!(
            provenance.current_registry_digest,
            state.registry.registry_digest
        );
        assert_eq!(state.registry_transition_records.len(), 1);
        assert!(!state.registry_transition_records[0].evidence_access_occurred);
        assert!(!state.registry_transition_records[0].labels_accessed);
    }

    #[test]
    fn prospective_registry_closure_reconstructs_legacy_committed_state_once() {
        let mut state = prospective_test_state();
        let expected_capsule = state.capsule.clone();
        let entry = state.registry.challenges.first_mut().unwrap();
        entry.status = ProspectiveChallengeStatusV0::PreRegistrationCommitted;
        state.registry.registry_digest = registry_digest_v0(&state.registry);
        let first =
            seal_missing_pre_registration_transition_v0(&mut state, &expected_capsule).unwrap();
        let second =
            seal_missing_pre_registration_transition_v0(&mut state, &expected_capsule).unwrap();
        assert_eq!(
            first.status,
            RegistryDigestProvenanceStatusV0::ValidTransitionChain
        );
        assert_eq!(first.provenance_digest, second.provenance_digest);
        assert_eq!(state.registry_transition_records.len(), 1);
        let mut altered = expected_capsule;
        altered.candidate.head_bias_bits ^= 1;
        assert!(!build_prospective_challenge_freeze_proof_v0(&state, &altered).all_equal);
    }

    #[test]
    fn prospective_event_append_is_idempotent_and_rejects_backdating() {
        let mut state = prospective_test_state();
        confirm_prospective_pre_registration_v0(&mut state).unwrap();
        append_prospective_vault_row_v0(
            &mut state,
            ProspectiveEvidenceRowRefV0 {
                timestamp_ms: 11,
                canonical_row_digest: "row-11".to_string(),
                finalized: true,
            },
        )
        .unwrap();
        let comparator_artifact_digests = state
            .capsule
            .comparators
            .iter()
            .map(|comparator| comparator.artifact_digest.clone())
            .collect::<Vec<_>>();
        let event = ProspectivePredictionEventV0 {
            challenge_id: state.capsule.challenge_id.clone(),
            event_id: "event-11".to_string(),
            prediction_timestamp_ms: 11,
            required_label_maturity_timestamp_ms: 12,
            input_evidence_digest: "input".to_string(),
            candidate_artifact_digest: state.capsule.candidate.artifact_digest.clone(),
            comparator_artifact_digests: comparator_artifact_digests.clone(),
            support_applicability: "applicable".to_string(),
            support_decision: "in_support".to_string(),
            candidate_prediction: Some(SealedProbabilityV0 {
                ieee754_bits: 0.7f32.to_bits(),
            }),
            comparator_predictions: comparator_artifact_digests
                .iter()
                .map(|digest| SealedComparatorPredictionV0 {
                    comparator_artifact_digest: digest.clone(),
                    probability: SealedProbabilityV0 {
                        ieee754_bits: 0.5f32.to_bits(),
                    },
                })
                .collect(),
            operational_outcome: ProspectiveShadowOutcomeV0::ShadowPredictionSealed,
            label_status: ProspectiveLabelStatusV0::AwaitingFutureRows,
            event_digest: String::new(),
        };
        append_prospective_prediction_event_v0(&mut state, event.clone()).unwrap();
        append_prospective_prediction_event_v0(&mut state, event.clone()).unwrap();
        assert_eq!(state.journal.events.len(), 1);
        let mut changed = event;
        changed.candidate_prediction = Some(SealedProbabilityV0 {
            ieee754_bits: 0.6f32.to_bits(),
        });
        assert_eq!(
            append_prospective_prediction_event_v0(&mut state, changed),
            Err(ProspectiveChallengeErrorV0::InvalidJournal)
        );
        assert!(
            build_prospective_causal_context_bridge_v0(
                &state,
                "historical-snapshot".to_string(),
                "historical-range".to_string(),
                0,
                1,
                11,
            )
            .is_ok()
        );
    }
}
