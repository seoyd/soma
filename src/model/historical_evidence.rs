//! Evidence-first orchestration for real historical momentum evaluation.
//!
//! This module orchestrates existing normalized snapshots and the existing
//! read-only broker. Provider-specific parsing, active voting, and execution
//! remain outside it.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    core::{ReasonCode, stable_hash_string},
    data::{
        AcquisitionMarketScope, AcquisitionMode, AgentDataIntent, ConfiguredUniverse,
        DataAcquisitionBroker, DataLookback, DataPriority, DataSnapshot, DatasetKind,
        ProviderCapabilities, ReadOnlyMarketDataProvider, ReadOnlyProviderRegistry,
        SnapshotSourceType, build_acquisition_plan,
    },
    league::AgentKind,
};

use super::{
    FrozenMamba3EncoderV0, MambaRepresentationValueStatusV0, ModelDriftStatusV0,
    MomentumLearningCampaignConfigV0, MomentumLearningCampaignResultV0,
    MomentumLearningCampaignStatusV0, build_momentum_learning_windows_v0,
    run_momentum_learning_campaign_v0,
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
    serde_json::to_string(&snapshot.normalized_dataset)
        .map(|value| stable_hash_string(&value) == snapshot.content_digest)
        .unwrap_or(false)
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
        core::{ReasonCode, stable_hash_string},
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
            content_digest: stable_hash_string(&serde_json::to_string(&dataset).unwrap()),
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
        unsafe_snapshot.content_digest = stable_hash_string(
            &serde_json::to_string(&unsafe_snapshot.normalized_dataset).unwrap(),
        );
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
            HistoricalAcquisitionStatusV0::SnapshotsAcquired
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
}
