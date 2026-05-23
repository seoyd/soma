use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

fn default_positive_outcome() -> Option<f64> {
    Some(0.012)
}

fn default_risk_veto_volatility_threshold() -> f64 {
    0.08
}

fn default_input_path() -> Option<String> {
    Some("examples/minimal_ai_committee_core_sample.json".to_string())
}

fn default_style_mapping_mode() -> StyleMappingMode {
    StyleMappingMode::None
}

fn local_only(path: &str) -> bool {
    let path = path.trim();
    !path.is_empty()
        && !path.contains("://")
        && !path.starts_with("http:")
        && !path.starts_with("https:")
        && !path.starts_with("s3:")
}

fn clamp_unit(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MarketScope {
    KoreaShortTerm,
    KoreaLongTerm,
    UsShortTerm,
    UsLongTerm,
    CryptoShortTerm,
    CryptoLongTerm,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarketDataSnapshot {
    pub symbol: String,
    pub market_scope: MarketScope,
    pub timestamp: String,
    pub price: f64,
    pub change_pct: f64,
    pub volume: f64,
    pub volatility_hint: f64,
    pub source_label: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NewsSnapshot {
    pub symbol: String,
    pub headline: String,
    pub summary: String,
    pub sentiment_hint: String,
    pub source_label: String,
    pub timestamp: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AICommitteeMemberStatus {
    Active,
    Watchlist,
    Demoted,
    Disabled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AIRuntimeMode {
    MockLocal,
    ExternalModelDeferred,
    TrainingDeferred,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndependentMemberRole {
    TrendEntry,
    RiskGuard,
    EvidenceRegime,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemberMemoryState {
    pub member_id: String,
    pub recent_symbols: Vec<String>,
    pub recent_opinion_count: u32,
    pub recent_event_count: u32,
    pub recent_good_call_count: u32,
    pub recent_bad_call_count: u32,
    pub recent_risk_veto_count: u32,
    pub notes: Vec<String>,
}

impl MemberMemoryState {
    pub fn new(member_id: &str) -> Self {
        Self {
            member_id: member_id.to_string(),
            recent_symbols: Vec::new(),
            recent_opinion_count: 0,
            recent_event_count: 0,
            recent_good_call_count: 0,
            recent_bad_call_count: 0,
            recent_risk_veto_count: 0,
            notes: vec!["offline lightweight member memory; no model weight update".to_string()],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreferredTimeHorizon {
    ShortTerm,
    LongTerm,
    MultiHorizon,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreferredMarketBias {
    Korea,
    US,
    Crypto,
    Global,
    Any,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchetypeStyleTag {
    Trend,
    Value,
    Momentum,
    Macro,
    RiskControl,
    EvidenceQuality,
    Contrarian,
    Liquidity,
    Volatility,
    EventDriven,
    Quality,
    Growth,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchetypeRiskBias {
    Conservative,
    Balanced,
    Aggressive,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidencePreference {
    PriceAction,
    Fundamentals,
    Macro,
    News,
    OnChain,
    Quant,
    Mixed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SourceConfidence {
    Low,
    Medium,
    High,
    ReviewRequired,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StyleCardStatus {
    ActiveStyleCard,
    ReviewRequired,
    Disabled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberStyleStatus {
    Ready,
    ReadyWithWarnings,
    NeedsReview,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StyleMappingMode {
    None,
    ThreeMemberDefault,
    LocalFixture,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InvestorArchetypeStyleCard {
    pub archetype_id: String,
    pub display_name: String,
    pub public_style_summary: String,
    pub preferred_time_horizon: PreferredTimeHorizon,
    pub preferred_market_bias: PreferredMarketBias,
    pub primary_style_tags: Vec<ArchetypeStyleTag>,
    pub risk_bias: ArchetypeRiskBias,
    pub evidence_preference: EvidencePreference,
    pub do_not_learn: Vec<String>,
    pub source_confidence: SourceConfidence,
    pub status: StyleCardStatus,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemberArchetypeWeight {
    pub archetype_id: String,
    pub weight: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemberStyleBlend {
    pub member_id: String,
    pub role: IndependentMemberRole,
    pub archetype_weights: Vec<MemberArchetypeWeight>,
    pub blend_summary: String,
    pub prohibited_claims: Vec<String>,
    pub source_confidence_minimum: SourceConfidence,
    pub style_status: MemberStyleStatus,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StyleInfluencedMemberProfile {
    pub member_id: String,
    pub base_role: IndependentMemberRole,
    pub style_blend: MemberStyleBlend,
    pub risk_bias: ArchetypeRiskBias,
    pub time_horizon_bias: PreferredTimeHorizon,
    pub evidence_bias: EvidencePreference,
    pub decision_bias_notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ArchetypeStyleCardRegistry {
    pub cards: Vec<InvestorArchetypeStyleCard>,
    pub active_count: usize,
    pub review_required_count: usize,
    pub disabled_count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RealArchetypeIntakePolicy {
    pub accepts_local_json_only: bool,
    pub validates_all_cards: bool,
    pub rejects_impersonation_wording: bool,
    pub rejects_private_strategy_claims: bool,
    pub rejects_guaranteed_return_claims: bool,
    pub requires_do_not_learn_guards: bool,
    pub requires_source_confidence: bool,
    pub review_required_stays_review_required: bool,
    pub does_not_activate_live_agents: bool,
    pub does_not_train_models: bool,
    pub does_not_call_network: bool,
}

impl Default for RealArchetypeIntakePolicy {
    fn default() -> Self {
        Self {
            accepts_local_json_only: true,
            validates_all_cards: true,
            rejects_impersonation_wording: true,
            rejects_private_strategy_claims: true,
            rejects_guaranteed_return_claims: true,
            requires_do_not_learn_guards: true,
            requires_source_confidence: true,
            review_required_stays_review_required: true,
            does_not_activate_live_agents: true,
            does_not_train_models: true,
            does_not_call_network: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThreeMemberStyleMapping {
    pub trend_entry_blend: MemberStyleBlend,
    pub risk_guard_blend: MemberStyleBlend,
    pub evidence_regime_blend: MemberStyleBlend,
    pub unmapped_archetypes: Vec<String>,
    pub review_required_archetypes: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberCoreFamily {
    Mamba3GatedDeltaNet,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SequenceCoreKind {
    Mamba3Deferred,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryCoreKind {
    GatedDeltaNetDeferred,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoreRuntimeStatus {
    OfflineFixture,
    MockLocal,
    RuntimeDeferred,
    TrainingDeferred,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberCoreLoadPolicy {
    LazyLoad,
    PrewarmTopMembers,
    Disabled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuantizationHint {
    None,
    Q8,
    Q6,
    Q4,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mamba3GatedDeltaNetCoreSpec {
    pub core_id: String,
    pub member_id: String,
    pub core_family: MemberCoreFamily,
    pub sequence_core: SequenceCoreKind,
    pub memory_core: MemoryCoreKind,
    pub runtime_status: CoreRuntimeStatus,
    pub load_policy: MemberCoreLoadPolicy,
    pub max_context_tokens_hint: u32,
    pub memory_budget_mb_hint: u32,
    pub quantization_hint: QuantizationHint,
    pub notes: Vec<String>,
}

impl Mamba3GatedDeltaNetCoreSpec {
    pub fn mock_local_for(member_id: &str) -> Self {
        Self::deferred_contract_for(member_id, CoreRuntimeStatus::MockLocal)
    }

    pub fn offline_fixture_for(member_id: &str) -> Self {
        Self::deferred_contract_for(member_id, CoreRuntimeStatus::OfflineFixture)
    }

    pub fn runtime_deferred_for(member_id: &str) -> Self {
        Self::deferred_contract_for(member_id, CoreRuntimeStatus::RuntimeDeferred)
    }

    pub fn training_deferred_for(member_id: &str) -> Self {
        Self::deferred_contract_for(member_id, CoreRuntimeStatus::TrainingDeferred)
    }

    fn deferred_contract_for(member_id: &str, runtime_status: CoreRuntimeStatus) -> Self {
        Self {
            core_id: format!("mamba3-gdn-{}", member_id),
            member_id: member_id.to_string(),
            core_family: MemberCoreFamily::Mamba3GatedDeltaNet,
            sequence_core: SequenceCoreKind::Mamba3Deferred,
            memory_core: MemoryCoreKind::GatedDeltaNetDeferred,
            runtime_status,
            load_policy: MemberCoreLoadPolicy::LazyLoad,
            max_context_tokens_hint: 4096,
            memory_budget_mb_hint: match runtime_status {
                CoreRuntimeStatus::OfflineFixture | CoreRuntimeStatus::MockLocal => 32,
                CoreRuntimeStatus::RuntimeDeferred => 512,
                CoreRuntimeStatus::TrainingDeferred => 0,
            },
            quantization_hint: QuantizationHint::Q8,
            notes: vec![
                "Mamba3 + Gated DeltaNet contract only".to_string(),
                "runtime/training/live inference deferred".to_string(),
            ],
        }
    }

    fn memory_estimate_mb(&self) -> u32 {
        match self.runtime_status {
            CoreRuntimeStatus::OfflineFixture | CoreRuntimeStatus::MockLocal => {
                self.memory_budget_mb_hint.min(50)
            }
            CoreRuntimeStatus::RuntimeDeferred => self.memory_budget_mb_hint,
            CoreRuntimeStatus::TrainingDeferred => 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AICommitteeMember {
    pub member_id: String,
    pub display_name: String,
    pub market_scopes: Vec<MarketScope>,
    pub style_profile: String,
    pub voice_weight: f64,
    pub score: f64,
    pub status: AICommitteeMemberStatus,
    pub runtime_mode: AIRuntimeMode,
    #[serde(default)]
    pub core_spec: Option<Mamba3GatedDeltaNetCoreSpec>,
    #[serde(default)]
    pub role: Option<IndependentMemberRole>,
    #[serde(default)]
    pub memory_state: Option<MemberMemoryState>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemberInputPacket {
    pub member_id: String,
    pub market_data: MarketDataSnapshot,
    pub news: Vec<NewsSnapshot>,
    pub owner_context: Option<String>,
    pub previous_member_score: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DataRouterInput {
    pub market_data: Vec<MarketDataSnapshot>,
    pub news: Vec<NewsSnapshot>,
    pub members: Vec<AICommitteeMember>,
    #[serde(default)]
    pub owner_context: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DataRouterOutput {
    pub packets: Vec<MemberInputPacket>,
    pub routed_member_count: usize,
    pub unrouted_symbol_count: usize,
    pub safety_notes: Vec<String>,
}

pub trait AiMemberBrain {
    fn produce_opinion(&self, packet: &MemberInputPacket) -> MemberOpinion;
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DeterministicMockBrain {
    pub member: AICommitteeMember,
}

impl AiMemberBrain for DeterministicMockBrain {
    fn produce_opinion(&self, packet: &MemberInputPacket) -> MemberOpinion {
        produce_mock_opinion(&self.member, packet)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemberActivationPolicy {
    pub max_active_members_per_cycle: usize,
    pub max_active_members_per_market_scope: usize,
    pub prefer_high_voice_weight: bool,
    pub include_risk_member_always: bool,
    pub include_recent_event_trigger_member: bool,
    pub lazy_load_enabled: bool,
    pub unload_after_cycle: bool,
    pub prewarm_count: usize,
}

impl Default for MemberActivationPolicy {
    fn default() -> Self {
        Self {
            max_active_members_per_cycle: 5,
            max_active_members_per_market_scope: 3,
            prefer_high_voice_weight: true,
            include_risk_member_always: true,
            include_recent_event_trigger_member: true,
            lazy_load_enabled: true,
            unload_after_cycle: true,
            prewarm_count: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberSelectionSkipReason {
    ScopeMismatch,
    OverActivationLimit,
    Demoted,
    Disabled,
    RuntimeDeferred,
    TrainingDeferred,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemberActivationSkip {
    pub member_id: String,
    pub reason: MemberSelectionSkipReason,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemberRuntimeStatusReport {
    pub member_id: String,
    pub runtime_status: CoreRuntimeStatus,
    pub load_policy: MemberCoreLoadPolicy,
    pub memory_budget_mb_hint: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemberActivationPlan {
    pub market_scope: MarketScope,
    pub committee_name: String,
    pub selected_member_ids: Vec<String>,
    pub skipped_members: Vec<MemberActivationSkip>,
    pub estimated_memory_hint_mb: u32,
    pub runtime_status_by_member: Vec<MemberRuntimeStatusReport>,
    pub policy_notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarketCommitteeLayout {
    pub committee_name: String,
    pub market_scope: MarketScope,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MacMiniLocalPolicy {
    pub do_not_run_all_18_cores_concurrently: bool,
    pub lazy_activation: bool,
    pub offline_fixture_or_mock_until_runtime_exists: bool,
    pub quantized_local_runtime_later: bool,
    pub active_members_per_event_cycle_hint: String,
    pub unload_after_cycle: bool,
    pub risk_governor_lightweight_always_available: bool,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AiMemberCoreRegistry {
    pub core_specs: Vec<Mamba3GatedDeltaNetCoreSpec>,
    #[serde(default)]
    pub member_scope_bindings: Vec<MemberCoreScopeBinding>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemberCoreScopeBinding {
    pub member_id: String,
    pub market_scopes: Vec<MarketScope>,
}

impl AiMemberCoreRegistry {
    pub fn from_members(members: &[AICommitteeMember]) -> Self {
        Self::from_members_with_offline_hint(members, false)
    }

    pub fn from_members_with_offline_hint(
        members: &[AICommitteeMember],
        prefer_offline_fixture: bool,
    ) -> Self {
        Self {
            core_specs: members
                .iter()
                .map(|member| resolved_core_spec_for_member(member, prefer_offline_fixture))
                .collect(),
            member_scope_bindings: members
                .iter()
                .map(|member| MemberCoreScopeBinding {
                    member_id: member.member_id.clone(),
                    market_scopes: member.market_scopes.clone(),
                })
                .collect(),
        }
    }

    pub fn get_core_spec(&self, member_id: &str) -> Option<&Mamba3GatedDeltaNetCoreSpec> {
        self.core_specs
            .iter()
            .find(|spec| spec.member_id == member_id)
    }

    pub fn active_core_count(&self, policy: &MemberActivationPolicy, scope: MarketScope) -> usize {
        self.core_specs
            .iter()
            .filter(|spec| {
                !matches!(spec.load_policy, MemberCoreLoadPolicy::Disabled)
                    && self.core_spec_matches_scope(spec, scope)
                    && matches!(
                        spec.runtime_status,
                        CoreRuntimeStatus::OfflineFixture | CoreRuntimeStatus::MockLocal
                    )
            })
            .take(policy_active_limit(policy))
            .count()
    }

    pub fn estimate_cycle_memory_mb(
        &self,
        policy: &MemberActivationPolicy,
        scope: MarketScope,
    ) -> u32 {
        self.core_specs
            .iter()
            .filter(|spec| {
                !matches!(spec.load_policy, MemberCoreLoadPolicy::Disabled)
                    && self.core_spec_matches_scope(spec, scope)
                    && matches!(
                        spec.runtime_status,
                        CoreRuntimeStatus::OfflineFixture | CoreRuntimeStatus::MockLocal
                    )
            })
            .take(policy_active_limit(policy))
            .map(Mamba3GatedDeltaNetCoreSpec::memory_estimate_mb)
            .sum()
    }

    fn core_spec_matches_scope(
        &self,
        spec: &Mamba3GatedDeltaNetCoreSpec,
        scope: MarketScope,
    ) -> bool {
        self.member_scope_bindings
            .iter()
            .find(|binding| binding.member_id == spec.member_id)
            .map(|binding| binding.market_scopes.contains(&scope))
            .unwrap_or(true)
    }

    pub fn select_members_for_cycle(
        &self,
        members: &[AICommitteeMember],
        scope: MarketScope,
        event_optional: Option<&InvestmentEvent>,
        policy: &MemberActivationPolicy,
    ) -> MemberActivationPlan {
        let mut skipped_members = Vec::new();
        let mut eligible_indices = Vec::new();
        let mut runtime_status_by_member = Vec::new();

        for (index, member) in members.iter().enumerate() {
            let core_spec = self
                .get_core_spec(&member.member_id)
                .cloned()
                .unwrap_or_else(|| resolved_core_spec_for_member(member, false));
            runtime_status_by_member.push(MemberRuntimeStatusReport {
                member_id: member.member_id.clone(),
                runtime_status: core_spec.runtime_status,
                load_policy: core_spec.load_policy,
                memory_budget_mb_hint: core_spec.memory_budget_mb_hint,
            });

            let skip_reason = if !member.market_scopes.contains(&scope) {
                Some(MemberSelectionSkipReason::ScopeMismatch)
            } else if matches!(member.status, AICommitteeMemberStatus::Disabled) {
                Some(MemberSelectionSkipReason::Disabled)
            } else if matches!(member.status, AICommitteeMemberStatus::Demoted) {
                Some(MemberSelectionSkipReason::Demoted)
            } else if matches!(core_spec.load_policy, MemberCoreLoadPolicy::Disabled) {
                Some(MemberSelectionSkipReason::Disabled)
            } else if matches!(core_spec.runtime_status, CoreRuntimeStatus::RuntimeDeferred) {
                Some(MemberSelectionSkipReason::RuntimeDeferred)
            } else if matches!(
                core_spec.runtime_status,
                CoreRuntimeStatus::TrainingDeferred
            ) {
                Some(MemberSelectionSkipReason::TrainingDeferred)
            } else {
                None
            };

            if let Some(reason) = skip_reason {
                skipped_members.push(MemberActivationSkip {
                    member_id: member.member_id.clone(),
                    reason,
                });
            } else {
                eligible_indices.push(index);
            }
        }

        let limit = policy_active_limit(policy);
        let mut selected = Vec::new();
        if policy.include_risk_member_always {
            if let Some(index) = eligible_indices.iter().copied().find(|index| {
                members[*index]
                    .style_profile
                    .to_ascii_lowercase()
                    .contains("risk")
            }) {
                selected.push(index);
            }
        }
        if policy.include_recent_event_trigger_member {
            if let Some(event) = event_optional {
                if let Some(index) = eligible_indices
                    .iter()
                    .copied()
                    .find(|index| members[*index].member_id == event.proposed_by_member_id)
                {
                    if !selected.contains(&index) {
                        selected.push(index);
                    }
                }
            }
        }

        let mut remaining = eligible_indices.clone();
        if policy.prefer_high_voice_weight {
            remaining.sort_by(|left, right| {
                members[*right]
                    .voice_weight
                    .partial_cmp(&members[*left].voice_weight)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        for index in remaining {
            if selected.len() >= limit {
                break;
            }
            if !selected.contains(&index) {
                selected.push(index);
            }
        }
        selected.sort_unstable();

        let selected_member_ids: Vec<String> = selected
            .iter()
            .map(|index| members[*index].member_id.clone())
            .collect();
        for index in eligible_indices {
            if !selected.contains(&index) {
                skipped_members.push(MemberActivationSkip {
                    member_id: members[index].member_id.clone(),
                    reason: MemberSelectionSkipReason::OverActivationLimit,
                });
            }
        }

        let estimated_memory_hint_mb = selected
            .iter()
            .filter_map(|index| self.get_core_spec(&members[*index].member_id))
            .map(Mamba3GatedDeltaNetCoreSpec::memory_estimate_mb)
            .sum();

        MemberActivationPlan {
            market_scope: scope,
            committee_name: committee_name_for_scope(scope).to_string(),
            selected_member_ids,
            skipped_members,
            estimated_memory_hint_mb,
            runtime_status_by_member,
            policy_notes: mac_mini_local_policy().notes,
        }
    }
}

fn policy_active_limit(policy: &MemberActivationPolicy) -> usize {
    policy
        .max_active_members_per_cycle
        .min(policy.max_active_members_per_market_scope)
        .max(1)
}

pub fn market_committee_layouts() -> Vec<MarketCommitteeLayout> {
    [
        MarketScope::KoreaShortTerm,
        MarketScope::KoreaLongTerm,
        MarketScope::UsShortTerm,
        MarketScope::UsLongTerm,
        MarketScope::CryptoShortTerm,
        MarketScope::CryptoLongTerm,
    ]
    .into_iter()
    .map(|market_scope| MarketCommitteeLayout {
        committee_name: committee_name_for_scope(market_scope).to_string(),
        market_scope,
    })
    .collect()
}

pub fn mac_mini_local_policy() -> MacMiniLocalPolicy {
    MacMiniLocalPolicy {
        do_not_run_all_18_cores_concurrently: true,
        lazy_activation: true,
        offline_fixture_or_mock_until_runtime_exists: true,
        quantized_local_runtime_later: true,
        active_members_per_event_cycle_hint: "3-5".to_string(),
        unload_after_cycle: true,
        risk_governor_lightweight_always_available: true,
        notes: vec![
            "Mac mini policy: do not run 18 AI cores concurrently".to_string(),
            "lazy activation selects only relevant market-scope/event/risk members".to_string(),
            "Mamba3 + Gated DeltaNet runtime is contract-only and deferred".to_string(),
        ],
    }
}

pub fn create_three_member_pilot_roster(scope: MarketScope) -> Vec<AICommitteeMember> {
    create_three_member_pilot_roster_with_runtime(scope, CoreRuntimeStatus::MockLocal)
}

fn create_three_member_pilot_roster_for_scopes(
    mut scopes: Vec<MarketScope>,
    runtime_status: CoreRuntimeStatus,
) -> Vec<AICommitteeMember> {
    scopes.sort();
    scopes.dedup();
    let first_scope = scopes
        .first()
        .copied()
        .unwrap_or(MarketScope::KoreaShortTerm);
    create_three_member_pilot_roster_with_runtime(first_scope, runtime_status)
        .into_iter()
        .map(|mut member| {
            member.market_scopes = scopes.clone();
            member
        })
        .collect()
}

impl ArchetypeStyleCardRegistry {
    pub fn from_cards(cards: Vec<InvestorArchetypeStyleCard>) -> Self {
        let active_count = cards
            .iter()
            .filter(|card| matches!(card.status, StyleCardStatus::ActiveStyleCard))
            .count();
        let review_required_count = cards
            .iter()
            .filter(|card| matches!(card.status, StyleCardStatus::ReviewRequired))
            .count();
        let disabled_count = cards
            .iter()
            .filter(|card| matches!(card.status, StyleCardStatus::Disabled))
            .count();
        Self {
            cards,
            active_count,
            review_required_count,
            disabled_count,
        }
    }

    pub fn load_style_cards_from_local_fixture(path: &Path) -> Result<Self, String> {
        if !local_only(&path.to_string_lossy()) {
            return Err("archetype style card fixture path must be local".to_string());
        }
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        let cards: Vec<InvestorArchetypeStyleCard> =
            serde_json::from_str(&text).map_err(|err| err.to_string())?;
        Ok(Self::from_cards(cards))
    }

    pub fn get_card(&self, archetype_id: &str) -> Option<&InvestorArchetypeStyleCard> {
        self.cards
            .iter()
            .find(|card| card.archetype_id == archetype_id)
    }

    pub fn cards_for_role(&self, role: IndependentMemberRole) -> Vec<&InvestorArchetypeStyleCard> {
        self.cards
            .iter()
            .filter(|card| {
                !matches!(card.status, StyleCardStatus::Disabled)
                    && card_tags_match_role(card, role)
            })
            .collect()
    }

    pub fn cards_for_market_scope(&self, scope: MarketScope) -> Vec<&InvestorArchetypeStyleCard> {
        self.cards
            .iter()
            .filter(|card| {
                !matches!(card.status, StyleCardStatus::Disabled)
                    && market_bias_matches_scope(card.preferred_market_bias, scope)
            })
            .collect()
    }

    pub fn validate_no_impersonation(&self) -> Result<(), String> {
        let forbidden = ["warren buffett ai", "exact copy", "trades like"];
        for card in &self.cards {
            let haystack = format!(
                "{} {}",
                card.display_name.to_ascii_lowercase(),
                card.public_style_summary.to_ascii_lowercase()
            );
            if forbidden.iter().any(|phrase| haystack.contains(phrase)) {
                return Err(format!(
                    "archetype style card {} contains impersonation wording",
                    card.archetype_id
                ));
            }
            if !card
                .public_style_summary
                .to_ascii_lowercase()
                .contains("not a real person clone")
            {
                return Err(format!(
                    "archetype style card {} must state not a real person clone",
                    card.archetype_id
                ));
            }
        }
        Ok(())
    }

    pub fn validate_public_claim_boundaries(&self) -> Result<(), String> {
        self.validate_forbidden_public_claims(
            &[
                "private strategy",
                "private-strategy",
                "private data",
                "nonpublic",
                "inside information",
                "guaranteed return",
                "guaranteed returns",
                "guaranteed profit",
                "guaranteed profits",
                "would buy",
                "trades like",
                "exact copy",
            ],
            "unsafe public-claim",
        )
    }

    pub fn validate_no_private_strategy_claims(&self) -> Result<(), String> {
        self.validate_forbidden_public_claims(
            &[
                "private strategy",
                "private-strategy",
                "private data",
                "nonpublic",
                "inside information",
            ],
            "private-strategy",
        )
    }

    pub fn validate_no_guaranteed_return_claims(&self) -> Result<(), String> {
        self.validate_forbidden_public_claims(
            &[
                "guaranteed return",
                "guaranteed returns",
                "guaranteed profit",
                "guaranteed profits",
            ],
            "guaranteed-return",
        )
    }

    fn validate_forbidden_public_claims(
        &self,
        forbidden: &[&str],
        claim_label: &str,
    ) -> Result<(), String> {
        for card in &self.cards {
            let haystack = format!(
                "{} {}",
                card.display_name.to_ascii_lowercase(),
                card.public_style_summary.to_ascii_lowercase()
            );
            if forbidden.iter().any(|phrase| haystack.contains(phrase)) {
                return Err(format!(
                    "archetype style card {} contains {} wording",
                    card.archetype_id, claim_label
                ));
            }
        }
        Ok(())
    }

    pub fn validate_review_required_preserved(&self) -> Result<(), String> {
        for card in &self.cards {
            if matches!(card.source_confidence, SourceConfidence::ReviewRequired)
                && !matches!(card.status, StyleCardStatus::ReviewRequired)
            {
                return Err(format!(
                    "archetype style card {} must keep ReviewRequired status",
                    card.archetype_id
                ));
            }
        }
        Ok(())
    }

    pub fn validate_do_not_learn_guards(&self) -> Result<(), String> {
        let required = [
            "private_life_details",
            "exact_personality_clone",
            "unverified_profit_claims",
            "unsourced_quotes",
            "illegal_or_private_info",
        ];
        for card in &self.cards {
            if !required
                .iter()
                .all(|guard| card.do_not_learn.iter().any(|item| item == guard))
            {
                return Err(format!(
                    "archetype style card {} is missing do-not-learn guards",
                    card.archetype_id
                ));
            }
        }
        Ok(())
    }
}

impl RealArchetypeIntakePolicy {
    pub fn load_registry_from_local_json(
        &self,
        path: &Path,
    ) -> Result<ArchetypeStyleCardRegistry, String> {
        if self.accepts_local_json_only && !local_only(&path.to_string_lossy()) {
            return Err("real archetype intake path must be local JSON".to_string());
        }
        let registry = ArchetypeStyleCardRegistry::load_style_cards_from_local_fixture(path)?;
        self.validate_registry(&registry)?;
        Ok(registry)
    }

    pub fn validate_registry(&self, registry: &ArchetypeStyleCardRegistry) -> Result<(), String> {
        if self.validates_all_cards && registry.cards.is_empty() {
            return Err("real archetype intake requires at least one style card".to_string());
        }
        if self.rejects_impersonation_wording {
            registry.validate_no_impersonation()?;
        }
        if self.rejects_private_strategy_claims {
            registry.validate_no_private_strategy_claims()?;
        }
        if self.rejects_guaranteed_return_claims {
            registry.validate_no_guaranteed_return_claims()?;
        }
        if self.requires_do_not_learn_guards {
            registry.validate_do_not_learn_guards()?;
        }
        if self.review_required_stays_review_required {
            registry.validate_review_required_preserved()?;
        }
        if !(self.does_not_activate_live_agents
            && self.does_not_train_models
            && self.does_not_call_network)
        {
            return Err(
                "real archetype intake must not activate live agents, train, or call network"
                    .to_string(),
            );
        }
        Ok(())
    }
}

pub fn map_style_cards_to_three_member_pilot(
    registry: &ArchetypeStyleCardRegistry,
) -> ThreeMemberStyleMapping {
    let trend_entry_blend = build_member_style_blend(
        registry,
        "trend-kr-short",
        IndependentMemberRole::TrendEntry,
    );
    let risk_guard_blend =
        build_member_style_blend(registry, "risk-kr-short", IndependentMemberRole::RiskGuard);
    let evidence_regime_blend = build_member_style_blend(
        registry,
        "evidence-kr-short",
        IndependentMemberRole::EvidenceRegime,
    );
    let mapped: std::collections::BTreeSet<String> = trend_entry_blend
        .archetype_weights
        .iter()
        .chain(risk_guard_blend.archetype_weights.iter())
        .chain(evidence_regime_blend.archetype_weights.iter())
        .map(|weight| weight.archetype_id.clone())
        .collect();
    let unmapped_archetypes = registry
        .cards
        .iter()
        .filter(|card| !mapped.contains(&card.archetype_id))
        .map(|card| card.archetype_id.clone())
        .collect();
    let review_required_archetypes = registry
        .cards
        .iter()
        .filter(|card| {
            matches!(
                card.status,
                StyleCardStatus::ReviewRequired | StyleCardStatus::Disabled
            ) || matches!(
                card.source_confidence,
                SourceConfidence::Low | SourceConfidence::ReviewRequired
            )
        })
        .map(|card| card.archetype_id.clone())
        .collect();
    ThreeMemberStyleMapping {
        trend_entry_blend,
        risk_guard_blend,
        evidence_regime_blend,
        unmapped_archetypes,
        review_required_archetypes,
    }
}

pub fn style_influenced_profiles_for_mapping(
    registry: &ArchetypeStyleCardRegistry,
    mapping: &ThreeMemberStyleMapping,
) -> Vec<StyleInfluencedMemberProfile> {
    [
        &mapping.trend_entry_blend,
        &mapping.risk_guard_blend,
        &mapping.evidence_regime_blend,
    ]
    .into_iter()
    .map(|blend| {
        let cards = cards_for_blend(registry, blend);
        StyleInfluencedMemberProfile {
            member_id: blend.member_id.clone(),
            base_role: blend.role,
            style_blend: blend.clone(),
            risk_bias: dominant_risk_bias(&cards),
            time_horizon_bias: dominant_time_horizon(&cards),
            evidence_bias: dominant_evidence_preference(&cards),
            decision_bias_notes: vec![
                "style influence only; not a real investor clone".to_string(),
                "style blend cannot override Risk Governor or create orders".to_string(),
            ],
        }
    })
    .collect()
}

fn create_three_member_pilot_roster_with_runtime(
    scope: MarketScope,
    runtime_status: CoreRuntimeStatus,
) -> Vec<AICommitteeMember> {
    [
        (
            "trend-kr-short",
            "TrendEntryAI",
            IndependentMemberRole::TrendEntry,
            "trend",
            0.55,
            0.62,
        ),
        (
            "risk-kr-short",
            "RiskGuardAI",
            IndependentMemberRole::RiskGuard,
            "risk",
            0.50,
            0.64,
        ),
        (
            "evidence-kr-short",
            "EvidenceRegimeAI",
            IndependentMemberRole::EvidenceRegime,
            "evidence",
            0.45,
            0.58,
        ),
    ]
    .into_iter()
    .map(
        |(member_id, display_name, role, style_profile, voice_weight, score)| AICommitteeMember {
            member_id: member_id.to_string(),
            display_name: display_name.to_string(),
            market_scopes: vec![scope],
            style_profile: style_profile.to_string(),
            voice_weight,
            score,
            status: AICommitteeMemberStatus::Active,
            runtime_mode: AIRuntimeMode::MockLocal,
            core_spec: Some(Mamba3GatedDeltaNetCoreSpec::deferred_contract_for(
                member_id,
                runtime_status,
            )),
            role: Some(role),
            memory_state: Some(MemberMemoryState::new(member_id)),
        },
    )
    .collect()
}

fn committee_name_for_scope(scope: MarketScope) -> &'static str {
    match scope {
        MarketScope::KoreaShortTerm => "KoreaShortTermCommittee",
        MarketScope::KoreaLongTerm => "KoreaLongTermCommittee",
        MarketScope::UsShortTerm => "UsShortTermCommittee",
        MarketScope::UsLongTerm => "UsLongTermCommittee",
        MarketScope::CryptoShortTerm => "CryptoShortTermCommittee",
        MarketScope::CryptoLongTerm => "CryptoLongTermCommittee",
    }
}

fn resolved_core_spec_for_member(
    member: &AICommitteeMember,
    prefer_offline_fixture: bool,
) -> Mamba3GatedDeltaNetCoreSpec {
    if let Some(core_spec) = &member.core_spec {
        return core_spec.clone();
    }
    if prefer_offline_fixture {
        return Mamba3GatedDeltaNetCoreSpec::offline_fixture_for(&member.member_id);
    }
    match member.runtime_mode {
        AIRuntimeMode::MockLocal => Mamba3GatedDeltaNetCoreSpec::mock_local_for(&member.member_id),
        AIRuntimeMode::ExternalModelDeferred => {
            Mamba3GatedDeltaNetCoreSpec::runtime_deferred_for(&member.member_id)
        }
        AIRuntimeMode::TrainingDeferred => {
            Mamba3GatedDeltaNetCoreSpec::training_deferred_for(&member.member_id)
        }
    }
}

fn card_tags_match_role(card: &InvestorArchetypeStyleCard, role: IndependentMemberRole) -> bool {
    match role {
        IndependentMemberRole::TrendEntry => {
            card.primary_style_tags.iter().any(|tag| {
                matches!(
                    tag,
                    ArchetypeStyleTag::Trend
                        | ArchetypeStyleTag::Momentum
                        | ArchetypeStyleTag::EventDriven
                        | ArchetypeStyleTag::Growth
                )
            }) || matches!(card.evidence_preference, EvidencePreference::PriceAction)
        }
        IndependentMemberRole::RiskGuard => {
            matches!(card.risk_bias, ArchetypeRiskBias::Conservative)
                || card.primary_style_tags.iter().any(|tag| {
                    matches!(
                        tag,
                        ArchetypeStyleTag::RiskControl
                            | ArchetypeStyleTag::Volatility
                            | ArchetypeStyleTag::Liquidity
                            | ArchetypeStyleTag::Macro
                    )
                })
        }
        IndependentMemberRole::EvidenceRegime => {
            matches!(
                card.evidence_preference,
                EvidencePreference::Fundamentals
                    | EvidencePreference::Macro
                    | EvidencePreference::News
                    | EvidencePreference::Quant
                    | EvidencePreference::Mixed
            ) || card.primary_style_tags.iter().any(|tag| {
                matches!(
                    tag,
                    ArchetypeStyleTag::EvidenceQuality
                        | ArchetypeStyleTag::Quality
                        | ArchetypeStyleTag::Macro
                )
            })
        }
    }
}

fn market_bias_matches_scope(bias: PreferredMarketBias, scope: MarketScope) -> bool {
    match bias {
        PreferredMarketBias::Any | PreferredMarketBias::Global => true,
        PreferredMarketBias::Korea => {
            matches!(
                scope,
                MarketScope::KoreaShortTerm | MarketScope::KoreaLongTerm
            )
        }
        PreferredMarketBias::US => {
            matches!(scope, MarketScope::UsShortTerm | MarketScope::UsLongTerm)
        }
        PreferredMarketBias::Crypto => {
            matches!(
                scope,
                MarketScope::CryptoShortTerm | MarketScope::CryptoLongTerm
            )
        }
    }
}

fn build_member_style_blend(
    registry: &ArchetypeStyleCardRegistry,
    member_id: &str,
    role: IndependentMemberRole,
) -> MemberStyleBlend {
    let mut candidates: Vec<&InvestorArchetypeStyleCard> = registry.cards_for_role(role);
    candidates.sort_by(|left, right| {
        style_card_base_weight(right, role)
            .partial_cmp(&style_card_base_weight(left, role))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let scored: Vec<(&InvestorArchetypeStyleCard, f64)> = candidates
        .into_iter()
        .take(6)
        .map(|card| (card, style_card_base_weight(card, role)))
        .collect();
    let total: f64 = scored.iter().map(|(_, weight)| weight).sum();
    let archetype_weights = if total > 0.0 {
        scored
            .iter()
            .map(|(card, weight)| MemberArchetypeWeight {
                archetype_id: card.archetype_id.clone(),
                weight: (weight / total * 10_000.0).round() / 10_000.0,
            })
            .collect()
    } else {
        Vec::new()
    };
    let source_confidence_minimum = lowest_safe_source_confidence(&scored);
    let has_review_warning = scored.iter().any(|(card, _)| {
        matches!(
            card.status,
            StyleCardStatus::ReviewRequired | StyleCardStatus::Disabled
        ) || matches!(
            card.source_confidence,
            SourceConfidence::Low | SourceConfidence::ReviewRequired
        )
    });
    MemberStyleBlend {
        member_id: member_id.to_string(),
        role,
        archetype_weights: normalize_weights(archetype_weights),
        blend_summary: format!(
            "{:?} receives public-philosophy-inspired style influence only",
            role
        ),
        prohibited_claims: vec![
            "not a real person clone".to_string(),
            "does not trade like any named investor".to_string(),
            "does not use private strategy or private data".to_string(),
            "does not imply guaranteed returns".to_string(),
        ],
        source_confidence_minimum,
        style_status: if has_review_warning {
            MemberStyleStatus::ReadyWithWarnings
        } else {
            MemberStyleStatus::Ready
        },
    }
}

fn normalize_weights(mut weights: Vec<MemberArchetypeWeight>) -> Vec<MemberArchetypeWeight> {
    if weights.is_empty() {
        return weights;
    }
    let sum: f64 = weights.iter().map(|weight| weight.weight).sum();
    if sum <= 0.0 {
        return weights;
    }
    for weight in &mut weights {
        weight.weight = (weight.weight / sum * 10_000.0).round() / 10_000.0;
    }
    let adjusted_sum: f64 = weights.iter().map(|weight| weight.weight).sum();
    if let Some(first) = weights.first_mut() {
        first.weight = ((first.weight + (1.0 - adjusted_sum)) * 10_000.0).round() / 10_000.0;
    }
    weights
}

fn lowest_safe_source_confidence(
    scored: &[(&InvestorArchetypeStyleCard, f64)],
) -> SourceConfidence {
    if scored.iter().any(|(card, _)| {
        matches!(card.status, StyleCardStatus::ReviewRequired)
            || matches!(card.source_confidence, SourceConfidence::ReviewRequired)
    }) {
        SourceConfidence::ReviewRequired
    } else if scored
        .iter()
        .any(|(card, _)| matches!(card.source_confidence, SourceConfidence::Low))
    {
        SourceConfidence::Low
    } else if scored
        .iter()
        .any(|(card, _)| matches!(card.source_confidence, SourceConfidence::Medium))
    {
        SourceConfidence::Medium
    } else {
        SourceConfidence::High
    }
}

fn style_card_base_weight(card: &InvestorArchetypeStyleCard, role: IndependentMemberRole) -> f64 {
    let mut weight = if card_tags_match_role(card, role) {
        1.0
    } else {
        0.0
    };
    weight *= match card.source_confidence {
        SourceConfidence::High => 1.0,
        SourceConfidence::Medium => 0.75,
        SourceConfidence::Low => 0.25,
        SourceConfidence::ReviewRequired => 0.15,
    };
    if matches!(card.status, StyleCardStatus::ReviewRequired) {
        weight *= 0.5;
    }
    if matches!(card.status, StyleCardStatus::Disabled) {
        0.0
    } else {
        weight
    }
}

fn cards_for_blend<'a>(
    registry: &'a ArchetypeStyleCardRegistry,
    blend: &MemberStyleBlend,
) -> Vec<&'a InvestorArchetypeStyleCard> {
    blend
        .archetype_weights
        .iter()
        .filter_map(|weight| registry.get_card(&weight.archetype_id))
        .collect()
}

fn dominant_risk_bias(cards: &[&InvestorArchetypeStyleCard]) -> ArchetypeRiskBias {
    if cards
        .iter()
        .any(|card| matches!(card.risk_bias, ArchetypeRiskBias::Conservative))
    {
        ArchetypeRiskBias::Conservative
    } else if cards
        .iter()
        .any(|card| matches!(card.risk_bias, ArchetypeRiskBias::Aggressive))
    {
        ArchetypeRiskBias::Aggressive
    } else if cards
        .iter()
        .any(|card| matches!(card.risk_bias, ArchetypeRiskBias::Balanced))
    {
        ArchetypeRiskBias::Balanced
    } else {
        ArchetypeRiskBias::Unknown
    }
}

fn dominant_time_horizon(cards: &[&InvestorArchetypeStyleCard]) -> PreferredTimeHorizon {
    if cards
        .iter()
        .any(|card| matches!(card.preferred_time_horizon, PreferredTimeHorizon::ShortTerm))
    {
        PreferredTimeHorizon::ShortTerm
    } else if cards
        .iter()
        .any(|card| matches!(card.preferred_time_horizon, PreferredTimeHorizon::LongTerm))
    {
        PreferredTimeHorizon::LongTerm
    } else {
        PreferredTimeHorizon::MultiHorizon
    }
}

fn dominant_evidence_preference(cards: &[&InvestorArchetypeStyleCard]) -> EvidencePreference {
    cards
        .first()
        .map(|card| card.evidence_preference)
        .unwrap_or(EvidencePreference::Mixed)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfflineMemberOpinionFixture {
    pub member_id: String,
    pub symbol: String,
    pub market_scope: MarketScope,
    pub stance: MemberStance,
    pub confidence: f64,
    pub expected_return_hint: f64,
    pub risk_hint: f64,
    pub evidence_notes: Vec<String>,
    pub event_triggered: bool,
    pub event_reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfflineMemberBrainAdapter {
    pub fixtures: Vec<OfflineMemberOpinionFixture>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfflineMemberOutputBatch {
    pub batch_id: String,
    pub created_at: String,
    pub source_label: String,
    pub opinions: Vec<OfflineMemberOpinionFixture>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfflineMemberOutputBatchLoadResult {
    pub batch_id: String,
    pub loaded_count: usize,
    pub invalid_count: usize,
    pub duplicate_count: usize,
    pub unmatched_count: usize,
    pub opinions: Vec<OfflineMemberOpinionFixture>,
    pub safety_notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct OfflineOpinionKey {
    member_id: String,
    symbol: String,
    market_scope: MarketScope,
}

impl OfflineOpinionKey {
    fn from_fixture(fixture: &OfflineMemberOpinionFixture) -> Self {
        Self {
            member_id: fixture.member_id.clone(),
            symbol: fixture.symbol.clone(),
            market_scope: fixture.market_scope,
        }
    }

    fn from_packet(packet: &MemberInputPacket) -> Self {
        Self {
            member_id: packet.member_id.clone(),
            symbol: packet.market_data.symbol.clone(),
            market_scope: packet.market_data.market_scope,
        }
    }
}

impl OfflineMemberOutputBatch {
    pub fn from_json_path(path: &Path) -> Result<OfflineMemberOutputBatchLoadResult, String> {
        if !local_only(&path.to_string_lossy()) {
            return Err("offline member output batch path must be local".to_string());
        }
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_json_str(&text)
    }

    pub fn from_json_str(text: &str) -> Result<OfflineMemberOutputBatchLoadResult, String> {
        reject_unsafe_offline_batch_text(text)?;
        let batch: OfflineMemberOutputBatch =
            serde_json::from_str(text).map_err(|err| err.to_string())?;
        let mut seen = std::collections::BTreeSet::new();
        let mut opinions = Vec::new();
        let mut invalid_count = 0;
        let mut duplicate_count = 0;
        for opinion in batch.opinions {
            if opinion.member_id.trim().is_empty()
                || opinion.symbol.trim().is_empty()
                || opinion
                    .evidence_notes
                    .iter()
                    .any(|note| note.trim().is_empty())
            {
                invalid_count += 1;
                continue;
            }
            if !seen.insert(OfflineOpinionKey::from_fixture(&opinion)) {
                duplicate_count += 1;
                continue;
            }
            opinions.push(opinion);
        }
        Ok(OfflineMemberOutputBatchLoadResult {
            batch_id: batch.batch_id,
            loaded_count: opinions.len(),
            invalid_count,
            duplicate_count,
            unmatched_count: 0,
            opinions,
            safety_notes: vec![
                "local offline member output batch only".to_string(),
                "no network, no live inference, no model training".to_string(),
                "no broker/order/account or live execution fields accepted".to_string(),
            ],
        })
    }
}

fn reject_unsafe_offline_batch_text(text: &str) -> Result<(), String> {
    let value: serde_json::Value = serde_json::from_str(text).map_err(|err| err.to_string())?;
    reject_unsafe_offline_batch_value(&value)
}

fn reject_unsafe_offline_batch_value(value: &serde_json::Value) -> Result<(), String> {
    match value {
        serde_json::Value::Object(map) => {
            for (key, value) in map {
                let lower = key.to_ascii_lowercase();
                if ["broker", "order", "account"]
                    .iter()
                    .any(|fragment| lower.contains(fragment))
                {
                    return Err(format!(
                        "offline member output batch rejected unsafe field: {key}"
                    ));
                }
                reject_unsafe_offline_batch_value(value)?;
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                reject_unsafe_offline_batch_value(item)?;
            }
        }
        serde_json::Value::String(text) => {
            let lower = text.to_ascii_lowercase();
            for prohibited in [
                "place order",
                "execute order",
                "submit order",
                "broker account",
                "guaranteed return",
                "guaranteed returns",
                "guaranteed profit",
                "guaranteed profits",
                "impersonate",
                "warren buffett ai",
                "exact copy",
                "trades like",
                "private strategy",
                "private-strategy",
                "private data",
                "nonpublic",
                "inside information",
                "live execution",
                "live trading",
                "live inference",
                "model training",
            ] {
                if lower.contains(prohibited) {
                    return Err(format!(
                        "offline member output batch rejected unsafe claim: {prohibited}"
                    ));
                }
            }
        }
        _ => {}
    }
    Ok(())
}

impl OfflineMemberBrainAdapter {
    pub fn from_json_path(path: &Path) -> Result<Self, String> {
        if !local_only(&path.to_string_lossy()) {
            return Err("offline member opinion fixture path must be local".to_string());
        }
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        let fixtures: Vec<OfflineMemberOpinionFixture> =
            serde_json::from_str(&text).map_err(|err| err.to_string())?;
        Ok(Self { fixtures })
    }

    fn fallback_opinion(packet: &MemberInputPacket) -> MemberOpinion {
        MemberOpinion {
            member_id: packet.member_id.clone(),
            symbol: packet.market_data.symbol.clone(),
            market_scope: packet.market_data.market_scope,
            stance: MemberStance::NeedMoreEvidence,
            confidence: 0.5,
            expected_return_hint: 0.0,
            risk_hint: clamp_unit(packet.market_data.volatility_hint),
            evidence_notes: vec![
                "offline fixture missing for member/symbol/scope".to_string(),
                "NeedMoreEvidence fallback; no external model, no training, no order".to_string(),
            ],
            event_triggered: false,
            event_reason: Some("offline fixture missing".to_string()),
        }
    }
}

impl AiMemberBrain for OfflineMemberBrainAdapter {
    fn produce_opinion(&self, packet: &MemberInputPacket) -> MemberOpinion {
        if let Some(fixture) = self.fixtures.iter().find(|fixture| {
            fixture.member_id == packet.member_id
                && fixture.symbol == packet.market_data.symbol
                && fixture.market_scope == packet.market_data.market_scope
        }) {
            return MemberOpinion {
                member_id: fixture.member_id.clone(),
                symbol: fixture.symbol.clone(),
                market_scope: fixture.market_scope,
                stance: fixture.stance,
                confidence: clamp_unit(fixture.confidence),
                expected_return_hint: fixture.expected_return_hint,
                risk_hint: clamp_unit(fixture.risk_hint),
                evidence_notes: fixture.evidence_notes.clone(),
                event_triggered: fixture.event_triggered,
                event_reason: fixture.event_reason.clone(),
            };
        }

        Self::fallback_opinion(packet)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CoreAwareMemberBrainAdapter {
    pub member: AICommitteeMember,
    pub core_spec: Mamba3GatedDeltaNetCoreSpec,
    pub offline_adapter: OfflineMemberBrainAdapter,
}

impl AiMemberBrain for CoreAwareMemberBrainAdapter {
    fn produce_opinion(&self, packet: &MemberInputPacket) -> MemberOpinion {
        match self.core_spec.runtime_status {
            CoreRuntimeStatus::OfflineFixture => self.offline_adapter.produce_opinion(packet),
            CoreRuntimeStatus::MockLocal => DeterministicMockBrain {
                member: self.member.clone(),
            }
            .produce_opinion(packet),
            CoreRuntimeStatus::RuntimeDeferred => deferred_core_opinion(packet, "runtime deferred"),
            CoreRuntimeStatus::TrainingDeferred => {
                deferred_core_opinion(packet, "training deferred")
            }
        }
    }
}

pub fn route_data_to_ai_members(input: DataRouterInput) -> DataRouterOutput {
    let mut packets = Vec::new();
    let mut unrouted_symbol_count = 0;

    for market_data in input.market_data {
        let relevant_members: Vec<&AICommitteeMember> = input
            .members
            .iter()
            .filter(|member| {
                !matches!(member.status, AICommitteeMemberStatus::Disabled)
                    && member.market_scopes.contains(&market_data.market_scope)
            })
            .collect();

        if relevant_members.is_empty() {
            unrouted_symbol_count += 1;
            continue;
        }

        let news: Vec<NewsSnapshot> = input
            .news
            .iter()
            .filter(|item| item.symbol == market_data.symbol)
            .cloned()
            .collect();

        for member in relevant_members {
            packets.push(MemberInputPacket {
                member_id: member.member_id.clone(),
                market_data: market_data.clone(),
                news: news.clone(),
                owner_context: input.owner_context.clone(),
                previous_member_score: Some(member.score),
            });
        }
    }

    DataRouterOutput {
        routed_member_count: packets.len(),
        packets,
        unrouted_symbol_count,
        safety_notes: vec![
            "program routes market/news snapshots only".to_string(),
            "DataRouter does not create opinions, buy/sell recommendations, or orders".to_string(),
            "AI members judge through the member brain boundary".to_string(),
        ],
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberStance {
    BuyProposal,
    SellProposal,
    Hold,
    NoTrade,
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemberOpinion {
    pub member_id: String,
    pub symbol: String,
    pub market_scope: MarketScope,
    pub stance: MemberStance,
    pub confidence: f64,
    pub expected_return_hint: f64,
    pub risk_hint: f64,
    pub evidence_notes: Vec<String>,
    pub event_triggered: bool,
    pub event_reason: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvestmentEventType {
    EntryProposal,
    ExitProposal,
    RiskWarning,
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InvestmentEvent {
    pub event_id: String,
    pub proposed_by_member_id: String,
    pub symbol: String,
    pub market_scope: MarketScope,
    pub event_type: InvestmentEventType,
    pub triggering_opinion: MemberOpinion,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InvestmentEventQueue {
    pub queue_id: String,
    pub events: Vec<InvestmentEvent>,
    pub event_count: usize,
    pub symbols: Vec<String>,
    pub market_scopes: Vec<MarketScope>,
}

impl InvestmentEventQueue {
    pub fn from_member_opinions(opinions: &[MemberOpinion]) -> Self {
        let mut events: Vec<InvestmentEvent> = opinions
            .iter()
            .filter(|opinion| opinion.event_triggered)
            .map(event_from_opinion)
            .collect();
        events = Self {
            queue_id: "offline-batch-event-queue".to_string(),
            events,
            event_count: 0,
            symbols: Vec::new(),
            market_scopes: Vec::new(),
        }
        .risk_first_ordering();
        Self::from_events(events)
    }

    pub fn group_by_symbol(&self) -> std::collections::BTreeMap<String, Vec<InvestmentEvent>> {
        let mut grouped: std::collections::BTreeMap<String, Vec<InvestmentEvent>> =
            std::collections::BTreeMap::new();
        for event in &self.events {
            grouped
                .entry(event.symbol.clone())
                .or_default()
                .push(event.clone());
        }
        grouped
    }

    pub fn group_by_market_scope(
        &self,
    ) -> std::collections::BTreeMap<MarketScope, Vec<InvestmentEvent>> {
        let mut grouped: std::collections::BTreeMap<MarketScope, Vec<InvestmentEvent>> =
            std::collections::BTreeMap::new();
        for event in &self.events {
            grouped
                .entry(event.market_scope)
                .or_default()
                .push(event.clone());
        }
        grouped
    }

    pub fn highest_confidence_event(&self) -> Option<&InvestmentEvent> {
        self.events.iter().max_by(|left, right| {
            left.triggering_opinion
                .confidence
                .partial_cmp(&right.triggering_opinion.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn risk_first_ordering(mut self) -> Vec<InvestmentEvent> {
        self.events.sort_by(|left, right| {
            event_priority(left)
                .cmp(&event_priority(right))
                .then_with(|| {
                    right
                        .triggering_opinion
                        .confidence
                        .partial_cmp(&left.triggering_opinion.confidence)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .then_with(|| left.event_id.cmp(&right.event_id))
        });
        self.events
    }

    fn from_events(events: Vec<InvestmentEvent>) -> Self {
        let mut symbols: Vec<String> = events.iter().map(|event| event.symbol.clone()).collect();
        symbols.sort();
        symbols.dedup();
        let mut market_scopes: Vec<MarketScope> =
            events.iter().map(|event| event.market_scope).collect();
        market_scopes.sort();
        market_scopes.dedup();
        Self {
            queue_id: "offline-batch-event-queue".to_string(),
            event_count: events.len(),
            events,
            symbols,
            market_scopes,
        }
    }
}

fn event_from_opinion(opinion: &MemberOpinion) -> InvestmentEvent {
    let event_type = match opinion.stance {
        MemberStance::BuyProposal => InvestmentEventType::EntryProposal,
        MemberStance::SellProposal => InvestmentEventType::ExitProposal,
        MemberStance::NoTrade => InvestmentEventType::RiskWarning,
        MemberStance::NeedMoreEvidence | MemberStance::Hold => {
            InvestmentEventType::NeedMoreEvidence
        }
    };
    InvestmentEvent {
        event_id: format!(
            "event-{}-{:?}-{}",
            opinion.symbol, opinion.market_scope, opinion.member_id
        ),
        proposed_by_member_id: opinion.member_id.clone(),
        symbol: opinion.symbol.clone(),
        market_scope: opinion.market_scope,
        event_type,
        triggering_opinion: opinion.clone(),
        created_at: "offline-batch".to_string(),
    }
}

fn event_priority(event: &InvestmentEvent) -> u8 {
    match event.event_type {
        InvestmentEventType::RiskWarning => 0,
        InvestmentEventType::EntryProposal | InvestmentEventType::ExitProposal => 1,
        InvestmentEventType::NeedMoreEvidence => 2,
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeSession {
    pub session_id: String,
    pub event: InvestmentEvent,
    pub invited_members: Vec<String>,
    pub member_opinions: Vec<MemberOpinion>,
    pub disagreement_level: f64,
    pub risk_flags: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChairmanFinalAction {
    PaperBuy,
    PaperSell,
    PaperHold,
    PaperNoTrade,
    NeedMoreEvidence,
    RiskVetoed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskGovernorStatus {
    Passed,
    Vetoed,
    NeedsReview,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChairmanDecision {
    pub decision_id: String,
    pub session_id: String,
    pub final_action: ChairmanFinalAction,
    pub rationale: String,
    pub winning_arguments: Vec<String>,
    pub dissenting_arguments: Vec<String>,
    pub risk_governor_status: RiskGovernorStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberScoreUpdateReason {
    GoodCall,
    BadCall,
    RiskyCall,
    HelpfulDissent,
    LowEvidence,
    Neutral,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemberScoreUpdate {
    pub member_id: String,
    pub previous_score: f64,
    pub new_score: f64,
    pub previous_voice_weight: f64,
    pub new_voice_weight: f64,
    pub update_reason: MemberScoreUpdateReason,
    pub promoted: bool,
    pub demoted: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SimulatedPaperOutcome {
    Positive,
    Negative,
    Neutral,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberLearningSignal {
    Reinforce,
    Penalize,
    Watch,
    Ignore,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemberLearningJournalEntry {
    pub journal_id: String,
    pub member_id: String,
    pub symbol: String,
    pub market_scope: MarketScope,
    pub opinion_stance: MemberStance,
    pub confidence: f64,
    pub chairman_action: Option<ChairmanFinalAction>,
    pub risk_governor_status: Option<RiskGovernorStatus>,
    pub simulated_outcome: SimulatedPaperOutcome,
    pub learning_signal: MemberLearningSignal,
    pub note: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemberLearningJournalSummary {
    pub reinforce_count: usize,
    pub penalize_count: usize,
    pub watch_count: usize,
    pub ignore_count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemberLearningJournal {
    pub member_id: String,
    pub entries: Vec<MemberLearningJournalEntry>,
    pub summary: MemberLearningJournalSummary,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaperOutcomeFeedback {
    pub symbol: String,
    pub market_scope: MarketScope,
    pub decision_id: String,
    pub simulated_result: SimulatedPaperOutcome,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaperOutcomeFeedbackResult {
    pub score_updates: Vec<MemberScoreUpdate>,
    pub learning_journal_entries: Vec<MemberLearningJournalEntry>,
    pub updated_memory_states: Vec<MemberMemoryState>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemberRoleReport {
    pub member_id: String,
    pub role: IndependentMemberRole,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MinimalCommitteeCycleInput {
    pub market_data: MarketDataSnapshot,
    pub news: Vec<NewsSnapshot>,
    pub members: Vec<AICommitteeMember>,
    #[serde(default)]
    pub activation_policy: MemberActivationPolicy,
    #[serde(default)]
    pub offline_member_opinions: Vec<OfflineMemberOpinionFixture>,
    #[serde(default)]
    pub archetype_style_cards: Vec<InvestorArchetypeStyleCard>,
    #[serde(default = "default_style_mapping_mode")]
    pub style_mapping_mode: StyleMappingMode,
    #[serde(default)]
    pub owner_context: Option<String>,
    #[serde(default = "default_positive_outcome")]
    pub simulated_outcome_return: Option<f64>,
    #[serde(default)]
    pub paper_outcome_feedback: Option<PaperOutcomeFeedback>,
    #[serde(default = "default_risk_veto_volatility_threshold")]
    pub risk_veto_volatility_threshold: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BatchCommitteeCycleInput {
    pub market_data: Vec<MarketDataSnapshot>,
    pub news: Vec<NewsSnapshot>,
    pub members: Vec<AICommitteeMember>,
    #[serde(default)]
    pub offline_output_batch: Option<OfflineMemberOutputBatch>,
    #[serde(default)]
    pub owner_context: Option<String>,
    #[serde(default)]
    pub paper_outcome: Option<SimulatedPaperOutcome>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MinimalCommitteeSafetySummary {
    pub paper_only: bool,
    pub no_real_order_path: bool,
    pub no_broker_order_account: bool,
    pub no_model_training: bool,
    pub no_live_inference: bool,
    pub runtime_mode: String,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MinimalCommitteeCycleResult {
    pub selected_scope: MarketScope,
    pub symbol: String,
    pub routed_packet_count: usize,
    pub triggered_event_count: usize,
    pub committee_session_count: usize,
    pub distributed_member_ids: Vec<String>,
    pub member_opinions: Vec<MemberOpinion>,
    pub event: Option<InvestmentEvent>,
    pub committee_session: Option<CommitteeSession>,
    pub chairman_decision: Option<ChairmanDecision>,
    pub score_updates: Vec<MemberScoreUpdate>,
    pub activation_plan: MemberActivationPlan,
    pub member_roles: Vec<MemberRoleReport>,
    pub memory_states: Vec<MemberMemoryState>,
    pub learning_journal_entries: Vec<MemberLearningJournalEntry>,
    pub learning_journal_entry_count: usize,
    pub learning_journals: Vec<MemberLearningJournal>,
    pub style_card_registry: Option<ArchetypeStyleCardRegistry>,
    pub three_member_style_mapping: Option<ThreeMemberStyleMapping>,
    pub style_influenced_profiles: Vec<StyleInfluencedMemberProfile>,
    pub safety_summary: MinimalCommitteeSafetySummary,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BatchCommitteeCycleResult {
    pub batch_id: String,
    pub routed_packet_count: usize,
    pub member_opinion_count: usize,
    pub member_opinions: Vec<MemberOpinion>,
    pub event_queue: InvestmentEventQueue,
    pub events_by_symbol: std::collections::BTreeMap<String, usize>,
    pub committee_sessions: Vec<CommitteeSession>,
    pub chairman_decisions: Vec<ChairmanDecision>,
    pub risk_veto_count: usize,
    pub score_updates: Vec<MemberScoreUpdate>,
    pub score_update_count: usize,
    pub memory_updates: Vec<MemberMemoryState>,
    pub learning_journal_entries: Vec<MemberLearningJournalEntry>,
    pub learning_journal_entry_count: usize,
    pub learning_journals: Vec<MemberLearningJournal>,
    pub safety_summary: MinimalCommitteeSafetySummary,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemberStateSnapshot {
    pub member_id: String,
    pub score: f64,
    pub voice_weight: f64,
    pub status: AICommitteeMemberStatus,
    pub memory_state: MemberMemoryState,
    pub learning_journal_summary: MemberLearningJournalSummary,
    pub last_updated_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemberStateStore {
    pub store_id: String,
    pub members: Vec<MemberStateSnapshot>,
    pub source_label: String,
    pub paper_only: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BatchCycleStateUpdate {
    pub cycle_id: String,
    pub score_updates: Vec<MemberScoreUpdate>,
    pub memory_updates: Vec<MemberMemoryState>,
    pub learning_journal_entries: Vec<MemberLearningJournalEntry>,
    pub updated_member_states: Vec<MemberStateSnapshot>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemberVoiceChange {
    pub member_id: String,
    pub previous_voice_weight: f64,
    pub new_voice_weight: f64,
    pub direction: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OwnerCommitteeSummary {
    pub cycle_id: String,
    pub symbols_reviewed: Vec<String>,
    pub event_count: usize,
    pub risk_veto_count: usize,
    pub paper_buy_count: usize,
    pub paper_hold_count: usize,
    pub no_trade_count: usize,
    pub need_more_evidence_count: usize,
    pub top_supporting_members: Vec<String>,
    pub top_dissenting_members: Vec<String>,
    pub member_voice_changes: Vec<MemberVoiceChange>,
    pub chairman_actions: Vec<ChairmanFinalAction>,
    pub risk_warnings: Vec<String>,
    pub owner_readable_summary: String,
    pub paper_only_warning: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OwnerCommitteeConsoleView {
    pub view_id: String,
    pub cycle_id: String,
    pub reviewed_symbols: Vec<String>,
    pub active_members: Vec<String>,
    pub member_status_rows: Vec<MemberStatusRow>,
    pub event_rows: Vec<EventRow>,
    pub committee_rows: Vec<CommitteeRow>,
    pub chairman_decision_rows: Vec<ChairmanDecisionRow>,
    pub risk_veto_rows: Vec<RiskVetoRow>,
    pub voice_change_rows: Vec<VoiceChangeRow>,
    pub next_action_rows: Vec<NextActionRow>,
    pub paper_only_warning: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemberStatusRow {
    pub member_id: String,
    pub display_name: String,
    pub role: Option<IndependentMemberRole>,
    pub score: f64,
    pub voice_weight: f64,
    pub status: AICommitteeMemberStatus,
    pub runtime_status: String,
    pub style_summary_short: String,
    pub last_opinion_summary: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EventRow {
    pub event_id: String,
    pub symbol: String,
    pub market_scope: MarketScope,
    pub proposed_by_member_id: String,
    pub event_type: InvestmentEventType,
    pub confidence: f64,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeRow {
    pub session_id: String,
    pub symbol: String,
    pub market_scope: MarketScope,
    pub invited_members: Vec<String>,
    pub disagreement_level: f64,
    pub risk_flags: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChairmanDecisionRow {
    pub decision_id: String,
    pub symbol: String,
    pub final_action: ChairmanFinalAction,
    pub rationale_short: String,
    pub risk_governor_status: RiskGovernorStatus,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RiskVetoRow {
    pub symbol: String,
    pub reason: String,
    pub blocked_action: String,
    pub risk_member_support: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VoiceChangeRow {
    pub member_id: String,
    pub previous_voice_weight: f64,
    pub new_voice_weight: f64,
    pub reason: MemberScoreUpdateReason,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NextActionType {
    Watch,
    NeedMoreEvidence,
    PaperReview,
    NoTrade,
    RiskBlocked,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NextActionRow {
    pub symbol: Option<String>,
    pub action_type: NextActionType,
    pub note: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnerFeedbackType {
    Comment,
    Disagree,
    RiskConcern,
    EvidenceRequest,
    WatchlistRequest,
    PaperOutcomeLabel,
    ReconsiderationRequest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnerFeedbackPriority {
    Low,
    Normal,
    High,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OwnerFeedback {
    pub feedback_id: String,
    pub symbol: Option<String>,
    pub market_scope: Option<MarketScope>,
    pub target_member_id: Option<String>,
    pub feedback_type: OwnerFeedbackType,
    pub text: String,
    pub priority: OwnerFeedbackPriority,
    pub created_at: Option<String>,
    pub paper_only: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OwnerFeedbackPacket {
    pub feedback: OwnerFeedback,
    pub related_market_data: Option<MarketDataSnapshot>,
    pub related_news: Vec<NewsSnapshot>,
    pub related_previous_opinions: Vec<MemberOpinion>,
    pub related_chairman_decision: Option<ChairmanDecision>,
    pub related_risk_status: Option<RiskGovernorStatus>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemberReconsiderationOpinion {
    pub member_id: String,
    pub symbol: String,
    pub market_scope: MarketScope,
    pub previous_stance: MemberStance,
    pub revised_stance: MemberStance,
    pub confidence_before: f64,
    pub confidence_after: f64,
    pub changed: bool,
    pub reason: String,
    pub evidence_needed: Vec<String>,
    pub risk_notes: Vec<String>,
    pub event_triggered: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeReconsiderationSession {
    pub session_id: String,
    pub original_session_id: Option<String>,
    pub owner_feedback: OwnerFeedback,
    pub invited_members: Vec<String>,
    pub revised_opinions: Vec<MemberReconsiderationOpinion>,
    pub disagreement_level: f64,
    pub risk_flags: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChairmanReconsiderationFinalAction {
    PaperBuy,
    PaperSell,
    PaperHold,
    PaperNoTrade,
    NeedMoreEvidence,
    RiskVetoed,
    KeepPreviousDecision,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChairmanReconsiderationDecision {
    pub decision_id: String,
    pub reconsideration_session_id: String,
    pub final_action: ChairmanReconsiderationFinalAction,
    pub rationale: String,
    pub what_changed: Vec<String>,
    pub what_did_not_change: Vec<String>,
    pub risk_governor_status: RiskGovernorStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnerFeedbackOutcome {
    ChangedDecision,
    KeptDecision,
    NeedMoreEvidence,
    RiskBlocked,
    LoggedOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OwnerFeedbackJournalEntry {
    pub feedback_id: String,
    pub symbol: Option<String>,
    pub routed_to_members: Vec<String>,
    pub reconsideration_opened: bool,
    pub decision_id: Option<String>,
    pub outcome: OwnerFeedbackOutcome,
    pub note: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OwnerFeedbackReconsiderationInput {
    pub previous_batch_result: BatchCommitteeCycleResult,
    pub previous_owner_console_view: OwnerCommitteeConsoleView,
    pub owner_feedback: Vec<OwnerFeedback>,
    pub member_state_store: MemberStateStore,
    #[serde(default)]
    pub market_data: Vec<MarketDataSnapshot>,
    #[serde(default)]
    pub news: Vec<NewsSnapshot>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OwnerFeedbackReconsiderationResult {
    pub owner_feedback_count: usize,
    pub routed_feedback_packets: Vec<OwnerFeedbackPacket>,
    pub revised_member_opinions: Vec<MemberReconsiderationOpinion>,
    pub reconsideration_sessions: Vec<CommitteeReconsiderationSession>,
    pub chairman_reconsideration_decisions: Vec<ChairmanReconsiderationDecision>,
    pub owner_feedback_journal_entries: Vec<OwnerFeedbackJournalEntry>,
    pub updated_owner_console_view: OwnerCommitteeConsoleView,
    pub paper_only_warning: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutonomousPaperCycleMode {
    SingleShot,
    FixedCount,
    ManualStep,
}

fn default_autonomous_cycle_mode() -> AutonomousPaperCycleMode {
    AutonomousPaperCycleMode::SingleShot
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnerConfirmationPolicy {
    Never,
    OnlyForRiskWarnings,
    OnlyForHighConfidenceEvents,
    Always,
}

fn default_owner_confirmation_policy() -> OwnerConfirmationPolicy {
    OwnerConfirmationPolicy::Never
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OwnerAttentionPriority {
    Low,
    Normal,
    High,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnerAttentionType {
    RiskVeto,
    NeedMoreEvidence,
    HighDisagreement,
    HighConfidenceEntry,
    RepeatedBadCall,
    WatchlistCandidate,
    OwnerFeedbackAvailable,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OwnerAttentionItem {
    pub item_id: String,
    pub symbol: Option<String>,
    pub market_scope: Option<MarketScope>,
    pub attention_type: OwnerAttentionType,
    pub priority: OwnerAttentionPriority,
    pub reason: String,
    pub related_member_ids: Vec<String>,
    pub related_decision_id: Option<String>,
    pub requires_owner_input: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OwnerAttentionQueue {
    pub queue_id: String,
    pub items: Vec<OwnerAttentionItem>,
    pub high_priority_count: usize,
    pub requires_owner_input_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnerAttentionInboxStatus {
    Open,
    Acknowledged,
    Deferred,
    Dismissed,
    ConvertedToWatchlist,
    ConvertedToFeedback,
    ReconsiderationRequested,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OwnerAttentionInboxItem {
    pub item_id: String,
    pub symbol: Option<String>,
    pub market_scope: Option<MarketScope>,
    pub attention_type: OwnerAttentionType,
    pub priority: OwnerAttentionPriority,
    pub status: OwnerAttentionInboxStatus,
    pub reason: String,
    pub related_member_ids: Vec<String>,
    pub related_decision_id: Option<String>,
    #[serde(default)]
    pub requires_owner_input: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub paper_only: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OwnerAttentionInbox {
    pub inbox_id: String,
    pub items: Vec<OwnerAttentionInboxItem>,
    pub open_count: usize,
    pub high_priority_count: usize,
    pub requires_owner_input_count: usize,
    pub last_updated_at: Option<String>,
    pub paper_only: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnerAttentionActionType {
    Acknowledge,
    Defer,
    Dismiss,
    ConvertToWatchlist,
    RequestMoreEvidence,
    RequestReconsideration,
    AddComment,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OwnerAttentionAction {
    pub action_id: String,
    pub item_id: String,
    pub action_type: OwnerAttentionActionType,
    pub comment: Option<String>,
    pub created_at: Option<String>,
    pub paper_only: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnerAttentionActionSafetyStatus {
    Passed,
    Rejected,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OwnerAttentionActionResult {
    pub action_id: String,
    pub item_id: String,
    pub previous_status: OwnerAttentionInboxStatus,
    pub new_status: OwnerAttentionInboxStatus,
    pub generated_owner_feedback: Option<OwnerFeedback>,
    pub generated_watchlist_candidate: Option<WatchlistCandidate>,
    pub safety_status: OwnerAttentionActionSafetyStatus,
    pub rejection_reason: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WatchlistCandidateStatus {
    Watching,
    NeedsEvidence,
    RiskBlocked,
    PaperCandidate,
    Archived,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WatchlistCandidate {
    pub candidate_id: String,
    pub symbol: String,
    pub market_scope: MarketScope,
    pub source_attention_item_id: String,
    pub reason: String,
    pub status: WatchlistCandidateStatus,
    pub created_at: Option<String>,
    pub paper_only: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WatchlistCandidateStore {
    pub store_id: String,
    pub candidates: Vec<WatchlistCandidate>,
    pub active_count: usize,
    pub risk_blocked_count: usize,
    pub needs_evidence_count: usize,
    pub paper_only: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum WatchlistRecheckSkipReason {
    Archived,
    RiskBlockedExcluded,
    MissingMarketData,
    MissingNews,
    OverCandidateLimit,
    InvalidCandidate,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SkippedWatchlistCandidate {
    pub candidate: WatchlistCandidate,
    pub reason: WatchlistRecheckSkipReason,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WatchlistRecheckSelection {
    pub selected_candidates: Vec<WatchlistCandidate>,
    pub skipped_candidates: Vec<SkippedWatchlistCandidate>,
    pub skip_reasons: Vec<WatchlistRecheckSkipReason>,
    pub selected_count: usize,
    pub skipped_count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WatchlistCandidateLifecycleEvent {
    pub event_id: String,
    pub candidate_id: String,
    pub symbol: String,
    pub market_scope: MarketScope,
    pub previous_status: WatchlistCandidateStatus,
    pub new_status: WatchlistCandidateStatus,
    pub reason: String,
    pub related_decision_id: Option<String>,
    pub related_attention_item_id: Option<String>,
    pub paper_only: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OwnerDailyBrief {
    pub brief_id: String,
    pub reviewed_symbols: Vec<String>,
    pub watchlist_updates: Vec<String>,
    pub risk_vetoes: Vec<String>,
    pub need_more_evidence_items: Vec<String>,
    pub paper_candidates: Vec<String>,
    pub archived_candidates: Vec<String>,
    pub top_member_voice_changes: Vec<String>,
    pub key_ai_opinions: Vec<String>,
    pub next_owner_attention: Vec<String>,
    pub brief_text: String,
    pub paper_only_warning: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WatchlistRecheckResult {
    pub recheck_id: String,
    pub selection: WatchlistRecheckSelection,
    pub selected_candidates: Vec<WatchlistCandidate>,
    pub batch_result: BatchCommitteeCycleResult,
    pub owner_summary: OwnerCommitteeSummary,
    pub owner_console_view: Option<OwnerCommitteeConsoleView>,
    pub generated_attention_items: Vec<OwnerAttentionItem>,
    pub lifecycle_events: Vec<WatchlistCandidateLifecycleEvent>,
    pub updated_watchlist_store: WatchlistCandidateStore,
    pub owner_daily_brief: Option<OwnerDailyBrief>,
    pub safety_summary: MinimalCommitteeSafetySummary,
    pub paper_only_warning: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WatchlistRecheckConfig {
    pub recheck_id: String,
    pub watchlist_input_path: Option<String>,
    pub watchlist_output_path: Option<String>,
    pub member_state_input_path: Option<String>,
    pub member_state_output_path: Option<String>,
    pub market_data_path: Option<String>,
    pub news_path: Option<String>,
    pub offline_member_output_batch_path: Option<String>,
    pub max_candidates_per_cycle: usize,
    pub include_risk_blocked: bool,
    pub include_needs_evidence: bool,
    pub emit_owner_daily_brief: bool,
    pub paper_only: bool,
    pub watchlist_store: WatchlistCandidateStore,
    pub batch_input: BatchCommitteeCycleInput,
    pub member_state_store: Option<MemberStateStore>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OwnerAttentionTriageInput {
    pub previous_run: AutonomousPaperRunResult,
    pub previous_inbox: Option<OwnerAttentionInbox>,
    pub owner_actions: Vec<OwnerAttentionAction>,
    pub watchlist_store: Option<WatchlistCandidateStore>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OwnerAttentionTriageResult {
    pub inbox: OwnerAttentionInbox,
    pub action_results: Vec<OwnerAttentionActionResult>,
    pub generated_owner_feedback_count: usize,
    pub generated_owner_feedback: Vec<OwnerFeedback>,
    pub generated_watchlist_candidate_count: usize,
    pub generated_watchlist_candidates: Vec<WatchlistCandidate>,
    pub watchlist_store: WatchlistCandidateStore,
    pub safety_summary: MinimalCommitteeSafetySummary,
    pub paper_only_warning: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaperDecisionArchiveEntry {
    pub archive_id: String,
    pub cycle_id: String,
    pub symbol: String,
    pub market_scope: MarketScope,
    pub chairman_action: ChairmanFinalAction,
    pub risk_governor_status: RiskGovernorStatus,
    pub event_count: usize,
    pub deciding_members: Vec<String>,
    pub dissenting_members: Vec<String>,
    pub paper_only: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaperDecisionArchive {
    pub run_id: String,
    pub entries: Vec<PaperDecisionArchiveEntry>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AutonomousPaperCycle {
    pub cycle_id: String,
    pub cycle_index: usize,
    pub market_scopes: Vec<MarketScope>,
    pub symbols: Vec<String>,
    pub batch_result: BatchCommitteeCycleResult,
    pub state_update: BatchCycleStateUpdate,
    pub owner_summary: OwnerCommitteeSummary,
    pub owner_console_view: Option<OwnerCommitteeConsoleView>,
    pub owner_feedback_reconsideration: Option<OwnerFeedbackReconsiderationResult>,
    pub attention_items: Vec<OwnerAttentionItem>,
    pub completed_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AutonomousPaperRunResult {
    pub run_id: String,
    pub cycle_count: usize,
    pub cycles: Vec<AutonomousPaperCycle>,
    pub final_member_states: Vec<MemberStateSnapshot>,
    pub attention_queue: OwnerAttentionQueue,
    pub owner_attention_triage: Option<OwnerAttentionTriageResult>,
    pub watchlist_recheck: Option<WatchlistRecheckResult>,
    pub paper_decision_archive: PaperDecisionArchive,
    pub safety_summary: MinimalCommitteeSafetySummary,
    pub paper_only_warning: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AutonomousPaperRunConfig {
    pub run_id: String,
    pub market_scopes: Vec<MarketScope>,
    pub symbols: Vec<String>,
    pub max_cycles: usize,
    pub cycle_mode: AutonomousPaperCycleMode,
    pub require_owner_confirmation: OwnerConfirmationPolicy,
    pub local_market_data_path: Option<String>,
    pub local_news_path: Option<String>,
    pub offline_member_output_batch_path: Option<String>,
    pub member_state_input_path: Option<String>,
    pub member_state_output_path: Option<String>,
    pub owner_feedback_path: Option<String>,
    pub owner_attention_inbox_input_path: Option<String>,
    pub owner_attention_inbox_output_path: Option<String>,
    pub owner_attention_actions_path: Option<String>,
    pub watchlist_candidate_input_path: Option<String>,
    pub watchlist_candidate_output_path: Option<String>,
    pub emit_owner_attention_inbox: bool,
    pub enable_watchlist_recheck: bool,
    pub watchlist_input_path: Option<String>,
    pub watchlist_output_path: Option<String>,
    pub max_candidates_per_cycle: usize,
    pub include_risk_blocked: bool,
    pub include_needs_evidence: bool,
    pub emit_owner_daily_brief: bool,
    pub emit_owner_console_view: bool,
    pub paper_only: bool,
    pub batch_input: BatchCommitteeCycleInput,
    pub member_state_store: Option<MemberStateStore>,
    pub owner_feedback: Vec<OwnerFeedback>,
    pub previous_owner_attention_inbox: Option<OwnerAttentionInbox>,
    pub owner_attention_actions: Vec<OwnerAttentionAction>,
    pub watchlist_candidate_store: Option<WatchlistCandidateStore>,
    pub watchlist_recheck_store: Option<WatchlistCandidateStore>,
}

fn default_autonomous_max_cycles() -> usize {
    1
}

fn default_paper_only_true() -> bool {
    true
}

fn default_max_candidates_per_cycle() -> usize {
    3
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BatchCommitteeCycleWithStateInput {
    pub batch_input: BatchCommitteeCycleInput,
    #[serde(default)]
    pub member_state_store: Option<MemberStateStore>,
    #[serde(default)]
    pub member_state_output_path: Option<String>,
    #[serde(default)]
    pub emit_owner_summary: bool,
    #[serde(default)]
    pub emit_owner_console_view: bool,
    #[serde(default)]
    pub owner_feedback: Vec<OwnerFeedback>,
    #[serde(default)]
    pub emit_reconsideration_view: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BatchCommitteeCycleWithStateResult {
    pub batch_result: BatchCommitteeCycleResult,
    pub state_update: BatchCycleStateUpdate,
    pub owner_summary: Option<OwnerCommitteeSummary>,
    pub owner_console_view: Option<OwnerCommitteeConsoleView>,
    pub owner_feedback_reconsideration: Option<OwnerFeedbackReconsiderationResult>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MinimalAiCommitteeCycleConfig {
    #[serde(default = "default_input_path")]
    pub input_path: Option<String>,
    #[serde(default)]
    pub offline_member_opinion_path: Option<String>,
    #[serde(default)]
    pub offline_member_output_batch_path: Option<String>,
    #[serde(default)]
    pub batch_mode: bool,
    #[serde(default)]
    pub member_state_input_path: Option<String>,
    #[serde(default)]
    pub member_state_output_path: Option<String>,
    #[serde(default)]
    pub emit_owner_summary: bool,
    #[serde(default)]
    pub emit_owner_console_view: bool,
    #[serde(default)]
    pub owner_feedback_path: Option<String>,
    #[serde(default)]
    pub emit_reconsideration_view: bool,
    #[serde(default)]
    pub autonomous_paper_run: bool,
    #[serde(default)]
    pub run_id: Option<String>,
    #[serde(default)]
    pub market_scopes: Vec<MarketScope>,
    #[serde(default)]
    pub symbols: Vec<String>,
    #[serde(default = "default_autonomous_max_cycles")]
    pub max_cycles: usize,
    #[serde(default = "default_autonomous_cycle_mode")]
    pub cycle_mode: AutonomousPaperCycleMode,
    #[serde(default = "default_owner_confirmation_policy")]
    pub require_owner_confirmation: OwnerConfirmationPolicy,
    #[serde(default)]
    pub local_market_data_path: Option<String>,
    #[serde(default)]
    pub local_news_path: Option<String>,
    #[serde(default = "default_paper_only_true")]
    pub paper_only: bool,
    #[serde(default)]
    pub owner_attention_inbox_input_path: Option<String>,
    #[serde(default)]
    pub owner_attention_inbox_output_path: Option<String>,
    #[serde(default)]
    pub owner_attention_actions_path: Option<String>,
    #[serde(default)]
    pub watchlist_candidate_input_path: Option<String>,
    #[serde(default)]
    pub watchlist_candidate_output_path: Option<String>,
    #[serde(default)]
    pub emit_owner_attention_inbox: bool,
    #[serde(default)]
    pub enable_watchlist_recheck: bool,
    #[serde(default)]
    pub watchlist_input_path: Option<String>,
    #[serde(default)]
    pub watchlist_output_path: Option<String>,
    #[serde(default = "default_max_candidates_per_cycle")]
    pub max_candidates_per_cycle: usize,
    #[serde(default)]
    pub include_risk_blocked: bool,
    #[serde(default = "default_paper_only_true")]
    pub include_needs_evidence: bool,
    #[serde(default)]
    pub emit_owner_daily_brief: bool,
    #[serde(default)]
    pub inline_offline_member_opinions: Vec<OfflineMemberOpinionFixture>,
    #[serde(default)]
    pub inline_input: Option<MinimalCommitteeCycleInput>,
    #[serde(default)]
    pub pilot_roster: Option<String>,
    #[serde(default)]
    pub paper_outcome: Option<SimulatedPaperOutcome>,
    #[serde(default)]
    pub archetype_style_cards_path: Option<String>,
    #[serde(default = "default_style_mapping_mode")]
    pub style_mapping_mode: StyleMappingMode,
}

impl MinimalAiCommitteeCycleConfig {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        if !local_only(&path.to_string_lossy()) {
            return Err("minimal AI committee config path must be local".to_string());
        }
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        let config: Self = toml::from_str(&text).map_err(|err| err.to_string())?;
        if config.autonomous_paper_run {
            reject_unsafe_autonomous_config_text(&text)?;
        }
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), String> {
        if let Some(path) = &self.input_path {
            if !local_only(path) {
                return Err("minimal AI committee input_path must be local".to_string());
            }
        }
        if let Some(path) = &self.offline_member_opinion_path {
            if !local_only(path) {
                return Err(
                    "minimal AI committee offline_member_opinion_path must be local".to_string(),
                );
            }
        }
        if let Some(path) = &self.offline_member_output_batch_path {
            if !local_only(path) {
                return Err(
                    "minimal AI committee offline_member_output_batch_path must be local"
                        .to_string(),
                );
            }
        }
        if let Some(path) = &self.member_state_input_path {
            if !local_only(path) {
                return Err(
                    "minimal AI committee member_state_input_path must be local".to_string()
                );
            }
        }
        if let Some(path) = &self.member_state_output_path {
            if !local_only(path) {
                return Err(
                    "minimal AI committee member_state_output_path must be local".to_string(),
                );
            }
        }
        if let Some(path) = &self.owner_feedback_path {
            if !local_only(path) {
                return Err("minimal AI committee owner_feedback_path must be local".to_string());
            }
        }
        if let Some(path) = &self.local_market_data_path {
            if !local_only(path) {
                return Err("minimal AI committee local_market_data_path must be local".to_string());
            }
        }
        if let Some(path) = &self.local_news_path {
            if !local_only(path) {
                return Err("minimal AI committee local_news_path must be local".to_string());
            }
        }
        if let Some(path) = &self.owner_attention_inbox_input_path {
            if !local_only(path) {
                return Err(
                    "minimal AI committee owner_attention_inbox_input_path must be local"
                        .to_string(),
                );
            }
        }
        if let Some(path) = &self.owner_attention_inbox_output_path {
            if !local_only(path) {
                return Err(
                    "minimal AI committee owner_attention_inbox_output_path must be local"
                        .to_string(),
                );
            }
        }
        if let Some(path) = &self.owner_attention_actions_path {
            if !local_only(path) {
                return Err(
                    "minimal AI committee owner_attention_actions_path must be local".to_string(),
                );
            }
        }
        if let Some(path) = &self.watchlist_candidate_input_path {
            if !local_only(path) {
                return Err(
                    "minimal AI committee watchlist_candidate_input_path must be local".to_string(),
                );
            }
        }
        if let Some(path) = &self.watchlist_candidate_output_path {
            if !local_only(path) {
                return Err(
                    "minimal AI committee watchlist_candidate_output_path must be local"
                        .to_string(),
                );
            }
        }
        if let Some(path) = &self.watchlist_input_path {
            if !local_only(path) {
                return Err("minimal AI committee watchlist_input_path must be local".to_string());
            }
        }
        if let Some(path) = &self.watchlist_output_path {
            if !local_only(path) {
                return Err("minimal AI committee watchlist_output_path must be local".to_string());
            }
        }
        if self.max_candidates_per_cycle == 0 {
            return Err(
                "watchlist recheck max_candidates_per_cycle must be at least 1".to_string(),
            );
        }
        if let Some(path) = &self.archetype_style_cards_path {
            if !local_only(path) {
                return Err(
                    "minimal AI committee archetype_style_cards_path must be local".to_string(),
                );
            }
        }
        if self.input_path.is_none() && self.inline_input.is_none() {
            return Err(
                "minimal AI committee config requires input_path or inline_input".to_string(),
            );
        }
        if !self.paper_only {
            return Err("autonomous paper run config must be paper-only".to_string());
        }
        if self.max_cycles == 0 {
            return Err("autonomous paper run max_cycles must be at least 1".to_string());
        }
        if let Some(pilot_roster) = &self.pilot_roster {
            if pilot_roster != "three_member" {
                return Err(
                    "minimal AI committee pilot_roster supports only three_member".to_string(),
                );
            }
        }
        Ok(())
    }

    pub fn load_input(&self) -> Result<MinimalCommitteeCycleInput, String> {
        self.validate()?;
        let mut input = if let Some(input) = &self.inline_input {
            input.clone()
        } else {
            let path = self
                .input_path
                .as_deref()
                .ok_or_else(|| "minimal AI committee input_path missing".to_string())?;
            let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
            serde_json::from_str(&text).map_err(|err| err.to_string())?
        };

        input
            .offline_member_opinions
            .extend(self.inline_offline_member_opinions.clone());
        if let Some(path) = &self.offline_member_opinion_path {
            let adapter = OfflineMemberBrainAdapter::from_json_path(Path::new(path))?;
            input.offline_member_opinions.extend(adapter.fixtures);
        }
        input.style_mapping_mode = self.style_mapping_mode;
        if let Some(path) = &self.archetype_style_cards_path {
            let registry = RealArchetypeIntakePolicy::default()
                .load_registry_from_local_json(Path::new(path))?;
            input.archetype_style_cards.extend(registry.cards);
        }
        if self.pilot_roster.as_deref() == Some("three_member") {
            let runtime_status = if input.offline_member_opinions.is_empty() {
                CoreRuntimeStatus::MockLocal
            } else {
                CoreRuntimeStatus::OfflineFixture
            };
            input.members = create_three_member_pilot_roster_with_runtime(
                input.market_data.market_scope,
                runtime_status,
            );
            input
                .paper_outcome_feedback
                .get_or_insert_with(|| PaperOutcomeFeedback {
                    symbol: input.market_data.symbol.clone(),
                    market_scope: input.market_data.market_scope,
                    decision_id: "pending-three-member-pilot".to_string(),
                    simulated_result: self
                        .paper_outcome
                        .unwrap_or_else(|| outcome_from_return(input.simulated_outcome_return)),
                    notes: vec![
                        "offline paper outcome feedback only".to_string(),
                        "learning journal does not train model weights".to_string(),
                    ],
                });
        }
        Ok(input)
    }

    pub fn load_batch_input(&self) -> Result<BatchCommitteeCycleInput, String> {
        self.validate()?;
        let path = self
            .input_path
            .as_deref()
            .ok_or_else(|| "minimal AI committee batch input_path missing".to_string())?;
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        let mut input: BatchCommitteeCycleInput =
            serde_json::from_str(&text).map_err(|err| err.to_string())?;
        if input.market_data.is_empty() {
            return Err("minimal AI committee batch requires market_data".to_string());
        }
        if self.pilot_roster.as_deref() == Some("three_member") {
            let runtime_status = if self.offline_member_output_batch_path.is_some()
                || !self.inline_offline_member_opinions.is_empty()
                || self.offline_member_opinion_path.is_some()
            {
                CoreRuntimeStatus::OfflineFixture
            } else {
                CoreRuntimeStatus::MockLocal
            };
            input.members = create_three_member_pilot_roster_for_scopes(
                input
                    .market_data
                    .iter()
                    .map(|market| market.market_scope)
                    .collect(),
                runtime_status,
            );
        }

        let mut batch = if let Some(path) = &self.offline_member_output_batch_path {
            let load = OfflineMemberOutputBatch::from_json_path(Path::new(path))?;
            Some(OfflineMemberOutputBatch {
                batch_id: load.batch_id,
                created_at: "loaded-from-local-file".to_string(),
                source_label: path.clone(),
                opinions: load.opinions,
            })
        } else {
            input.offline_output_batch.take()
        };

        if !self.inline_offline_member_opinions.is_empty()
            || self.offline_member_opinion_path.is_some()
        {
            let mut opinions = batch
                .as_ref()
                .map(|batch| batch.opinions.clone())
                .unwrap_or_default();
            opinions.extend(self.inline_offline_member_opinions.clone());
            if let Some(path) = &self.offline_member_opinion_path {
                opinions
                    .extend(OfflineMemberBrainAdapter::from_json_path(Path::new(path))?.fixtures);
            }
            batch = Some(OfflineMemberOutputBatch {
                batch_id: batch
                    .as_ref()
                    .map(|batch| batch.batch_id.clone())
                    .unwrap_or_else(|| "inline-offline-member-output-batch".to_string()),
                created_at: batch
                    .as_ref()
                    .map(|batch| batch.created_at.clone())
                    .unwrap_or_else(|| "inline".to_string()),
                source_label: "minimal-ai-committee-cycle-config".to_string(),
                opinions,
            });
        }
        input.offline_output_batch = batch;
        input.paper_outcome = input.paper_outcome.or(self.paper_outcome);
        if let Some(path) = &self.local_market_data_path {
            input.market_data = load_market_data_from_local_json(Path::new(path))?;
        }
        if let Some(path) = &self.local_news_path {
            input.news = load_news_from_local_json(Path::new(path))?;
        }
        filter_batch_input_for_autonomous_scope(&mut input, &self.symbols, &self.market_scopes);
        if input.market_data.is_empty() {
            return Err("minimal AI committee batch requires filtered market_data".to_string());
        }
        Ok(input)
    }

    pub fn load_batch_state_input(&self) -> Result<BatchCommitteeCycleWithStateInput, String> {
        let batch_input = self.load_batch_input()?;
        let member_state_store = if let Some(path) = &self.member_state_input_path {
            Some(MemberStateStore::load_from_local_json(Path::new(path))?)
        } else {
            None
        };
        let owner_feedback = if let Some(path) = &self.owner_feedback_path {
            load_owner_feedback_from_local_json(Path::new(path))?
        } else {
            Vec::new()
        };
        Ok(BatchCommitteeCycleWithStateInput {
            batch_input,
            member_state_store,
            member_state_output_path: self.member_state_output_path.clone(),
            emit_owner_summary: self.emit_owner_summary,
            emit_owner_console_view: self.emit_owner_console_view,
            owner_feedback,
            emit_reconsideration_view: self.emit_reconsideration_view,
        })
    }

    pub fn load_autonomous_paper_run_config(&self) -> Result<AutonomousPaperRunConfig, String> {
        self.validate()?;
        let batch_input = self.load_batch_input()?;
        let member_state_store = if let Some(path) = &self.member_state_input_path {
            Some(MemberStateStore::load_from_local_json(Path::new(path))?)
        } else {
            None
        };
        let owner_feedback = if let Some(path) = &self.owner_feedback_path {
            load_owner_feedback_from_local_json(Path::new(path))?
        } else {
            Vec::new()
        };
        let previous_owner_attention_inbox =
            if let Some(path) = &self.owner_attention_inbox_input_path {
                Some(OwnerAttentionInbox::load_from_local_json(Path::new(path))?)
            } else {
                None
            };
        let owner_attention_actions = if let Some(path) = &self.owner_attention_actions_path {
            load_owner_attention_actions_from_local_json(Path::new(path))?
        } else {
            Vec::new()
        };
        let watchlist_candidate_store = if let Some(path) = &self.watchlist_candidate_input_path {
            Some(WatchlistCandidateStore::load_from_local_json(Path::new(
                path,
            ))?)
        } else {
            None
        };
        let watchlist_recheck_store = if let Some(path) = self
            .watchlist_input_path
            .as_ref()
            .or(self.watchlist_candidate_input_path.as_ref())
        {
            Some(WatchlistCandidateStore::load_from_local_json(Path::new(
                path,
            ))?)
        } else {
            watchlist_candidate_store.clone()
        };
        Ok(AutonomousPaperRunConfig {
            run_id: self
                .run_id
                .clone()
                .unwrap_or_else(|| "autonomous-paper-run".to_string()),
            market_scopes: self.market_scopes.clone(),
            symbols: self.symbols.clone(),
            max_cycles: self.max_cycles,
            cycle_mode: self.cycle_mode,
            require_owner_confirmation: self.require_owner_confirmation,
            local_market_data_path: self.local_market_data_path.clone(),
            local_news_path: self.local_news_path.clone(),
            offline_member_output_batch_path: self.offline_member_output_batch_path.clone(),
            member_state_input_path: self.member_state_input_path.clone(),
            member_state_output_path: self.member_state_output_path.clone(),
            owner_feedback_path: self.owner_feedback_path.clone(),
            owner_attention_inbox_input_path: self.owner_attention_inbox_input_path.clone(),
            owner_attention_inbox_output_path: self.owner_attention_inbox_output_path.clone(),
            owner_attention_actions_path: self.owner_attention_actions_path.clone(),
            watchlist_candidate_input_path: self.watchlist_candidate_input_path.clone(),
            watchlist_candidate_output_path: self.watchlist_candidate_output_path.clone(),
            emit_owner_attention_inbox: self.emit_owner_attention_inbox,
            enable_watchlist_recheck: self.enable_watchlist_recheck,
            watchlist_input_path: self.watchlist_input_path.clone(),
            watchlist_output_path: self.watchlist_output_path.clone(),
            max_candidates_per_cycle: self.max_candidates_per_cycle,
            include_risk_blocked: self.include_risk_blocked,
            include_needs_evidence: self.include_needs_evidence,
            emit_owner_daily_brief: self.emit_owner_daily_brief,
            emit_owner_console_view: self.emit_owner_console_view,
            paper_only: self.paper_only,
            batch_input,
            member_state_store,
            owner_feedback,
            previous_owner_attention_inbox,
            owner_attention_actions,
            watchlist_candidate_store,
            watchlist_recheck_store,
        })
    }
}

fn reject_unsafe_autonomous_config_text(text: &str) -> Result<(), String> {
    let value: toml::Value = toml::from_str(text).map_err(|err| err.to_string())?;
    reject_unsafe_autonomous_config_value(&value)
}

fn reject_unsafe_autonomous_config_value(value: &toml::Value) -> Result<(), String> {
    match value {
        toml::Value::Table(table) => {
            for (key, value) in table {
                reject_unsafe_autonomous_config_key_or_text(key)?;
                reject_unsafe_autonomous_config_value(value)?;
            }
        }
        toml::Value::Array(items) => {
            for item in items {
                reject_unsafe_autonomous_config_value(item)?;
            }
        }
        toml::Value::String(text) => reject_unsafe_autonomous_config_key_or_text(text)?,
        _ => {}
    }
    Ok(())
}

fn reject_unsafe_autonomous_config_key_or_text(text: &str) -> Result<(), String> {
    let lower = text.to_ascii_lowercase();
    for prohibited in [
        "broker",
        "order",
        "account",
        "live_trading",
        "execute",
        "buying_power",
        "holdings",
        "max leverage",
        "maximum leverage",
        "레버리지 최대로",
    ] {
        if lower.contains(prohibited) {
            return Err(format!(
                "autonomous paper run config rejected unsafe field or instruction: {prohibited}"
            ));
        }
    }
    Ok(())
}

fn load_market_data_from_local_json(path: &Path) -> Result<Vec<MarketDataSnapshot>, String> {
    if !local_only(&path.to_string_lossy()) {
        return Err("local market data path must be local".to_string());
    }
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    reject_unsafe_offline_batch_text(&text)?;
    serde_json::from_str(&text).map_err(|err| err.to_string())
}

fn load_news_from_local_json(path: &Path) -> Result<Vec<NewsSnapshot>, String> {
    if !local_only(&path.to_string_lossy()) {
        return Err("local news path must be local".to_string());
    }
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    reject_unsafe_offline_batch_text(&text)?;
    serde_json::from_str(&text).map_err(|err| err.to_string())
}

fn filter_batch_input_for_autonomous_scope(
    input: &mut BatchCommitteeCycleInput,
    symbols: &[String],
    market_scopes: &[MarketScope],
) {
    if !symbols.is_empty() {
        input
            .market_data
            .retain(|market| symbols.contains(&market.symbol));
        input.news.retain(|news| symbols.contains(&news.symbol));
        if let Some(batch) = &mut input.offline_output_batch {
            batch
                .opinions
                .retain(|opinion| symbols.contains(&opinion.symbol));
        }
    }
    if !market_scopes.is_empty() {
        input
            .market_data
            .retain(|market| market_scopes.contains(&market.market_scope));
        if let Some(batch) = &mut input.offline_output_batch {
            batch
                .opinions
                .retain(|opinion| market_scopes.contains(&opinion.market_scope));
        }
        input.members.retain(|member| {
            member
                .market_scopes
                .iter()
                .any(|scope| market_scopes.contains(scope))
        });
    }
}

pub fn load_owner_feedback_from_local_json(path: &Path) -> Result<Vec<OwnerFeedback>, String> {
    if !local_only(&path.to_string_lossy()) {
        return Err("owner feedback path must be local".to_string());
    }
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    reject_unsafe_owner_feedback_text(&text)?;
    let feedback: Vec<OwnerFeedback> =
        serde_json::from_str(&text).map_err(|err| err.to_string())?;
    for item in &feedback {
        validate_owner_feedback(item)?;
    }
    Ok(feedback)
}

fn validate_owner_feedback(feedback: &OwnerFeedback) -> Result<(), String> {
    if !feedback.paper_only {
        return Err("owner feedback must be paper-only".to_string());
    }
    if feedback.feedback_id.trim().is_empty() {
        return Err("owner feedback requires feedback_id".to_string());
    }
    reject_unsafe_owner_feedback_text(&feedback.text)
}

fn reject_unsafe_owner_feedback_text(text: &str) -> Result<(), String> {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(text) {
        reject_unsafe_offline_batch_value(&value)?;
        reject_unsafe_owner_feedback_value(&value)
    } else {
        reject_unsafe_owner_feedback_string(text)
    }
}

fn reject_unsafe_owner_feedback_value(value: &serde_json::Value) -> Result<(), String> {
    match value {
        serde_json::Value::Object(map) => {
            for (key, value) in map {
                reject_unsafe_owner_feedback_string(key)?;
                reject_unsafe_owner_feedback_value(value)?;
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                reject_unsafe_owner_feedback_value(item)?;
            }
        }
        serde_json::Value::String(text) => reject_unsafe_owner_feedback_string(text)?,
        _ => {}
    }
    Ok(())
}

fn reject_unsafe_owner_feedback_string(text: &str) -> Result<(), String> {
    let lower = text.to_ascii_lowercase();
    for prohibited in [
        "buy now",
        "buy now with real money",
        "place order",
        "execute order",
        "submit order",
        "execute trade",
        "place trade",
        "live execution",
        "live trading",
        "real money",
        "broker account",
        "trading account",
        "use my account",
        "from account",
        "guaranteed return",
        "guaranteed returns",
        "guaranteed profit",
        "guaranteed profits",
        "private data",
        "illegal data",
        "inside information",
        "nonpublic",
        "max leverage",
        "maximum leverage",
        "계좌",
        "주문",
        "실거래",
        "레버리지 최대로",
    ] {
        if lower.contains(prohibited) {
            return Err(format!(
                "owner feedback rejected unsafe instruction: {prohibited}"
            ));
        }
    }
    Ok(())
}

impl MemberStateStore {
    pub fn from_members(store_id: &str, members: &[AICommitteeMember], source_label: &str) -> Self {
        Self {
            store_id: store_id.to_string(),
            members: members
                .iter()
                .map(|member| MemberStateSnapshot {
                    member_id: member.member_id.clone(),
                    score: member.score,
                    voice_weight: member.voice_weight,
                    status: member.status,
                    memory_state: member
                        .memory_state
                        .clone()
                        .unwrap_or_else(|| MemberMemoryState::new(&member.member_id)),
                    learning_journal_summary: empty_learning_journal_summary(),
                    last_updated_at: None,
                })
                .collect(),
            source_label: source_label.to_string(),
            paper_only: true,
        }
    }

    pub fn load_from_local_json(path: &Path) -> Result<Self, String> {
        if !local_only(&path.to_string_lossy()) {
            return Err("member state store path must be local".to_string());
        }
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        reject_unsafe_offline_batch_text(&text)?;
        let store: MemberStateStore = serde_json::from_str(&text).map_err(|err| err.to_string())?;
        store.validate()?;
        Ok(store)
    }

    pub fn save_to_local_json(&self, path: &Path) -> Result<(), String> {
        if !local_only(&path.to_string_lossy()) {
            return Err("member state store path must be local".to_string());
        }
        self.validate()?;
        let text = serde_json::to_string_pretty(self).map_err(|err| err.to_string())?;
        fs::write(path, text).map_err(|err| err.to_string())
    }

    pub fn apply_score_updates(&mut self, score_updates: &[MemberScoreUpdate]) {
        for update in score_updates {
            if let Some(state) = self
                .members
                .iter_mut()
                .find(|state| state.member_id == update.member_id)
            {
                let previous_score = state.score;
                let score_delta = update.new_score - update.previous_score;
                let voice_delta = update.new_voice_weight - update.previous_voice_weight;
                state.score = clamp_unit(state.score + score_delta);
                state.voice_weight = clamp_unit(state.voice_weight + voice_delta);
                if previous_score >= 0.3 && state.score < 0.3 {
                    state.status = AICommitteeMemberStatus::Demoted;
                } else if previous_score < 0.8 && state.score >= 0.8 {
                    state.status = AICommitteeMemberStatus::Active;
                }
                state.last_updated_at = Some("offline-batch-cycle".to_string());
            }
        }
    }

    pub fn apply_memory_updates(&mut self, memory_updates: &[MemberMemoryState]) {
        for update in memory_updates {
            if let Some(state) = self
                .members
                .iter_mut()
                .find(|state| state.member_id == update.member_id)
            {
                merge_memory_state(&mut state.memory_state, update);
                state.last_updated_at = Some("offline-batch-cycle".to_string());
            }
        }
    }

    pub fn append_learning_journal_entries(&mut self, entries: &[MemberLearningJournalEntry]) {
        for entry in entries {
            if let Some(state) = self
                .members
                .iter_mut()
                .find(|state| state.member_id == entry.member_id)
            {
                match entry.learning_signal {
                    MemberLearningSignal::Reinforce => {
                        state.learning_journal_summary.reinforce_count += 1;
                    }
                    MemberLearningSignal::Penalize => {
                        state.learning_journal_summary.penalize_count += 1;
                    }
                    MemberLearningSignal::Watch => {
                        state.learning_journal_summary.watch_count += 1;
                    }
                    MemberLearningSignal::Ignore => {
                        state.learning_journal_summary.ignore_count += 1;
                    }
                }
                state.last_updated_at = Some("offline-batch-cycle".to_string());
            }
        }
    }

    pub fn get_member_state(&self, member_id: &str) -> Option<&MemberStateSnapshot> {
        self.members
            .iter()
            .find(|state| state.member_id == member_id)
    }

    fn validate(&self) -> Result<(), String> {
        if !self.paper_only {
            return Err("member state store must be paper-only".to_string());
        }
        if self.store_id.trim().is_empty() {
            return Err("member state store requires store_id".to_string());
        }
        Ok(())
    }
}

fn empty_learning_journal_summary() -> MemberLearningJournalSummary {
    MemberLearningJournalSummary {
        reinforce_count: 0,
        penalize_count: 0,
        watch_count: 0,
        ignore_count: 0,
    }
}

fn merge_memory_state(current: &mut MemberMemoryState, update: &MemberMemoryState) {
    for symbol in &update.recent_symbols {
        if !current.recent_symbols.contains(symbol) {
            current.recent_symbols.push(symbol.clone());
        }
    }
    current.recent_opinion_count += update.recent_opinion_count;
    current.recent_event_count += update.recent_event_count;
    current.recent_good_call_count += update.recent_good_call_count;
    current.recent_bad_call_count += update.recent_bad_call_count;
    current.recent_risk_veto_count += update.recent_risk_veto_count;
    for note in &update.notes {
        if !current.notes.contains(note) {
            current.notes.push(note.clone());
        }
    }
}

pub fn run_minimal_committee_cycle_from_config_path(
    path: &Path,
) -> Result<MinimalCommitteeCycleResult, String> {
    let config = MinimalAiCommitteeCycleConfig::from_toml_path(path)?;
    run_minimal_committee_cycle(config.load_input()?)
}

pub fn run_batch_committee_cycle_from_config_path(
    path: &Path,
) -> Result<BatchCommitteeCycleResult, String> {
    let config = MinimalAiCommitteeCycleConfig::from_toml_path(path)?;
    if !config.batch_mode {
        return Err("minimal AI committee batch_mode must be true for batch cycle".to_string());
    }
    run_batch_committee_cycle(config.load_batch_input()?)
}

pub fn run_batch_committee_cycle_with_state_from_config_path(
    path: &Path,
) -> Result<BatchCommitteeCycleWithStateResult, String> {
    let config = MinimalAiCommitteeCycleConfig::from_toml_path(path)?;
    if !config.batch_mode {
        return Err(
            "minimal AI committee batch_mode must be true for stateful batch cycle".to_string(),
        );
    }
    run_batch_committee_cycle_with_state(config.load_batch_state_input()?)
}

pub fn run_minimal_committee_cycle(
    input: MinimalCommitteeCycleInput,
) -> Result<MinimalCommitteeCycleResult, String> {
    if input.members.is_empty() {
        return Err("minimal AI committee cycle requires at least one member".to_string());
    }
    if input.market_data.symbol.trim().is_empty() {
        return Err("minimal AI committee cycle requires a symbol".to_string());
    }

    let selected_scope = input.market_data.market_scope;
    let registry = AiMemberCoreRegistry::from_members_with_offline_hint(
        &input.members,
        !input.offline_member_opinions.is_empty(),
    );
    let activation_plan = registry.select_members_for_cycle(
        &input.members,
        selected_scope,
        None,
        &input.activation_policy,
    );
    let selected_members: Vec<AICommitteeMember> = input
        .members
        .iter()
        .filter(|member| {
            activation_plan
                .selected_member_ids
                .contains(&member.member_id)
        })
        .cloned()
        .collect();

    let router_output = route_data_to_ai_members(DataRouterInput {
        market_data: vec![input.market_data.clone()],
        news: input.news.clone(),
        members: selected_members.clone(),
        owner_context: input.owner_context.clone(),
    });

    if router_output.packets.is_empty() {
        return Ok(no_committee_result(
            input.market_data,
            selected_scope,
            activation_plan,
        ));
    }

    let invited_members: Vec<AICommitteeMember> = router_output
        .packets
        .iter()
        .filter_map(|packet| {
            input
                .members
                .iter()
                .find(|member| member.member_id == packet.member_id)
                .cloned()
        })
        .collect();

    let offline_adapter = if input.offline_member_opinions.is_empty() {
        None
    } else {
        Some(OfflineMemberBrainAdapter {
            fixtures: input.offline_member_opinions.clone(),
        })
    };
    let style_card_registry = if input.archetype_style_cards.is_empty()
        || matches!(input.style_mapping_mode, StyleMappingMode::None)
    {
        None
    } else {
        Some(ArchetypeStyleCardRegistry::from_cards(
            input.archetype_style_cards.clone(),
        ))
    };
    let three_member_style_mapping = style_card_registry
        .as_ref()
        .map(map_style_cards_to_three_member_pilot);
    let style_influenced_profiles = style_card_registry
        .as_ref()
        .zip(three_member_style_mapping.as_ref())
        .map(|(registry, mapping)| style_influenced_profiles_for_mapping(registry, mapping))
        .unwrap_or_default();

    let member_opinions: Vec<MemberOpinion> = invited_members
        .iter()
        .zip(router_output.packets.iter())
        .map(|(member, packet)| {
            let core_spec = registry
                .get_core_spec(&member.member_id)
                .cloned()
                .unwrap_or_else(|| {
                    resolved_core_spec_for_member(member, offline_adapter.is_some())
                });
            let opinion = CoreAwareMemberBrainAdapter {
                member: member.clone(),
                core_spec,
                offline_adapter: offline_adapter.clone().unwrap_or_else(|| {
                    OfflineMemberBrainAdapter {
                        fixtures: Vec::new(),
                    }
                }),
            }
            .produce_opinion(packet);
            if let Some(profile) = style_influenced_profiles
                .iter()
                .find(|profile| profile.member_id == member.member_id)
            {
                apply_style_influence_to_opinion(opinion, profile)
            } else {
                opinion
            }
        })
        .collect();

    let event = select_event(&input.market_data, &member_opinions);
    let (committee_session, chairman_decision) = if let Some(event) = event.clone() {
        let session = build_committee_session(
            &input.market_data,
            event,
            &invited_members,
            &member_opinions,
        );
        let decision = synthesize_chairman_decision(&session, input.risk_veto_volatility_threshold);
        (Some(session), Some(decision))
    } else {
        (None, None)
    };

    let paper_outcome_feedback =
        input
            .paper_outcome_feedback
            .clone()
            .unwrap_or_else(|| PaperOutcomeFeedback {
                symbol: input.market_data.symbol.clone(),
                market_scope: input.market_data.market_scope,
                decision_id: chairman_decision
                    .as_ref()
                    .map(|decision| decision.decision_id.clone())
                    .unwrap_or_else(|| "no-committee-decision".to_string()),
                simulated_result: outcome_from_return(input.simulated_outcome_return),
                notes: vec!["default offline paper outcome feedback".to_string()],
            });
    let feedback_result = apply_paper_outcome_feedback(
        &invited_members,
        &member_opinions,
        chairman_decision.as_ref(),
        &paper_outcome_feedback,
    );
    let member_roles = invited_members
        .iter()
        .filter_map(|member| {
            member.role.map(|role| MemberRoleReport {
                member_id: member.member_id.clone(),
                role,
            })
        })
        .collect();
    let learning_journals = build_learning_journals(&feedback_result.learning_journal_entries);

    Ok(MinimalCommitteeCycleResult {
        selected_scope,
        symbol: input.market_data.symbol,
        routed_packet_count: router_output.routed_member_count,
        triggered_event_count: usize::from(event.is_some()),
        committee_session_count: usize::from(committee_session.is_some()),
        distributed_member_ids: router_output
            .packets
            .into_iter()
            .map(|packet| packet.member_id)
            .collect(),
        member_opinions,
        event,
        committee_session,
        chairman_decision,
        score_updates: feedback_result.score_updates,
        activation_plan,
        member_roles,
        memory_states: feedback_result.updated_memory_states,
        learning_journal_entry_count: feedback_result.learning_journal_entries.len(),
        learning_journal_entries: feedback_result.learning_journal_entries,
        learning_journals,
        style_card_registry,
        three_member_style_mapping,
        style_influenced_profiles,
        safety_summary: safety_summary(),
    })
}

pub fn run_batch_committee_cycle(
    input: BatchCommitteeCycleInput,
) -> Result<BatchCommitteeCycleResult, String> {
    if input.members.is_empty() {
        return Err("minimal AI committee batch requires at least one member".to_string());
    }
    if input.market_data.is_empty() {
        return Err("minimal AI committee batch requires market_data".to_string());
    }
    if input
        .market_data
        .iter()
        .any(|market| market.symbol.trim().is_empty())
    {
        return Err("minimal AI committee batch requires symbols".to_string());
    }

    let router_output = route_data_to_ai_members(DataRouterInput {
        market_data: input.market_data.clone(),
        news: input.news.clone(),
        members: input.members.clone(),
        owner_context: input.owner_context.clone(),
    });
    let offline_adapter = OfflineMemberBrainAdapter {
        fixtures: input
            .offline_output_batch
            .as_ref()
            .map(|batch| batch.opinions.clone())
            .unwrap_or_default(),
    };
    let offline_keys: std::collections::BTreeSet<OfflineOpinionKey> = offline_adapter
        .fixtures
        .iter()
        .map(OfflineOpinionKey::from_fixture)
        .collect();
    let registry = AiMemberCoreRegistry::from_members_with_offline_hint(
        &input.members,
        !offline_adapter.fixtures.is_empty(),
    );
    let member_opinions: Vec<MemberOpinion> = router_output
        .packets
        .iter()
        .filter_map(|packet| {
            let member = input
                .members
                .iter()
                .find(|member| member.member_id == packet.member_id)?;
            let prefer_offline_fixture =
                offline_keys.contains(&OfflineOpinionKey::from_packet(packet));
            let core_spec = registry
                .get_core_spec(&member.member_id)
                .cloned()
                .unwrap_or_else(|| resolved_core_spec_for_member(member, prefer_offline_fixture));
            Some(
                CoreAwareMemberBrainAdapter {
                    member: member.clone(),
                    core_spec,
                    offline_adapter: offline_adapter.clone(),
                }
                .produce_opinion(packet),
            )
        })
        .collect();
    let event_queue = InvestmentEventQueue::from_member_opinions(&member_opinions);
    let mut committee_sessions = Vec::new();
    let mut chairman_decisions = Vec::new();
    let mut score_updates = Vec::new();
    let mut memory_updates = Vec::new();
    let mut learning_journal_entries = Vec::new();
    let mut risk_veto_count = 0;

    for event in &event_queue.events {
        let Some(market) = input.market_data.iter().find(|market| {
            market.symbol == event.symbol && market.market_scope == event.market_scope
        }) else {
            continue;
        };
        let scoped_members: Vec<AICommitteeMember> = input
            .members
            .iter()
            .filter(|member| {
                member.market_scopes.contains(&event.market_scope)
                    && !matches!(member.status, AICommitteeMemberStatus::Disabled)
            })
            .cloned()
            .collect();
        let scoped_opinions: Vec<MemberOpinion> = member_opinions
            .iter()
            .filter(|opinion| {
                opinion.symbol == event.symbol && opinion.market_scope == event.market_scope
            })
            .cloned()
            .collect();
        let session =
            build_committee_session(market, event.clone(), &scoped_members, &scoped_opinions);
        let decision =
            synthesize_chairman_decision(&session, default_risk_veto_volatility_threshold());
        if decision.risk_governor_status == RiskGovernorStatus::Vetoed {
            risk_veto_count += 1;
        }
        let feedback = PaperOutcomeFeedback {
            symbol: event.symbol.clone(),
            market_scope: event.market_scope,
            decision_id: decision.decision_id.clone(),
            simulated_result: input
                .paper_outcome
                .unwrap_or(SimulatedPaperOutcome::Unknown),
            notes: vec![
                "offline batch paper outcome feedback only".to_string(),
                "learning journal does not train model weights".to_string(),
            ],
        };
        let feedback_result = apply_paper_outcome_feedback(
            &scoped_members,
            &scoped_opinions,
            Some(&decision),
            &feedback,
        );
        committee_sessions.push(session);
        chairman_decisions.push(decision);
        score_updates.extend(feedback_result.score_updates);
        memory_updates.extend(feedback_result.updated_memory_states);
        learning_journal_entries.extend(feedback_result.learning_journal_entries);
    }

    let mut events_by_symbol = std::collections::BTreeMap::new();
    for (symbol, events) in event_queue.group_by_symbol() {
        events_by_symbol.insert(symbol, events.len());
    }
    let learning_journals = build_learning_journals(&learning_journal_entries);
    Ok(BatchCommitteeCycleResult {
        batch_id: input
            .offline_output_batch
            .as_ref()
            .map(|batch| batch.batch_id.clone())
            .unwrap_or_else(|| "mock-local-batch".to_string()),
        routed_packet_count: router_output.routed_member_count,
        member_opinion_count: member_opinions.len(),
        member_opinions,
        event_queue,
        events_by_symbol,
        committee_sessions,
        chairman_decisions,
        risk_veto_count,
        score_update_count: score_updates.len(),
        score_updates,
        memory_updates,
        learning_journal_entry_count: learning_journal_entries.len(),
        learning_journal_entries,
        learning_journals,
        safety_summary: safety_summary(),
    })
}

pub fn run_batch_committee_cycle_with_state(
    input: BatchCommitteeCycleWithStateInput,
) -> Result<BatchCommitteeCycleWithStateResult, String> {
    if let Some(path) = &input.member_state_output_path {
        if !local_only(path) {
            return Err("member state output path must be local".to_string());
        }
    }
    let mut store = input.member_state_store.clone().unwrap_or_else(|| {
        MemberStateStore::from_members(
            "minimal-ai-member-state-store",
            &input.batch_input.members,
            "initialized-from-batch-input",
        )
    });
    let market_data = input.batch_input.market_data.clone();
    let news = input.batch_input.news.clone();
    let batch_result = run_batch_committee_cycle(input.batch_input)?;
    store.apply_score_updates(&batch_result.score_updates);
    store.apply_memory_updates(&batch_result.memory_updates);
    store.append_learning_journal_entries(&batch_result.learning_journal_entries);
    if let Some(path) = &input.member_state_output_path {
        store.save_to_local_json(Path::new(path))?;
    }
    let state_update = BatchCycleStateUpdate {
        cycle_id: batch_result.batch_id.clone(),
        score_updates: batch_result.score_updates.clone(),
        memory_updates: batch_result.memory_updates.clone(),
        learning_journal_entries: batch_result.learning_journal_entries.clone(),
        updated_member_states: store.members.clone(),
    };
    let owner_summary = input
        .emit_owner_summary
        .then(|| build_owner_committee_summary(&batch_result));
    let owner_console_view = input.emit_owner_console_view.then(|| {
        build_owner_committee_console_view(&batch_result, &state_update, owner_summary.as_ref())
    });
    let owner_feedback_reconsideration = if input.emit_reconsideration_view
        && !input.owner_feedback.is_empty()
    {
        let previous_owner_console_view = owner_console_view.clone().unwrap_or_else(|| {
            build_owner_committee_console_view(&batch_result, &state_update, owner_summary.as_ref())
        });
        Some(run_owner_feedback_reconsideration_cycle(
            OwnerFeedbackReconsiderationInput {
                previous_batch_result: batch_result.clone(),
                previous_owner_console_view,
                owner_feedback: input.owner_feedback,
                member_state_store: store,
                market_data,
                news,
            },
        )?)
    } else {
        None
    };
    Ok(BatchCommitteeCycleWithStateResult {
        batch_result,
        state_update,
        owner_summary,
        owner_console_view,
        owner_feedback_reconsideration,
    })
}

pub fn run_autonomous_paper_committee_loop_from_config_path(
    path: &Path,
) -> Result<AutonomousPaperRunResult, String> {
    let config = MinimalAiCommitteeCycleConfig::from_toml_path(path)?;
    if !config.autonomous_paper_run {
        return Err("minimal AI committee autonomous_paper_run must be true".to_string());
    }
    run_autonomous_paper_committee_loop(config.load_autonomous_paper_run_config()?)
}

pub fn run_autonomous_paper_committee_loop(
    config: AutonomousPaperRunConfig,
) -> Result<AutonomousPaperRunResult, String> {
    validate_autonomous_paper_run_config(&config)?;
    let cycle_count = autonomous_cycle_count(config.cycle_mode, config.max_cycles);
    let mut store = config.member_state_store.clone().unwrap_or_else(|| {
        MemberStateStore::from_members(
            &format!("{}-member-state-store", config.run_id),
            &config.batch_input.members,
            "initialized-from-autonomous-paper-run",
        )
    });
    let mut cycles = Vec::new();
    let mut archive = PaperDecisionArchive {
        run_id: config.run_id.clone(),
        entries: Vec::new(),
    };

    for cycle_index in 0..cycle_count {
        let cycle_id = format!("{}-cycle-{}", config.run_id, cycle_index + 1);
        let owner_feedback = if cycle_index == 0 {
            config.owner_feedback.clone()
        } else {
            Vec::new()
        };
        let result = run_batch_committee_cycle_with_state(BatchCommitteeCycleWithStateInput {
            batch_input: config.batch_input.clone(),
            member_state_store: Some(store.clone()),
            member_state_output_path: None,
            emit_owner_summary: true,
            emit_owner_console_view: config.emit_owner_console_view,
            emit_reconsideration_view: !owner_feedback.is_empty(),
            owner_feedback,
        })?;
        store.members = result.state_update.updated_member_states.clone();
        store.source_label = cycle_id.clone();
        archive.append_cycle_decisions(&cycle_id, &result.batch_result);
        let attention_queue = OwnerAttentionQueue::from_batch_cycle_result(
            &format!("{}-attention-{}", config.run_id, cycle_index + 1),
            cycle_index,
            &result.batch_result,
            result.owner_feedback_reconsideration.as_ref(),
            config.require_owner_confirmation,
        );
        let owner_summary = result
            .owner_summary
            .clone()
            .unwrap_or_else(|| build_owner_committee_summary(&result.batch_result));
        cycles.push(AutonomousPaperCycle {
            cycle_id,
            cycle_index,
            market_scopes: market_scopes_from_batch_result(&result.batch_result),
            symbols: symbols_from_batch_result(&result.batch_result),
            batch_result: result.batch_result,
            state_update: result.state_update,
            owner_summary,
            owner_console_view: result.owner_console_view,
            owner_feedback_reconsideration: result.owner_feedback_reconsideration,
            attention_items: attention_queue.items,
            completed_at: Some("offline-autonomous-paper-cycle".to_string()),
        });
    }

    if let Some(path) = &config.member_state_output_path {
        store.save_to_local_json(Path::new(path))?;
    }
    let attention_queue =
        OwnerAttentionQueue::merge_cycles(&format!("{}-attention-queue", config.run_id), &cycles);
    let mut run_result = AutonomousPaperRunResult {
        run_id: config.run_id.clone(),
        cycle_count: cycles.len(),
        cycles,
        final_member_states: store.members,
        attention_queue,
        owner_attention_triage: None,
        watchlist_recheck: None,
        paper_decision_archive: archive,
        safety_summary: safety_summary(),
        paper_only_warning:
            "autonomous paper loop only schedules, routes, records, and summarizes; no orders"
                .to_string(),
    };
    if config.emit_owner_attention_inbox
        || config.owner_attention_inbox_input_path.is_some()
        || config.owner_attention_inbox_output_path.is_some()
        || config.owner_attention_actions_path.is_some()
        || config.watchlist_candidate_input_path.is_some()
        || config.watchlist_candidate_output_path.is_some()
    {
        let triage = run_owner_attention_triage(OwnerAttentionTriageInput {
            previous_run: run_result.clone(),
            previous_inbox: config.previous_owner_attention_inbox,
            owner_actions: config.owner_attention_actions,
            watchlist_store: config.watchlist_candidate_store,
        })?;
        if let Some(path) = &config.owner_attention_inbox_output_path {
            triage.inbox.save_to_local_json(Path::new(path))?;
        }
        if let Some(path) = &config.watchlist_candidate_output_path {
            triage.watchlist_store.save_to_local_json(Path::new(path))?;
        }
        run_result.owner_attention_triage = Some(triage);
    }
    if config.enable_watchlist_recheck {
        let watchlist_store = config
            .watchlist_recheck_store
            .clone()
            .or_else(|| {
                run_result
                    .owner_attention_triage
                    .as_ref()
                    .map(|triage| triage.watchlist_store.clone())
            })
            .unwrap_or_else(|| WatchlistCandidateStore::new("watchlist-recheck-empty-store"));
        let recheck = run_watchlist_recheck_cycle(WatchlistRecheckConfig {
            recheck_id: format!("{}-watchlist-recheck", config.run_id),
            watchlist_input_path: config.watchlist_input_path.clone(),
            watchlist_output_path: config
                .watchlist_output_path
                .clone()
                .or(config.watchlist_candidate_output_path.clone()),
            member_state_input_path: config.member_state_input_path.clone(),
            member_state_output_path: config.member_state_output_path.clone(),
            market_data_path: config.local_market_data_path.clone(),
            news_path: config.local_news_path.clone(),
            offline_member_output_batch_path: config.offline_member_output_batch_path.clone(),
            max_candidates_per_cycle: config.max_candidates_per_cycle,
            include_risk_blocked: config.include_risk_blocked,
            include_needs_evidence: config.include_needs_evidence,
            emit_owner_daily_brief: config.emit_owner_daily_brief,
            paper_only: true,
            watchlist_store,
            batch_input: config.batch_input.clone(),
            member_state_store: config.member_state_store.clone(),
        })?;
        if let Some(path) = &config.watchlist_output_path {
            recheck
                .updated_watchlist_store
                .save_to_local_json(Path::new(path))?;
        }
        run_result.watchlist_recheck = Some(recheck);
    }
    Ok(run_result)
}

fn validate_autonomous_paper_run_config(config: &AutonomousPaperRunConfig) -> Result<(), String> {
    if !config.paper_only {
        return Err("autonomous paper run must be paper-only".to_string());
    }
    if config.max_cycles == 0 {
        return Err("autonomous paper run max_cycles must be at least 1".to_string());
    }
    for path in [
        config.local_market_data_path.as_ref(),
        config.local_news_path.as_ref(),
        config.offline_member_output_batch_path.as_ref(),
        config.member_state_input_path.as_ref(),
        config.member_state_output_path.as_ref(),
        config.owner_feedback_path.as_ref(),
        config.owner_attention_inbox_input_path.as_ref(),
        config.owner_attention_inbox_output_path.as_ref(),
        config.owner_attention_actions_path.as_ref(),
        config.watchlist_candidate_input_path.as_ref(),
        config.watchlist_candidate_output_path.as_ref(),
        config.watchlist_input_path.as_ref(),
        config.watchlist_output_path.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        if !local_only(path) {
            return Err("autonomous paper run paths must be local".to_string());
        }
    }
    if config.batch_input.market_data.is_empty() {
        return Err("autonomous paper run requires market_data".to_string());
    }
    if config.max_candidates_per_cycle == 0 {
        return Err("watchlist recheck max_candidates_per_cycle must be at least 1".to_string());
    }
    Ok(())
}

fn autonomous_cycle_count(mode: AutonomousPaperCycleMode, max_cycles: usize) -> usize {
    match mode {
        AutonomousPaperCycleMode::SingleShot | AutonomousPaperCycleMode::ManualStep => 1,
        AutonomousPaperCycleMode::FixedCount => max_cycles.max(1),
    }
}

pub fn run_watchlist_recheck_cycle(
    config: WatchlistRecheckConfig,
) -> Result<WatchlistRecheckResult, String> {
    validate_watchlist_recheck_config(&config)?;
    let watchlist_store = load_watchlist_recheck_store(&config)?;
    let batch_input = load_watchlist_recheck_batch_input(&config)?;
    let member_state_store = load_watchlist_recheck_member_state_store(&config)?;
    let selection = select_watchlist_candidates_for_recheck(
        &watchlist_store,
        &batch_input,
        config.max_candidates_per_cycle,
        config.include_risk_blocked,
        config.include_needs_evidence,
    );
    if selection.selected_candidates.is_empty() {
        return Err("watchlist recheck requires at least one selected candidate".to_string());
    }
    let batch_input =
        batch_input_for_watchlist_candidates(&batch_input, &selection.selected_candidates);
    let stateful = run_batch_committee_cycle_with_state(BatchCommitteeCycleWithStateInput {
        batch_input,
        member_state_store,
        member_state_output_path: config.member_state_output_path.clone(),
        emit_owner_summary: true,
        emit_owner_console_view: true,
        owner_feedback: Vec::new(),
        emit_reconsideration_view: false,
    })?;
    let mut updated_watchlist_store = watchlist_store;
    let lifecycle_events = update_watchlist_lifecycle(
        &mut updated_watchlist_store,
        &selection.selected_candidates,
        &stateful.batch_result,
    );
    let attention_queue = OwnerAttentionQueue::from_batch_cycle_result(
        &format!("{}-attention", config.recheck_id),
        0,
        &stateful.batch_result,
        None,
        OwnerConfirmationPolicy::Never,
    );
    let owner_summary = stateful
        .owner_summary
        .clone()
        .unwrap_or_else(|| build_owner_committee_summary(&stateful.batch_result));
    let owner_daily_brief = config.emit_owner_daily_brief.then(|| {
        build_owner_daily_brief(
            &config.recheck_id,
            &selection.selected_candidates,
            &lifecycle_events,
            &stateful.batch_result,
            &owner_summary,
            &attention_queue.items,
        )
    });
    if let Some(path) = &config.watchlist_output_path {
        updated_watchlist_store.save_to_local_json(Path::new(path))?;
    }
    Ok(WatchlistRecheckResult {
        recheck_id: config.recheck_id,
        selected_candidates: selection.selected_candidates.clone(),
        selection,
        batch_result: stateful.batch_result,
        owner_summary,
        owner_console_view: stateful.owner_console_view,
        generated_attention_items: attention_queue.items,
        lifecycle_events,
        updated_watchlist_store,
        owner_daily_brief,
        safety_summary: safety_summary(),
        paper_only_warning:
            "watchlist recheck is paper-only; candidates are not orders or positions".to_string(),
    })
}

fn validate_watchlist_recheck_config(config: &WatchlistRecheckConfig) -> Result<(), String> {
    if !config.paper_only {
        return Err("watchlist recheck config must be paper-only".to_string());
    }
    if config.max_candidates_per_cycle == 0 {
        return Err("watchlist recheck max_candidates_per_cycle must be at least 1".to_string());
    }
    if config.watchlist_input_path.is_none() {
        validate_watchlist_candidate_store(&config.watchlist_store)?;
    }
    for path in [
        config.watchlist_input_path.as_ref(),
        config.watchlist_output_path.as_ref(),
        config.member_state_input_path.as_ref(),
        config.member_state_output_path.as_ref(),
        config.market_data_path.as_ref(),
        config.news_path.as_ref(),
        config.offline_member_output_batch_path.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        if !local_only(path) {
            return Err("watchlist recheck paths must be local".to_string());
        }
    }
    Ok(())
}

fn load_watchlist_recheck_store(
    config: &WatchlistRecheckConfig,
) -> Result<WatchlistCandidateStore, String> {
    if let Some(path) = &config.watchlist_input_path {
        WatchlistCandidateStore::load_from_local_json(Path::new(path))
    } else {
        Ok(config.watchlist_store.clone())
    }
}

fn load_watchlist_recheck_batch_input(
    config: &WatchlistRecheckConfig,
) -> Result<BatchCommitteeCycleInput, String> {
    let mut batch_input = config.batch_input.clone();
    if let Some(path) = &config.market_data_path {
        batch_input.market_data = load_market_data_from_local_json(Path::new(path))?;
    }
    if let Some(path) = &config.news_path {
        batch_input.news = load_news_from_local_json(Path::new(path))?;
    }
    if let Some(path) = &config.offline_member_output_batch_path {
        let load = OfflineMemberOutputBatch::from_json_path(Path::new(path))?;
        batch_input.offline_output_batch = Some(OfflineMemberOutputBatch {
            batch_id: load.batch_id,
            created_at: "loaded-from-local-file".to_string(),
            source_label: path.clone(),
            opinions: load.opinions,
        });
    }
    if batch_input.market_data.is_empty() {
        return Err("watchlist recheck requires market_data".to_string());
    }
    Ok(batch_input)
}

fn load_watchlist_recheck_member_state_store(
    config: &WatchlistRecheckConfig,
) -> Result<Option<MemberStateStore>, String> {
    if let Some(path) = &config.member_state_input_path {
        MemberStateStore::load_from_local_json(Path::new(path)).map(Some)
    } else {
        Ok(config.member_state_store.clone())
    }
}

fn select_watchlist_candidates_for_recheck(
    store: &WatchlistCandidateStore,
    batch_input: &BatchCommitteeCycleInput,
    max_candidates: usize,
    include_risk_blocked: bool,
    include_needs_evidence: bool,
) -> WatchlistRecheckSelection {
    let mut selected_candidates = Vec::new();
    let mut skipped_candidates = Vec::new();
    for candidate in &store.candidates {
        let skip_reason = if !candidate.paper_only || candidate.symbol.trim().is_empty() {
            Some(WatchlistRecheckSkipReason::InvalidCandidate)
        } else if candidate.status == WatchlistCandidateStatus::Archived {
            Some(WatchlistRecheckSkipReason::Archived)
        } else if candidate.status == WatchlistCandidateStatus::RiskBlocked && !include_risk_blocked
        {
            Some(WatchlistRecheckSkipReason::RiskBlockedExcluded)
        } else if candidate.status == WatchlistCandidateStatus::NeedsEvidence
            && !include_needs_evidence
        {
            Some(WatchlistRecheckSkipReason::MissingNews)
        } else if !batch_input.market_data.iter().any(|market| {
            market.symbol == candidate.symbol && market.market_scope == candidate.market_scope
        }) {
            Some(WatchlistRecheckSkipReason::MissingMarketData)
        } else if !batch_input
            .news
            .iter()
            .any(|news| news.symbol == candidate.symbol)
        {
            Some(WatchlistRecheckSkipReason::MissingNews)
        } else if selected_candidates.len() >= max_candidates {
            Some(WatchlistRecheckSkipReason::OverCandidateLimit)
        } else {
            None
        };
        if let Some(reason) = skip_reason {
            skipped_candidates.push(SkippedWatchlistCandidate {
                candidate: candidate.clone(),
                reason,
            });
        } else {
            selected_candidates.push(candidate.clone());
        }
    }
    let mut skip_reasons: Vec<WatchlistRecheckSkipReason> = skipped_candidates
        .iter()
        .map(|skipped| skipped.reason)
        .collect();
    skip_reasons.sort();
    skip_reasons.dedup();
    WatchlistRecheckSelection {
        selected_count: selected_candidates.len(),
        skipped_count: skipped_candidates.len(),
        selected_candidates,
        skipped_candidates,
        skip_reasons,
    }
}

fn batch_input_for_watchlist_candidates(
    input: &BatchCommitteeCycleInput,
    candidates: &[WatchlistCandidate],
) -> BatchCommitteeCycleInput {
    let mut batch = input.clone();
    batch.market_data.retain(|market| {
        candidates.iter().any(|candidate| {
            candidate.symbol == market.symbol && candidate.market_scope == market.market_scope
        })
    });
    batch.news.retain(|news| {
        candidates
            .iter()
            .any(|candidate| candidate.symbol == news.symbol)
    });
    if let Some(offline_batch) = &mut batch.offline_output_batch {
        offline_batch.opinions.retain(|opinion| {
            candidates.iter().any(|candidate| {
                candidate.symbol == opinion.symbol && candidate.market_scope == opinion.market_scope
            })
        });
    }
    batch.members.retain(|member| {
        candidates.iter().any(|candidate| {
            member.market_scopes.contains(&candidate.market_scope)
                && !matches!(member.status, AICommitteeMemberStatus::Disabled)
        })
    });
    batch
}

fn update_watchlist_lifecycle(
    store: &mut WatchlistCandidateStore,
    selected_candidates: &[WatchlistCandidate],
    batch_result: &BatchCommitteeCycleResult,
) -> Vec<WatchlistCandidateLifecycleEvent> {
    let mut events = Vec::new();
    for candidate in selected_candidates {
        let decision = chairman_decision_for_symbol_scope(
            batch_result,
            &candidate.symbol,
            candidate.market_scope,
        );
        let previous_status = candidate.status;
        let (new_status, reason, decision_id) = if let Some(decision) = decision {
            if decision.risk_governor_status == RiskGovernorStatus::Vetoed {
                (
                    WatchlistCandidateStatus::RiskBlocked,
                    "Risk Governor vetoed watchlist recheck".to_string(),
                    Some(decision.decision_id.clone()),
                )
            } else if decision.final_action == ChairmanFinalAction::NeedMoreEvidence
                || decision.risk_governor_status == RiskGovernorStatus::NeedsReview
            {
                (
                    WatchlistCandidateStatus::NeedsEvidence,
                    "committee needs more evidence for watchlist candidate".to_string(),
                    Some(decision.decision_id.clone()),
                )
            } else if matches!(
                decision.final_action,
                ChairmanFinalAction::PaperBuy | ChairmanFinalAction::PaperSell
            ) && decision.risk_governor_status == RiskGovernorStatus::Passed
            {
                (
                    WatchlistCandidateStatus::PaperCandidate,
                    "AI members produced a passed paper candidate; not an order".to_string(),
                    Some(decision.decision_id.clone()),
                )
            } else {
                (
                    WatchlistCandidateStatus::Watching,
                    "watchlist candidate remains under paper observation".to_string(),
                    Some(decision.decision_id.clone()),
                )
            }
        } else {
            (
                WatchlistCandidateStatus::NeedsEvidence,
                "no committee decision was available for watchlist candidate".to_string(),
                None,
            )
        };
        if let Some(stored) = store
            .candidates
            .iter_mut()
            .find(|stored| stored.candidate_id == candidate.candidate_id)
        {
            stored.status = new_status;
        }
        events.push(WatchlistCandidateLifecycleEvent {
            event_id: format!("lifecycle-{}-{:?}", candidate.candidate_id, new_status),
            candidate_id: candidate.candidate_id.clone(),
            symbol: candidate.symbol.clone(),
            market_scope: candidate.market_scope,
            previous_status,
            new_status,
            reason,
            related_decision_id: decision_id,
            related_attention_item_id: Some(candidate.source_attention_item_id.clone()),
            paper_only: true,
        });
    }
    store.refresh_counts();
    events
}

fn chairman_decision_for_symbol_scope<'a>(
    result: &'a BatchCommitteeCycleResult,
    symbol: &str,
    market_scope: MarketScope,
) -> Option<&'a ChairmanDecision> {
    result.chairman_decisions.iter().find(|decision| {
        result
            .committee_sessions
            .iter()
            .find(|session| session.session_id == decision.session_id)
            .is_some_and(|session| {
                session.event.symbol == symbol && session.event.market_scope == market_scope
            })
    })
}

fn build_owner_daily_brief(
    recheck_id: &str,
    selected_candidates: &[WatchlistCandidate],
    lifecycle_events: &[WatchlistCandidateLifecycleEvent],
    batch_result: &BatchCommitteeCycleResult,
    owner_summary: &OwnerCommitteeSummary,
    attention_items: &[OwnerAttentionItem],
) -> OwnerDailyBrief {
    let reviewed_symbols = selected_candidates
        .iter()
        .map(|candidate| candidate.symbol.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let watchlist_updates: Vec<String> = lifecycle_events
        .iter()
        .map(|event| {
            format!(
                "{}:{:?}->{:?}",
                event.symbol, event.previous_status, event.new_status
            )
        })
        .collect();
    let risk_vetoes: Vec<String> = lifecycle_events
        .iter()
        .filter(|event| event.new_status == WatchlistCandidateStatus::RiskBlocked)
        .map(|event| event.symbol.clone())
        .collect();
    let need_more_evidence_items: Vec<String> = lifecycle_events
        .iter()
        .filter(|event| event.new_status == WatchlistCandidateStatus::NeedsEvidence)
        .map(|event| event.symbol.clone())
        .collect();
    let paper_candidates: Vec<String> = lifecycle_events
        .iter()
        .filter(|event| event.new_status == WatchlistCandidateStatus::PaperCandidate)
        .map(|event| event.symbol.clone())
        .collect();
    let archived_candidates: Vec<String> = lifecycle_events
        .iter()
        .filter(|event| event.new_status == WatchlistCandidateStatus::Archived)
        .map(|event| event.symbol.clone())
        .collect();
    let top_member_voice_changes: Vec<String> = owner_summary
        .member_voice_changes
        .iter()
        .take(5)
        .map(|change| {
            format!(
                "{}:{}->{:.2}",
                change.member_id, change.direction, change.new_voice_weight
            )
        })
        .collect();
    let key_ai_opinions: Vec<String> = batch_result
        .member_opinions
        .iter()
        .filter(|opinion| opinion.event_triggered)
        .take(8)
        .map(|opinion| {
            format!(
                "{}:{}:{:?}:{:.2}",
                opinion.symbol, opinion.member_id, opinion.stance, opinion.confidence
            )
        })
        .collect();
    let next_owner_attention: Vec<String> = attention_items
        .iter()
        .take(5)
        .map(|item| format!("{:?}:{}", item.attention_type, item.reason))
        .collect();
    let brief_text = format!(
        "Watchlist recheck reviewed {} symbols; risk_vetoes={}, need_more_evidence={}, paper_candidates={}. This is paper-only and not an order.",
        reviewed_symbols.len(),
        risk_vetoes.len(),
        need_more_evidence_items.len(),
        paper_candidates.len()
    );
    OwnerDailyBrief {
        brief_id: format!("brief-{recheck_id}"),
        reviewed_symbols,
        watchlist_updates,
        risk_vetoes,
        need_more_evidence_items,
        paper_candidates,
        archived_candidates,
        top_member_voice_changes,
        key_ai_opinions,
        next_owner_attention,
        brief_text,
        paper_only_warning:
            "owner daily brief is advisory paper-only output, not investment advice or trading"
                .to_string(),
    }
}

pub fn run_owner_attention_triage(
    input: OwnerAttentionTriageInput,
) -> Result<OwnerAttentionTriageResult, String> {
    let mut inbox = input.previous_inbox.unwrap_or_else(|| {
        OwnerAttentionInbox::from_attention_queue(&input.previous_run.attention_queue)
    });
    inbox.merge_new_items(&input.previous_run.attention_queue);
    let mut watchlist_store = input
        .watchlist_store
        .unwrap_or_else(|| WatchlistCandidateStore::new("owner-watchlist-candidates"));
    let (action_results, generated_owner_feedback, generated_watchlist_candidates) =
        inbox.apply_owner_actions(&input.owner_actions, &mut watchlist_store);
    Ok(OwnerAttentionTriageResult {
        inbox,
        action_results,
        generated_owner_feedback_count: generated_owner_feedback.len(),
        generated_owner_feedback,
        generated_watchlist_candidate_count: generated_watchlist_candidates.len(),
        generated_watchlist_candidates,
        watchlist_store,
        safety_summary: safety_summary(),
        paper_only_warning:
            "owner attention triage is paper-only; actions cannot create orders or bypass risk"
                .to_string(),
    })
}

impl OwnerAttentionInbox {
    pub fn from_attention_queue(queue: &OwnerAttentionQueue) -> Self {
        let items = queue
            .items
            .iter()
            .map(owner_attention_inbox_item_from_queue_item)
            .collect();
        Self::from_items(&format!("{}-inbox", queue.queue_id), items)
    }

    pub fn merge_new_items(&mut self, queue: &OwnerAttentionQueue) {
        self.items.extend(
            queue
                .items
                .iter()
                .map(owner_attention_inbox_item_from_queue_item),
        );
        self.dedupe_by_symbol_scope_type();
        self.refresh_counts();
    }

    pub fn dedupe_by_symbol_scope_type(&mut self) {
        let mut seen = std::collections::BTreeSet::new();
        self.items.retain(|item| {
            seen.insert(format!(
                "{:?}|{:?}|{:?}",
                item.symbol, item.market_scope, item.attention_type
            ))
        });
    }

    pub fn open_items(&self) -> Vec<OwnerAttentionInboxItem> {
        self.items
            .iter()
            .filter(|item| item.status == OwnerAttentionInboxStatus::Open)
            .cloned()
            .collect()
    }

    pub fn high_priority_items(&self) -> Vec<OwnerAttentionInboxItem> {
        self.items
            .iter()
            .filter(|item| item.priority == OwnerAttentionPriority::High)
            .cloned()
            .collect()
    }

    pub fn items_requiring_owner_input(&self) -> Vec<OwnerAttentionInboxItem> {
        self.items
            .iter()
            .filter(|item| item.status == OwnerAttentionInboxStatus::Open)
            .filter(|item| item.requires_owner_input)
            .cloned()
            .collect()
    }

    pub fn apply_owner_actions(
        &mut self,
        actions: &[OwnerAttentionAction],
        watchlist_store: &mut WatchlistCandidateStore,
    ) -> (
        Vec<OwnerAttentionActionResult>,
        Vec<OwnerFeedback>,
        Vec<WatchlistCandidate>,
    ) {
        let mut results = Vec::new();
        let mut feedback = Vec::new();
        let mut candidates = Vec::new();
        for action in actions {
            let item_index = self
                .items
                .iter()
                .position(|item| item.item_id == action.item_id);
            let previous_status = item_index
                .map(|index| self.items[index].status)
                .unwrap_or(OwnerAttentionInboxStatus::Open);
            let validation = validate_owner_attention_action(action);
            if let Err(reason) = validation {
                results.push(rejected_owner_attention_action_result(
                    action,
                    previous_status,
                    reason,
                ));
                continue;
            }
            let Some(index) = item_index else {
                results.push(rejected_owner_attention_action_result(
                    action,
                    previous_status,
                    "owner attention action item_id not found".to_string(),
                ));
                continue;
            };
            let item = self.items[index].clone();
            let mut generated_feedback = None;
            let mut generated_candidate = None;
            let new_status = match action.action_type {
                OwnerAttentionActionType::Acknowledge => OwnerAttentionInboxStatus::Acknowledged,
                OwnerAttentionActionType::Defer => OwnerAttentionInboxStatus::Deferred,
                OwnerAttentionActionType::Dismiss => OwnerAttentionInboxStatus::Dismissed,
                OwnerAttentionActionType::ConvertToWatchlist => {
                    if let Some(candidate) = watchlist_candidate_from_action(action, &item) {
                        watchlist_store.add_candidate(candidate.clone());
                        candidates.push(candidate.clone());
                        generated_candidate = Some(candidate);
                        OwnerAttentionInboxStatus::ConvertedToWatchlist
                    } else {
                        results.push(rejected_owner_attention_action_result(
                            action,
                            previous_status,
                            "watchlist conversion requires symbol and market_scope".to_string(),
                        ));
                        continue;
                    }
                }
                OwnerAttentionActionType::RequestMoreEvidence => {
                    let item_feedback = owner_feedback_from_action(
                        action,
                        &item,
                        OwnerFeedbackType::EvidenceRequest,
                    );
                    feedback.push(item_feedback.clone());
                    generated_feedback = Some(item_feedback);
                    OwnerAttentionInboxStatus::ConvertedToFeedback
                }
                OwnerAttentionActionType::RequestReconsideration => {
                    let item_feedback = owner_feedback_from_action(
                        action,
                        &item,
                        OwnerFeedbackType::ReconsiderationRequest,
                    );
                    feedback.push(item_feedback.clone());
                    generated_feedback = Some(item_feedback);
                    OwnerAttentionInboxStatus::ReconsiderationRequested
                }
                OwnerAttentionActionType::AddComment => {
                    let item_feedback =
                        owner_feedback_from_action(action, &item, OwnerFeedbackType::Comment);
                    feedback.push(item_feedback.clone());
                    generated_feedback = Some(item_feedback);
                    OwnerAttentionInboxStatus::ConvertedToFeedback
                }
            };
            self.items[index].status = new_status;
            self.items[index].updated_at = action.created_at.clone();
            results.push(OwnerAttentionActionResult {
                action_id: action.action_id.clone(),
                item_id: action.item_id.clone(),
                previous_status,
                new_status,
                generated_owner_feedback: generated_feedback,
                generated_watchlist_candidate: generated_candidate,
                safety_status: OwnerAttentionActionSafetyStatus::Passed,
                rejection_reason: None,
            });
        }
        self.refresh_counts();
        (results, feedback, candidates)
    }

    pub fn save_to_local_json(&self, path: &Path) -> Result<(), String> {
        if !local_only(&path.to_string_lossy()) {
            return Err("owner attention inbox path must be local".to_string());
        }
        validate_owner_attention_inbox(self)?;
        let text = serde_json::to_string_pretty(self).map_err(|err| err.to_string())?;
        fs::write(path, text).map_err(|err| err.to_string())
    }

    pub fn load_from_local_json(path: &Path) -> Result<Self, String> {
        if !local_only(&path.to_string_lossy()) {
            return Err("owner attention inbox path must be local".to_string());
        }
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        reject_unsafe_owner_attention_text(&text)?;
        let mut inbox: Self = serde_json::from_str(&text).map_err(|err| err.to_string())?;
        validate_owner_attention_inbox(&inbox)?;
        inbox.refresh_counts();
        Ok(inbox)
    }

    fn from_items(inbox_id: &str, items: Vec<OwnerAttentionInboxItem>) -> Self {
        let mut inbox = Self {
            inbox_id: inbox_id.to_string(),
            items,
            open_count: 0,
            high_priority_count: 0,
            requires_owner_input_count: 0,
            last_updated_at: Some("offline-owner-attention-inbox".to_string()),
            paper_only: true,
        };
        inbox.dedupe_by_symbol_scope_type();
        inbox.refresh_counts();
        inbox
    }

    fn refresh_counts(&mut self) {
        self.items.sort_by(|left, right| {
            owner_attention_priority_rank(right.priority)
                .cmp(&owner_attention_priority_rank(left.priority))
                .then_with(|| left.item_id.cmp(&right.item_id))
        });
        self.open_count = self
            .items
            .iter()
            .filter(|item| item.status == OwnerAttentionInboxStatus::Open)
            .count();
        self.high_priority_count = self
            .items
            .iter()
            .filter(|item| item.priority == OwnerAttentionPriority::High)
            .count();
        self.requires_owner_input_count = self.items_requiring_owner_input().len();
    }
}

impl WatchlistCandidateStore {
    pub fn new(store_id: &str) -> Self {
        Self {
            store_id: store_id.to_string(),
            candidates: Vec::new(),
            active_count: 0,
            risk_blocked_count: 0,
            needs_evidence_count: 0,
            paper_only: true,
        }
    }

    pub fn add_candidate(&mut self, candidate: WatchlistCandidate) {
        if !self
            .candidates
            .iter()
            .any(|existing| existing.candidate_id == candidate.candidate_id)
        {
            self.candidates.push(candidate);
        }
        self.refresh_counts();
    }

    pub fn archive_candidate(&mut self, candidate_id: &str) {
        if let Some(candidate) = self
            .candidates
            .iter_mut()
            .find(|candidate| candidate.candidate_id == candidate_id)
        {
            candidate.status = WatchlistCandidateStatus::Archived;
        }
        self.refresh_counts();
    }

    pub fn candidates_by_symbol(&self, symbol: &str) -> Vec<WatchlistCandidate> {
        self.candidates
            .iter()
            .filter(|candidate| candidate.symbol == symbol)
            .cloned()
            .collect()
    }

    pub fn save_to_local_json(&self, path: &Path) -> Result<(), String> {
        if !local_only(&path.to_string_lossy()) {
            return Err("watchlist candidate store path must be local".to_string());
        }
        validate_watchlist_candidate_store(self)?;
        let text = serde_json::to_string_pretty(self).map_err(|err| err.to_string())?;
        fs::write(path, text).map_err(|err| err.to_string())
    }

    pub fn load_from_local_json(path: &Path) -> Result<Self, String> {
        if !local_only(&path.to_string_lossy()) {
            return Err("watchlist candidate store path must be local".to_string());
        }
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        reject_unsafe_owner_attention_text(&text)?;
        let mut store: Self = serde_json::from_str(&text).map_err(|err| err.to_string())?;
        validate_watchlist_candidate_store(&store)?;
        store.refresh_counts();
        Ok(store)
    }

    fn refresh_counts(&mut self) {
        self.active_count = self
            .candidates
            .iter()
            .filter(|candidate| !matches!(candidate.status, WatchlistCandidateStatus::Archived))
            .count();
        self.risk_blocked_count = self
            .candidates
            .iter()
            .filter(|candidate| candidate.status == WatchlistCandidateStatus::RiskBlocked)
            .count();
        self.needs_evidence_count = self
            .candidates
            .iter()
            .filter(|candidate| candidate.status == WatchlistCandidateStatus::NeedsEvidence)
            .count();
    }
}

pub fn load_owner_attention_actions_from_local_json(
    path: &Path,
) -> Result<Vec<OwnerAttentionAction>, String> {
    if !local_only(&path.to_string_lossy()) {
        return Err("owner attention actions path must be local".to_string());
    }
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    reject_unsafe_owner_attention_text(&text)?;
    let actions: Vec<OwnerAttentionAction> =
        serde_json::from_str(&text).map_err(|err| err.to_string())?;
    for action in &actions {
        validate_owner_attention_action(action)?;
    }
    Ok(actions)
}

fn owner_attention_inbox_item_from_queue_item(
    item: &OwnerAttentionItem,
) -> OwnerAttentionInboxItem {
    OwnerAttentionInboxItem {
        item_id: item.item_id.clone(),
        symbol: item.symbol.clone(),
        market_scope: item.market_scope,
        attention_type: item.attention_type,
        priority: item.priority,
        status: OwnerAttentionInboxStatus::Open,
        reason: item.reason.clone(),
        related_member_ids: item.related_member_ids.clone(),
        related_decision_id: item.related_decision_id.clone(),
        requires_owner_input: item.requires_owner_input,
        created_at: Some("offline-owner-attention-queue".to_string()),
        updated_at: None,
        paper_only: true,
    }
}

fn validate_owner_attention_action(action: &OwnerAttentionAction) -> Result<(), String> {
    if !action.paper_only {
        return Err("owner attention action must be paper-only".to_string());
    }
    if action.action_id.trim().is_empty() || action.item_id.trim().is_empty() {
        return Err("owner attention action requires action_id and item_id".to_string());
    }
    if let Some(comment) = &action.comment {
        reject_unsafe_owner_attention_text(comment)?;
    }
    Ok(())
}

fn validate_owner_attention_inbox(inbox: &OwnerAttentionInbox) -> Result<(), String> {
    if !inbox.paper_only {
        return Err("owner attention inbox must be paper-only".to_string());
    }
    for item in &inbox.items {
        if !item.paper_only {
            return Err("owner attention inbox item must be paper-only".to_string());
        }
        reject_unsafe_owner_attention_text(&item.reason)?;
    }
    Ok(())
}

fn validate_watchlist_candidate_store(store: &WatchlistCandidateStore) -> Result<(), String> {
    if !store.paper_only {
        return Err("watchlist candidate store must be paper-only".to_string());
    }
    for candidate in &store.candidates {
        if !candidate.paper_only {
            return Err("watchlist candidate must be paper-only".to_string());
        }
        reject_unsafe_owner_attention_text(&candidate.reason)?;
    }
    Ok(())
}

fn reject_unsafe_owner_attention_text(text: &str) -> Result<(), String> {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(text) {
        reject_unsafe_offline_batch_value(&value)?;
        reject_unsafe_owner_attention_value(&value)
    } else {
        reject_unsafe_owner_attention_string(text)
    }
}

fn reject_unsafe_owner_attention_value(value: &serde_json::Value) -> Result<(), String> {
    match value {
        serde_json::Value::Object(map) => {
            for (key, value) in map {
                reject_unsafe_owner_attention_string(key)?;
                reject_unsafe_owner_attention_value(value)?;
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                reject_unsafe_owner_attention_value(item)?;
            }
        }
        serde_json::Value::String(text) => reject_unsafe_owner_attention_string(text)?,
        _ => {}
    }
    Ok(())
}

fn reject_unsafe_owner_attention_string(text: &str) -> Result<(), String> {
    let lower = text.to_ascii_lowercase();
    for prohibited in [
        "buy with real money",
        "sell with real money",
        "execute",
        "order",
        "broker",
        "account",
        "holdings",
        "buying power",
        "leverage",
        "max position",
        "guaranteed return",
        "private info",
        "illegal info",
        "live trading",
        "주문",
        "계좌",
        "실거래",
    ] {
        if lower.contains(prohibited) {
            return Err(format!(
                "owner attention action rejected unsafe instruction: {prohibited}"
            ));
        }
    }
    Ok(())
}

fn rejected_owner_attention_action_result(
    action: &OwnerAttentionAction,
    previous_status: OwnerAttentionInboxStatus,
    reason: String,
) -> OwnerAttentionActionResult {
    OwnerAttentionActionResult {
        action_id: action.action_id.clone(),
        item_id: action.item_id.clone(),
        previous_status,
        new_status: previous_status,
        generated_owner_feedback: None,
        generated_watchlist_candidate: None,
        safety_status: OwnerAttentionActionSafetyStatus::Rejected,
        rejection_reason: Some(reason),
    }
}

fn owner_feedback_from_action(
    action: &OwnerAttentionAction,
    item: &OwnerAttentionInboxItem,
    feedback_type: OwnerFeedbackType,
) -> OwnerFeedback {
    OwnerFeedback {
        feedback_id: format!("feedback-{}", action.action_id),
        symbol: item.symbol.clone(),
        market_scope: item.market_scope,
        target_member_id: None,
        feedback_type,
        text: action
            .comment
            .clone()
            .unwrap_or_else(|| item.reason.clone()),
        priority: match item.priority {
            OwnerAttentionPriority::High => OwnerFeedbackPriority::High,
            OwnerAttentionPriority::Normal => OwnerFeedbackPriority::Normal,
            OwnerAttentionPriority::Low => OwnerFeedbackPriority::Low,
        },
        created_at: action.created_at.clone(),
        paper_only: true,
    }
}

fn watchlist_candidate_from_action(
    action: &OwnerAttentionAction,
    item: &OwnerAttentionInboxItem,
) -> Option<WatchlistCandidate> {
    Some(WatchlistCandidate {
        candidate_id: format!("watchlist-{}", action.action_id),
        symbol: item.symbol.clone()?,
        market_scope: item.market_scope?,
        source_attention_item_id: item.item_id.clone(),
        reason: action
            .comment
            .clone()
            .unwrap_or_else(|| item.reason.clone()),
        status: match item.attention_type {
            OwnerAttentionType::RiskVeto => WatchlistCandidateStatus::RiskBlocked,
            OwnerAttentionType::NeedMoreEvidence => WatchlistCandidateStatus::NeedsEvidence,
            OwnerAttentionType::WatchlistCandidate => WatchlistCandidateStatus::Watching,
            _ => WatchlistCandidateStatus::PaperCandidate,
        },
        created_at: action.created_at.clone(),
        paper_only: true,
    })
}

impl OwnerAttentionQueue {
    pub fn from_batch_cycle_result(
        queue_id: &str,
        cycle_index: usize,
        result: &BatchCommitteeCycleResult,
        reconsideration: Option<&OwnerFeedbackReconsiderationResult>,
        confirmation_policy: OwnerConfirmationPolicy,
    ) -> Self {
        let mut items = Vec::new();
        for decision in &result.chairman_decisions {
            let session = result
                .committee_sessions
                .iter()
                .find(|session| session.session_id == decision.session_id);
            let symbol = session.map(|session| session.event.symbol.clone());
            let market_scope = session.map(|session| session.event.market_scope);
            if decision.risk_governor_status == RiskGovernorStatus::Vetoed {
                items.push(owner_attention_item(
                    cycle_index,
                    items.len(),
                    symbol.clone(),
                    market_scope,
                    OwnerAttentionType::RiskVeto,
                    OwnerAttentionPriority::High,
                    decision.rationale.clone(),
                    decision.winning_arguments.clone(),
                    Some(decision.decision_id.clone()),
                    confirmation_policy,
                ));
            }
            if decision.final_action == ChairmanFinalAction::NeedMoreEvidence {
                items.push(owner_attention_item(
                    cycle_index,
                    items.len(),
                    symbol.clone(),
                    market_scope,
                    OwnerAttentionType::NeedMoreEvidence,
                    OwnerAttentionPriority::Normal,
                    decision.rationale.clone(),
                    decision.winning_arguments.clone(),
                    Some(decision.decision_id.clone()),
                    confirmation_policy,
                ));
            }
            if decision.final_action == ChairmanFinalAction::PaperHold {
                items.push(owner_attention_item(
                    cycle_index,
                    items.len(),
                    symbol.clone(),
                    market_scope,
                    OwnerAttentionType::WatchlistCandidate,
                    OwnerAttentionPriority::Normal,
                    "chairman kept this as paper hold/watchlist candidate".to_string(),
                    decision.winning_arguments.clone(),
                    Some(decision.decision_id.clone()),
                    confirmation_policy,
                ));
            }
            if session.is_some_and(|session| session.disagreement_level >= 0.5) {
                items.push(owner_attention_item(
                    cycle_index,
                    items.len(),
                    symbol,
                    market_scope,
                    OwnerAttentionType::HighDisagreement,
                    OwnerAttentionPriority::Normal,
                    "committee disagreement is elevated".to_string(),
                    session
                        .map(|session| session.invited_members.clone())
                        .unwrap_or_default(),
                    Some(decision.decision_id.clone()),
                    confirmation_policy,
                ));
            }
        }
        for opinion in &result.member_opinions {
            if matches!(
                opinion.stance,
                MemberStance::BuyProposal | MemberStance::SellProposal
            ) && opinion.confidence >= 0.8
            {
                items.push(owner_attention_item(
                    cycle_index,
                    items.len(),
                    Some(opinion.symbol.clone()),
                    Some(opinion.market_scope),
                    OwnerAttentionType::HighConfidenceEntry,
                    OwnerAttentionPriority::Normal,
                    "high confidence paper entry/exit proposal".to_string(),
                    vec![opinion.member_id.clone()],
                    None,
                    confirmation_policy,
                ));
            }
        }
        for update in &result.score_updates {
            if matches!(
                update.update_reason,
                MemberScoreUpdateReason::BadCall | MemberScoreUpdateReason::RiskyCall
            ) {
                items.push(owner_attention_item(
                    cycle_index,
                    items.len(),
                    None,
                    None,
                    OwnerAttentionType::RepeatedBadCall,
                    OwnerAttentionPriority::Low,
                    "paper learning journal noted a risky or bad call".to_string(),
                    vec![update.member_id.clone()],
                    None,
                    confirmation_policy,
                ));
            }
        }
        if let Some(reconsideration) = reconsideration {
            if reconsideration.owner_feedback_count > 0 {
                items.push(owner_attention_item(
                    cycle_index,
                    items.len(),
                    None,
                    None,
                    OwnerAttentionType::OwnerFeedbackAvailable,
                    OwnerAttentionPriority::Low,
                    "owner feedback was available for this paper cycle".to_string(),
                    Vec::new(),
                    None,
                    confirmation_policy,
                ));
            }
            for decision in &reconsideration.chairman_reconsideration_decisions {
                if decision.final_action == ChairmanReconsiderationFinalAction::RiskVetoed {
                    items.push(owner_attention_item(
                        cycle_index,
                        items.len(),
                        reconsideration_symbol_for_decision(reconsideration, decision),
                        reconsideration_scope_for_decision(reconsideration, decision),
                        OwnerAttentionType::RiskVeto,
                        OwnerAttentionPriority::High,
                        decision.rationale.clone(),
                        decision.what_changed.clone(),
                        Some(decision.decision_id.clone()),
                        confirmation_policy,
                    ));
                }
                if decision.final_action == ChairmanReconsiderationFinalAction::NeedMoreEvidence {
                    items.push(owner_attention_item(
                        cycle_index,
                        items.len(),
                        reconsideration_symbol_for_decision(reconsideration, decision),
                        reconsideration_scope_for_decision(reconsideration, decision),
                        OwnerAttentionType::NeedMoreEvidence,
                        OwnerAttentionPriority::Normal,
                        decision.rationale.clone(),
                        decision.what_changed.clone(),
                        Some(decision.decision_id.clone()),
                        confirmation_policy,
                    ));
                }
                if decision.final_action == ChairmanReconsiderationFinalAction::PaperHold {
                    items.push(owner_attention_item(
                        cycle_index,
                        items.len(),
                        reconsideration_symbol_for_decision(reconsideration, decision),
                        reconsideration_scope_for_decision(reconsideration, decision),
                        OwnerAttentionType::WatchlistCandidate,
                        OwnerAttentionPriority::Normal,
                        decision.rationale.clone(),
                        decision.what_changed.clone(),
                        Some(decision.decision_id.clone()),
                        confirmation_policy,
                    ));
                }
            }
        }
        Self::from_items(queue_id, items)
    }

    pub fn merge_cycles(queue_id: &str, cycles: &[AutonomousPaperCycle]) -> Self {
        let items = cycles
            .iter()
            .flat_map(|cycle| cycle.attention_items.clone())
            .collect();
        Self::from_items(queue_id, items)
    }

    pub fn sort_by_priority(&mut self) {
        self.items.sort_by(|left, right| {
            owner_attention_priority_rank(right.priority)
                .cmp(&owner_attention_priority_rank(left.priority))
                .then_with(|| left.item_id.cmp(&right.item_id))
        });
    }

    pub fn unresolved_items(&self) -> Vec<OwnerAttentionItem> {
        self.items
            .iter()
            .filter(|item| item.requires_owner_input)
            .cloned()
            .collect()
    }

    fn from_items(queue_id: &str, items: Vec<OwnerAttentionItem>) -> Self {
        let mut queue = Self {
            queue_id: queue_id.to_string(),
            high_priority_count: items
                .iter()
                .filter(|item| item.priority == OwnerAttentionPriority::High)
                .count(),
            requires_owner_input_count: items
                .iter()
                .filter(|item| item.requires_owner_input)
                .count(),
            items,
        };
        queue.sort_by_priority();
        queue
    }
}

fn reconsideration_symbol_for_decision(
    reconsideration: &OwnerFeedbackReconsiderationResult,
    decision: &ChairmanReconsiderationDecision,
) -> Option<String> {
    reconsideration
        .reconsideration_sessions
        .iter()
        .find(|session| session.session_id == decision.reconsideration_session_id)
        .and_then(|session| session.owner_feedback.symbol.clone())
}

fn reconsideration_scope_for_decision(
    reconsideration: &OwnerFeedbackReconsiderationResult,
    decision: &ChairmanReconsiderationDecision,
) -> Option<MarketScope> {
    reconsideration
        .reconsideration_sessions
        .iter()
        .find(|session| session.session_id == decision.reconsideration_session_id)
        .and_then(|session| session.owner_feedback.market_scope)
}

impl PaperDecisionArchive {
    pub fn append_cycle_decisions(&mut self, cycle_id: &str, result: &BatchCommitteeCycleResult) {
        for decision in &result.chairman_decisions {
            let Some(session) = result
                .committee_sessions
                .iter()
                .find(|session| session.session_id == decision.session_id)
            else {
                continue;
            };
            self.entries.push(PaperDecisionArchiveEntry {
                archive_id: format!("archive-{}-{}", cycle_id, decision.decision_id),
                cycle_id: cycle_id.to_string(),
                symbol: session.event.symbol.clone(),
                market_scope: session.event.market_scope,
                chairman_action: decision.final_action,
                risk_governor_status: decision.risk_governor_status,
                event_count: result
                    .events_by_symbol
                    .get(&session.event.symbol)
                    .copied()
                    .unwrap_or(1),
                deciding_members: decision.winning_arguments.clone(),
                dissenting_members: decision.dissenting_arguments.clone(),
                paper_only: true,
            });
        }
    }

    pub fn decisions_by_symbol(&self, symbol: &str) -> Vec<PaperDecisionArchiveEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.symbol == symbol)
            .cloned()
            .collect()
    }

    pub fn risk_veto_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.risk_governor_status == RiskGovernorStatus::Vetoed)
            .count()
    }

    pub fn need_more_evidence_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.chairman_action == ChairmanFinalAction::NeedMoreEvidence)
            .count()
    }
}

fn owner_attention_item(
    cycle_index: usize,
    item_index: usize,
    symbol: Option<String>,
    market_scope: Option<MarketScope>,
    attention_type: OwnerAttentionType,
    priority: OwnerAttentionPriority,
    reason: String,
    related_member_ids: Vec<String>,
    related_decision_id: Option<String>,
    confirmation_policy: OwnerConfirmationPolicy,
) -> OwnerAttentionItem {
    OwnerAttentionItem {
        item_id: format!("attention-{}-{}", cycle_index + 1, item_index + 1),
        symbol,
        market_scope,
        attention_type,
        priority,
        reason,
        related_member_ids,
        related_decision_id,
        requires_owner_input: owner_attention_requires_input(confirmation_policy, attention_type),
    }
}

fn owner_attention_requires_input(
    policy: OwnerConfirmationPolicy,
    attention_type: OwnerAttentionType,
) -> bool {
    match policy {
        OwnerConfirmationPolicy::Never => false,
        OwnerConfirmationPolicy::Always => true,
        OwnerConfirmationPolicy::OnlyForRiskWarnings => {
            attention_type == OwnerAttentionType::RiskVeto
        }
        OwnerConfirmationPolicy::OnlyForHighConfidenceEvents => {
            attention_type == OwnerAttentionType::HighConfidenceEntry
        }
    }
}

fn owner_attention_priority_rank(priority: OwnerAttentionPriority) -> u8 {
    match priority {
        OwnerAttentionPriority::Low => 0,
        OwnerAttentionPriority::Normal => 1,
        OwnerAttentionPriority::High => 2,
    }
}

fn symbols_from_batch_result(result: &BatchCommitteeCycleResult) -> Vec<String> {
    let mut symbols: Vec<String> = result
        .member_opinions
        .iter()
        .map(|opinion| opinion.symbol.clone())
        .collect();
    symbols.sort();
    symbols.dedup();
    symbols
}

fn market_scopes_from_batch_result(result: &BatchCommitteeCycleResult) -> Vec<MarketScope> {
    let mut scopes: Vec<MarketScope> = result
        .member_opinions
        .iter()
        .map(|opinion| opinion.market_scope)
        .collect();
    scopes.sort();
    scopes.dedup();
    scopes
}

pub fn run_owner_feedback_reconsideration_cycle(
    input: OwnerFeedbackReconsiderationInput,
) -> Result<OwnerFeedbackReconsiderationResult, String> {
    for feedback in &input.owner_feedback {
        validate_owner_feedback(feedback)?;
    }

    let mut routed_feedback_packets = Vec::new();
    let mut revised_member_opinions = Vec::new();
    let mut reconsideration_sessions = Vec::new();
    let mut chairman_reconsideration_decisions = Vec::new();
    let mut owner_feedback_journal_entries = Vec::new();
    let mut updated_owner_console_view = input.previous_owner_console_view.clone();

    for feedback in &input.owner_feedback {
        let routed_members = routed_members_for_feedback(feedback, &input.previous_batch_result);
        let feedback_packets: Vec<OwnerFeedbackPacket> = routed_members
            .iter()
            .map(|member_id| build_owner_feedback_packet(feedback, member_id, &input))
            .collect();
        let mut revised_for_feedback: Vec<MemberReconsiderationOpinion> = feedback_packets
            .iter()
            .filter_map(|packet| {
                packet
                    .related_previous_opinions
                    .first()
                    .map(|opinion| produce_reconsideration_opinion(feedback, opinion))
            })
            .collect();
        revised_for_feedback.sort_by(|left, right| left.member_id.cmp(&right.member_id));
        routed_feedback_packets.extend(feedback_packets);

        if opens_reconsideration(feedback.feedback_type) {
            let session = build_reconsideration_session(
                feedback,
                &input.previous_batch_result,
                &routed_members,
                &revised_for_feedback,
            );
            let decision = synthesize_reconsideration_decision(&session);
            append_reconsideration_to_console(&mut updated_owner_console_view, feedback, &decision);
            owner_feedback_journal_entries.push(OwnerFeedbackJournalEntry {
                feedback_id: feedback.feedback_id.clone(),
                symbol: feedback.symbol.clone(),
                routed_to_members: routed_members,
                reconsideration_opened: true,
                decision_id: Some(decision.decision_id.clone()),
                outcome: outcome_for_reconsideration(decision.final_action),
                note: "owner feedback routed through AI members; no order path".to_string(),
            });
            revised_member_opinions.extend(revised_for_feedback);
            reconsideration_sessions.push(session);
            chairman_reconsideration_decisions.push(decision);
        } else {
            let note = match feedback.feedback_type {
                OwnerFeedbackType::PaperOutcomeLabel => {
                    "paper outcome label recorded in owner feedback journal only; no real trading"
                }
                OwnerFeedbackType::Comment => {
                    "owner comment logged only; no committee reopened and no real trading"
                }
                _ => "owner feedback logged only; no committee reopened and no real trading",
            };
            owner_feedback_journal_entries.push(OwnerFeedbackJournalEntry {
                feedback_id: feedback.feedback_id.clone(),
                symbol: feedback.symbol.clone(),
                routed_to_members: routed_members,
                reconsideration_opened: false,
                decision_id: None,
                outcome: OwnerFeedbackOutcome::LoggedOnly,
                note: note.to_string(),
            });
        }
    }

    updated_owner_console_view.view_id =
        format!("{}-reconsidered", updated_owner_console_view.view_id);

    Ok(OwnerFeedbackReconsiderationResult {
        owner_feedback_count: input.owner_feedback.len(),
        routed_feedback_packets,
        revised_member_opinions,
        reconsideration_sessions,
        chairman_reconsideration_decisions,
        owner_feedback_journal_entries,
        updated_owner_console_view,
        paper_only_warning:
            "owner feedback is paper-only discussion; no broker/order/account or live execution"
                .to_string(),
    })
}

fn opens_reconsideration(feedback_type: OwnerFeedbackType) -> bool {
    matches!(
        feedback_type,
        OwnerFeedbackType::Disagree
            | OwnerFeedbackType::RiskConcern
            | OwnerFeedbackType::EvidenceRequest
            | OwnerFeedbackType::WatchlistRequest
            | OwnerFeedbackType::ReconsiderationRequest
    )
}

fn routed_members_for_feedback(
    feedback: &OwnerFeedback,
    batch_result: &BatchCommitteeCycleResult,
) -> Vec<String> {
    let mut members: Vec<String> = batch_result
        .member_opinions
        .iter()
        .filter(|opinion| {
            feedback
                .symbol
                .as_ref()
                .map_or(true, |symbol| &opinion.symbol == symbol)
                && feedback
                    .market_scope
                    .map_or(true, |scope| opinion.market_scope == scope)
        })
        .filter(|opinion| {
            if let Some(target) = &feedback.target_member_id {
                &opinion.member_id == target
            } else {
                match feedback.feedback_type {
                    OwnerFeedbackType::RiskConcern => opinion.member_id.contains("risk"),
                    OwnerFeedbackType::EvidenceRequest => opinion.member_id.contains("evidence"),
                    OwnerFeedbackType::Disagree
                    | OwnerFeedbackType::WatchlistRequest
                    | OwnerFeedbackType::ReconsiderationRequest => true,
                    OwnerFeedbackType::Comment | OwnerFeedbackType::PaperOutcomeLabel => false,
                }
            }
        })
        .map(|opinion| opinion.member_id.clone())
        .collect();
    members.sort();
    members.dedup();
    members
}

fn build_owner_feedback_packet(
    feedback: &OwnerFeedback,
    member_id: &str,
    input: &OwnerFeedbackReconsiderationInput,
) -> OwnerFeedbackPacket {
    let related_previous_opinions: Vec<MemberOpinion> = input
        .previous_batch_result
        .member_opinions
        .iter()
        .filter(|opinion| {
            opinion.member_id == member_id
                && feedback
                    .symbol
                    .as_ref()
                    .map_or(true, |symbol| &opinion.symbol == symbol)
                && feedback
                    .market_scope
                    .map_or(true, |scope| opinion.market_scope == scope)
        })
        .cloned()
        .collect();
    let related_market_data = input
        .market_data
        .iter()
        .find(|market| {
            feedback
                .symbol
                .as_ref()
                .map_or(false, |symbol| &market.symbol == symbol)
                && feedback
                    .market_scope
                    .map_or(true, |scope| market.market_scope == scope)
        })
        .cloned();
    let related_news: Vec<NewsSnapshot> = input
        .news
        .iter()
        .filter(|news| {
            feedback
                .symbol
                .as_ref()
                .map_or(false, |symbol| &news.symbol == symbol)
        })
        .cloned()
        .collect();
    let related_chairman_decision = input
        .previous_batch_result
        .chairman_decisions
        .iter()
        .find(|decision| {
            input
                .previous_batch_result
                .committee_sessions
                .iter()
                .find(|session| session.session_id == decision.session_id)
                .map_or(false, |session| {
                    feedback
                        .symbol
                        .as_ref()
                        .map_or(true, |symbol| &session.event.symbol == symbol)
                        && feedback
                            .market_scope
                            .map_or(true, |scope| session.event.market_scope == scope)
                })
        })
        .cloned();
    OwnerFeedbackPacket {
        feedback: feedback.clone(),
        related_market_data,
        related_news,
        related_previous_opinions,
        related_risk_status: related_chairman_decision
            .as_ref()
            .map(|decision| decision.risk_governor_status),
        related_chairman_decision,
    }
}

fn produce_reconsideration_opinion(
    feedback: &OwnerFeedback,
    previous: &MemberOpinion,
) -> MemberReconsiderationOpinion {
    let mut revised_stance = previous.stance;
    let mut confidence_after = previous.confidence;
    let mut evidence_needed = Vec::new();
    let mut risk_notes = Vec::new();
    let mut reason = "owner feedback reviewed; previous stance kept".to_string();

    match feedback.feedback_type {
        OwnerFeedbackType::RiskConcern if previous.member_id.contains("risk") => {
            revised_stance = MemberStance::NoTrade;
            confidence_after = clamp_unit(previous.confidence + 0.1);
            risk_notes.push("owner risk concern raised defensive confidence".to_string());
            reason = "RiskGuardAI incorporated owner risk concern".to_string();
        }
        OwnerFeedbackType::RiskConcern if previous.member_id.contains("trend") => {
            confidence_after = clamp_unit(previous.confidence - 0.08);
            risk_notes.push("owner risk concern reduced entry confidence".to_string());
            reason = "TrendEntryAI lowered confidence after owner risk concern".to_string();
        }
        OwnerFeedbackType::EvidenceRequest if previous.member_id.contains("evidence") => {
            revised_stance = MemberStance::NeedMoreEvidence;
            confidence_after = clamp_unit(previous.confidence + 0.08);
            evidence_needed.push("owner requested additional evidence".to_string());
            reason = "EvidenceRegimeAI requested more evidence".to_string();
        }
        OwnerFeedbackType::Disagree => {
            if feedback
                .target_member_id
                .as_ref()
                .map_or(false, |target| target == &previous.member_id)
            {
                revised_stance = if matches!(previous.stance, MemberStance::BuyProposal) {
                    MemberStance::Hold
                } else {
                    previous.stance
                };
                confidence_after = clamp_unit(previous.confidence - 0.05);
                reason = "targeted member reconsidered after owner disagreement".to_string();
            }
        }
        OwnerFeedbackType::WatchlistRequest => {
            revised_stance = MemberStance::Hold;
            reason = "owner requested paper watchlist only".to_string();
        }
        OwnerFeedbackType::ReconsiderationRequest => {
            reason = "owner requested committee reconsideration".to_string();
        }
        OwnerFeedbackType::Comment | OwnerFeedbackType::PaperOutcomeLabel => {}
        _ => {}
    }

    MemberReconsiderationOpinion {
        member_id: previous.member_id.clone(),
        symbol: previous.symbol.clone(),
        market_scope: previous.market_scope,
        previous_stance: previous.stance,
        revised_stance,
        confidence_before: previous.confidence,
        confidence_after,
        changed: revised_stance != previous.stance
            || (confidence_after - previous.confidence).abs() > f64::EPSILON,
        reason,
        evidence_needed,
        risk_notes,
        event_triggered: true,
    }
}

fn build_reconsideration_session(
    feedback: &OwnerFeedback,
    batch_result: &BatchCommitteeCycleResult,
    invited_members: &[String],
    revised_opinions: &[MemberReconsiderationOpinion],
) -> CommitteeReconsiderationSession {
    let original_session_id = batch_result
        .committee_sessions
        .iter()
        .find(|session| {
            feedback
                .symbol
                .as_ref()
                .map_or(true, |symbol| &session.event.symbol == symbol)
                && feedback
                    .market_scope
                    .map_or(true, |scope| session.event.market_scope == scope)
        })
        .map(|session| session.session_id.clone());
    let defensive_count = revised_opinions
        .iter()
        .filter(|opinion| {
            matches!(
                opinion.revised_stance,
                MemberStance::NoTrade | MemberStance::NeedMoreEvidence
            )
        })
        .count();
    let total = revised_opinions.len().max(1) as f64;
    let mut risk_flags = Vec::new();
    if feedback.feedback_type == OwnerFeedbackType::RiskConcern {
        risk_flags.push("owner_risk_concern".to_string());
    }
    if revised_opinions
        .iter()
        .any(|opinion| matches!(opinion.revised_stance, MemberStance::NoTrade))
    {
        risk_flags.push("member_no_trade_warning".to_string());
    }
    CommitteeReconsiderationSession {
        session_id: format!("reconsideration-{}", feedback.feedback_id),
        original_session_id,
        owner_feedback: feedback.clone(),
        invited_members: invited_members.to_vec(),
        revised_opinions: revised_opinions.to_vec(),
        disagreement_level: defensive_count as f64 / total,
        risk_flags,
    }
}

fn synthesize_reconsideration_decision(
    session: &CommitteeReconsiderationSession,
) -> ChairmanReconsiderationDecision {
    let risk_vetoed = session.risk_flags.iter().any(|flag| flag.contains("risk"))
        && session
            .revised_opinions
            .iter()
            .any(|opinion| matches!(opinion.revised_stance, MemberStance::NoTrade));
    let any_changed = session
        .revised_opinions
        .iter()
        .any(|opinion| opinion.changed);
    let any_need_evidence = session
        .revised_opinions
        .iter()
        .any(|opinion| matches!(opinion.revised_stance, MemberStance::NeedMoreEvidence));
    let final_action = if risk_vetoed {
        ChairmanReconsiderationFinalAction::RiskVetoed
    } else if session.owner_feedback.feedback_type == OwnerFeedbackType::WatchlistRequest {
        ChairmanReconsiderationFinalAction::PaperHold
    } else if any_need_evidence {
        ChairmanReconsiderationFinalAction::NeedMoreEvidence
    } else if any_changed {
        ChairmanReconsiderationFinalAction::PaperHold
    } else {
        ChairmanReconsiderationFinalAction::KeepPreviousDecision
    };
    let risk_governor_status = if risk_vetoed {
        RiskGovernorStatus::Vetoed
    } else if any_need_evidence {
        RiskGovernorStatus::NeedsReview
    } else {
        RiskGovernorStatus::Passed
    };
    ChairmanReconsiderationDecision {
        decision_id: format!("decision-{}", session.session_id),
        reconsideration_session_id: session.session_id.clone(),
        final_action,
        rationale: match final_action {
            ChairmanReconsiderationFinalAction::RiskVetoed => {
                "Risk Governor still vetoes after owner feedback reconsideration".to_string()
            }
            ChairmanReconsiderationFinalAction::PaperHold => {
                "Owner feedback moved discussion to paper hold/watchlist".to_string()
            }
            ChairmanReconsiderationFinalAction::NeedMoreEvidence => {
                "Committee needs more evidence after owner feedback".to_string()
            }
            ChairmanReconsiderationFinalAction::KeepPreviousDecision => {
                "Owner feedback reviewed; previous paper decision remains".to_string()
            }
            ChairmanReconsiderationFinalAction::PaperBuy
            | ChairmanReconsiderationFinalAction::PaperSell
            | ChairmanReconsiderationFinalAction::PaperNoTrade => {
                "Chairman synthesized reconsidered member opinions".to_string()
            }
        },
        what_changed: session
            .revised_opinions
            .iter()
            .filter(|opinion| opinion.changed)
            .map(|opinion| format!("{}:{:?}", opinion.member_id, opinion.revised_stance))
            .collect(),
        what_did_not_change: session
            .revised_opinions
            .iter()
            .filter(|opinion| !opinion.changed)
            .map(|opinion| format!("{}:{:?}", opinion.member_id, opinion.revised_stance))
            .collect(),
        risk_governor_status,
    }
}

fn append_reconsideration_to_console(
    console: &mut OwnerCommitteeConsoleView,
    feedback: &OwnerFeedback,
    decision: &ChairmanReconsiderationDecision,
) {
    console.next_action_rows.push(NextActionRow {
        symbol: feedback.symbol.clone(),
        action_type: next_action_for_reconsideration(decision.final_action),
        note: decision.rationale.clone(),
    });
    if decision.risk_governor_status == RiskGovernorStatus::Vetoed {
        console.risk_veto_rows.push(RiskVetoRow {
            symbol: feedback
                .symbol
                .clone()
                .unwrap_or_else(|| "owner-feedback".to_string()),
            reason: decision.rationale.clone(),
            blocked_action: "owner reconsideration".to_string(),
            risk_member_support: decision
                .what_changed
                .iter()
                .find(|item| item.contains("risk"))
                .cloned(),
        });
    }
}

fn next_action_for_reconsideration(action: ChairmanReconsiderationFinalAction) -> NextActionType {
    match action {
        ChairmanReconsiderationFinalAction::RiskVetoed => NextActionType::RiskBlocked,
        ChairmanReconsiderationFinalAction::NeedMoreEvidence => NextActionType::NeedMoreEvidence,
        ChairmanReconsiderationFinalAction::PaperHold => NextActionType::Watch,
        ChairmanReconsiderationFinalAction::PaperNoTrade => NextActionType::NoTrade,
        ChairmanReconsiderationFinalAction::PaperBuy
        | ChairmanReconsiderationFinalAction::PaperSell
        | ChairmanReconsiderationFinalAction::KeepPreviousDecision => NextActionType::PaperReview,
    }
}

fn outcome_for_reconsideration(action: ChairmanReconsiderationFinalAction) -> OwnerFeedbackOutcome {
    match action {
        ChairmanReconsiderationFinalAction::RiskVetoed => OwnerFeedbackOutcome::RiskBlocked,
        ChairmanReconsiderationFinalAction::NeedMoreEvidence => {
            OwnerFeedbackOutcome::NeedMoreEvidence
        }
        ChairmanReconsiderationFinalAction::KeepPreviousDecision => {
            OwnerFeedbackOutcome::KeptDecision
        }
        ChairmanReconsiderationFinalAction::PaperBuy
        | ChairmanReconsiderationFinalAction::PaperSell
        | ChairmanReconsiderationFinalAction::PaperHold
        | ChairmanReconsiderationFinalAction::PaperNoTrade => OwnerFeedbackOutcome::ChangedDecision,
    }
}

fn build_owner_committee_summary(result: &BatchCommitteeCycleResult) -> OwnerCommitteeSummary {
    let mut symbols_reviewed: Vec<String> = result
        .member_opinions
        .iter()
        .map(|opinion| opinion.symbol.clone())
        .collect();
    symbols_reviewed.sort();
    symbols_reviewed.dedup();
    let mut top_supporting_members: Vec<String> = result
        .chairman_decisions
        .iter()
        .flat_map(|decision| decision.winning_arguments.clone())
        .collect();
    top_supporting_members.sort();
    top_supporting_members.dedup();
    let mut top_dissenting_members: Vec<String> = result
        .chairman_decisions
        .iter()
        .flat_map(|decision| decision.dissenting_arguments.clone())
        .collect();
    top_dissenting_members.sort();
    top_dissenting_members.dedup();
    let mut risk_warnings: Vec<String> = result
        .committee_sessions
        .iter()
        .flat_map(|session| {
            session
                .risk_flags
                .iter()
                .map(|flag| format!("{}:{}", session.event.symbol, flag))
        })
        .collect();
    risk_warnings.sort();
    risk_warnings.dedup();
    let member_voice_changes: Vec<MemberVoiceChange> = result
        .score_updates
        .iter()
        .filter(|update| {
            (update.new_voice_weight - update.previous_voice_weight).abs() > f64::EPSILON
        })
        .map(|update| MemberVoiceChange {
            member_id: update.member_id.clone(),
            previous_voice_weight: update.previous_voice_weight,
            new_voice_weight: update.new_voice_weight,
            direction: if update.new_voice_weight > update.previous_voice_weight {
                "up".to_string()
            } else {
                "down".to_string()
            },
        })
        .collect();
    let chairman_actions: Vec<ChairmanFinalAction> = result
        .chairman_decisions
        .iter()
        .map(|decision| decision.final_action)
        .collect();
    let paper_buy_count = chairman_actions
        .iter()
        .filter(|action| matches!(action, ChairmanFinalAction::PaperBuy))
        .count();
    let paper_hold_count = chairman_actions
        .iter()
        .filter(|action| matches!(action, ChairmanFinalAction::PaperHold))
        .count();
    let no_trade_count = chairman_actions
        .iter()
        .filter(|action| {
            matches!(
                action,
                ChairmanFinalAction::PaperNoTrade | ChairmanFinalAction::RiskVetoed
            )
        })
        .count();
    let need_more_evidence_count = chairman_actions
        .iter()
        .filter(|action| matches!(action, ChairmanFinalAction::NeedMoreEvidence))
        .count();
    let mut event_triggers: Vec<String> = result
        .event_queue
        .events
        .iter()
        .map(|event| {
            format!(
                "{}:{:?}:{}",
                event.symbol, event.event_type, event.proposed_by_member_id
            )
        })
        .collect();
    event_triggers.sort();
    let mut voice_up_members: Vec<String> = member_voice_changes
        .iter()
        .filter(|change| change.direction == "up")
        .map(|change| change.member_id.clone())
        .collect();
    voice_up_members.sort();
    voice_up_members.dedup();
    let mut voice_down_members: Vec<String> = member_voice_changes
        .iter()
        .filter(|change| change.direction == "down")
        .map(|change| change.member_id.clone())
        .collect();
    voice_down_members.sort();
    voice_down_members.dedup();
    let action_summary: Vec<String> = chairman_actions
        .iter()
        .map(|action| format!("{:?}", action))
        .collect();
    let next_watch = if risk_warnings.is_empty() {
        format!("need_more_evidence_count={need_more_evidence_count}")
    } else {
        format!("risk_warnings={}", risk_warnings.join(","))
    };
    OwnerCommitteeSummary {
        cycle_id: result.batch_id.clone(),
        symbols_reviewed: symbols_reviewed.clone(),
        event_count: result.event_queue.event_count,
        risk_veto_count: result.risk_veto_count,
        paper_buy_count,
        paper_hold_count,
        no_trade_count,
        need_more_evidence_count,
        top_supporting_members,
        top_dissenting_members,
        member_voice_changes,
        chairman_actions,
        risk_warnings,
        owner_readable_summary: format!(
            "검토 종목: {}. 이벤트 {}개: {}. 의장 결정: {}. Risk Governor veto {}개. 발언권 상승: {}. 발언권 하락: {}. 다음 확인: {}. AI member opinions only; paper-only summary.",
            symbols_reviewed.join(","),
            result.event_queue.event_count,
            event_triggers.join(","),
            action_summary.join(","),
            result.risk_veto_count,
            voice_up_members.join(","),
            voice_down_members.join(","),
            next_watch
        ),
        paper_only_warning:
            "paper-only explanation; not an order, not investment advice, no broker/account path"
                .to_string(),
    }
}

pub fn build_owner_committee_console_view(
    batch_result: &BatchCommitteeCycleResult,
    state_update: &BatchCycleStateUpdate,
    owner_summary: Option<&OwnerCommitteeSummary>,
) -> OwnerCommitteeConsoleView {
    let mut reviewed_symbols: Vec<String> = batch_result
        .member_opinions
        .iter()
        .map(|opinion| opinion.symbol.clone())
        .collect();
    reviewed_symbols.sort();
    reviewed_symbols.dedup();
    let active_members: Vec<String> = state_update
        .updated_member_states
        .iter()
        .map(|state| state.member_id.clone())
        .collect();
    let member_status_rows = state_update
        .updated_member_states
        .iter()
        .map(|state| {
            let last_opinion = batch_result
                .member_opinions
                .iter()
                .rev()
                .find(|opinion| opinion.member_id == state.member_id);
            MemberStatusRow {
                member_id: state.member_id.clone(),
                display_name: display_name_for_member_id(&state.member_id).to_string(),
                role: role_for_member_id(&state.member_id),
                score: state.score,
                voice_weight: state.voice_weight,
                status: state.status,
                runtime_status: "OfflineFixture".to_string(),
                style_summary_short: style_summary_for_member_id(&state.member_id).to_string(),
                last_opinion_summary: last_opinion
                    .map(|opinion| {
                        format!(
                            "{}:{:?}:confidence={:.2}",
                            opinion.symbol, opinion.stance, opinion.confidence
                        )
                    })
                    .unwrap_or_else(|| "no opinion in latest batch".to_string()),
            }
        })
        .collect();
    let event_rows: Vec<EventRow> = batch_result
        .event_queue
        .events
        .iter()
        .map(|event| EventRow {
            event_id: event.event_id.clone(),
            symbol: event.symbol.clone(),
            market_scope: event.market_scope,
            proposed_by_member_id: event.proposed_by_member_id.clone(),
            event_type: event.event_type,
            confidence: event.triggering_opinion.confidence,
            reason: event
                .triggering_opinion
                .event_reason
                .clone()
                .unwrap_or_else(|| "member triggered event".to_string()),
        })
        .collect();
    let committee_rows: Vec<CommitteeRow> = batch_result
        .committee_sessions
        .iter()
        .map(|session| CommitteeRow {
            session_id: session.session_id.clone(),
            symbol: session.event.symbol.clone(),
            market_scope: session.event.market_scope,
            invited_members: session.invited_members.clone(),
            disagreement_level: session.disagreement_level,
            risk_flags: session.risk_flags.clone(),
        })
        .collect();
    let chairman_decision_rows: Vec<ChairmanDecisionRow> = batch_result
        .chairman_decisions
        .iter()
        .map(|decision| {
            let symbol = batch_result
                .committee_sessions
                .iter()
                .find(|session| session.session_id == decision.session_id)
                .map(|session| session.event.symbol.clone())
                .unwrap_or_else(|| "unknown".to_string());
            ChairmanDecisionRow {
                decision_id: decision.decision_id.clone(),
                symbol,
                final_action: decision.final_action,
                rationale_short: decision.rationale.clone(),
                risk_governor_status: decision.risk_governor_status,
            }
        })
        .collect();
    let risk_veto_rows: Vec<RiskVetoRow> = batch_result
        .chairman_decisions
        .iter()
        .filter(|decision| decision.risk_governor_status == RiskGovernorStatus::Vetoed)
        .filter_map(|decision| {
            let session = batch_result
                .committee_sessions
                .iter()
                .find(|session| session.session_id == decision.session_id)?;
            Some(RiskVetoRow {
                symbol: session.event.symbol.clone(),
                reason: decision.rationale.clone(),
                blocked_action: format!("{:?}", session.event.event_type),
                risk_member_support: decision
                    .winning_arguments
                    .iter()
                    .find(|argument| argument.contains("risk"))
                    .cloned(),
            })
        })
        .collect();
    let voice_change_rows: Vec<VoiceChangeRow> = state_update
        .score_updates
        .iter()
        .filter(|update| {
            (update.new_voice_weight - update.previous_voice_weight).abs() > f64::EPSILON
        })
        .map(|update| VoiceChangeRow {
            member_id: update.member_id.clone(),
            previous_voice_weight: update.previous_voice_weight,
            new_voice_weight: update.new_voice_weight,
            reason: update.update_reason,
        })
        .collect();
    let next_action_rows = build_next_action_rows(batch_result, &reviewed_symbols);
    OwnerCommitteeConsoleView {
        view_id: format!("owner-console-{}", batch_result.batch_id),
        cycle_id: batch_result.batch_id.clone(),
        reviewed_symbols,
        active_members,
        member_status_rows,
        event_rows,
        committee_rows,
        chairman_decision_rows,
        risk_veto_rows,
        voice_change_rows,
        next_action_rows,
        paper_only_warning: owner_summary
            .map(|summary| summary.paper_only_warning.clone())
            .unwrap_or_else(|| {
                "paper-only display; not investment advice; no live execution".to_string()
            }),
    }
}

fn build_next_action_rows(
    batch_result: &BatchCommitteeCycleResult,
    reviewed_symbols: &[String],
) -> Vec<NextActionRow> {
    let mut rows = Vec::new();
    let event_symbols: std::collections::BTreeSet<String> = batch_result
        .event_queue
        .events
        .iter()
        .map(|event| event.symbol.clone())
        .collect();
    for decision in &batch_result.chairman_decisions {
        let symbol = batch_result
            .committee_sessions
            .iter()
            .find(|session| session.session_id == decision.session_id)
            .map(|session| session.event.symbol.clone());
        let action_type = match decision.final_action {
            ChairmanFinalAction::RiskVetoed => NextActionType::RiskBlocked,
            ChairmanFinalAction::NeedMoreEvidence => NextActionType::NeedMoreEvidence,
            ChairmanFinalAction::PaperNoTrade => NextActionType::NoTrade,
            ChairmanFinalAction::PaperBuy
            | ChairmanFinalAction::PaperSell
            | ChairmanFinalAction::PaperHold => NextActionType::PaperReview,
        };
        rows.push(NextActionRow {
            symbol,
            action_type,
            note: decision.rationale.clone(),
        });
    }
    for event in &batch_result.event_queue.events {
        if event.event_type == InvestmentEventType::NeedMoreEvidence {
            rows.push(NextActionRow {
                symbol: Some(event.symbol.clone()),
                action_type: NextActionType::NeedMoreEvidence,
                note: event
                    .triggering_opinion
                    .event_reason
                    .clone()
                    .unwrap_or_else(|| "member requested more evidence".to_string()),
            });
        }
    }
    for symbol in reviewed_symbols {
        if !event_symbols.contains(symbol) {
            rows.push(NextActionRow {
                symbol: Some(symbol.clone()),
                action_type: NextActionType::Watch,
                note: "no triggered event; keep on paper watchlist".to_string(),
            });
        }
    }
    rows
}

fn role_for_member_id(member_id: &str) -> Option<IndependentMemberRole> {
    if member_id.contains("trend") {
        Some(IndependentMemberRole::TrendEntry)
    } else if member_id.contains("risk") {
        Some(IndependentMemberRole::RiskGuard)
    } else if member_id.contains("evidence") {
        Some(IndependentMemberRole::EvidenceRegime)
    } else {
        None
    }
}

fn display_name_for_member_id(member_id: &str) -> &str {
    if member_id.contains("trend") {
        "TrendEntryAI"
    } else if member_id.contains("risk") {
        "RiskGuardAI"
    } else if member_id.contains("evidence") {
        "EvidenceRegimeAI"
    } else {
        member_id
    }
}

fn style_summary_for_member_id(member_id: &str) -> &str {
    if member_id.contains("trend") {
        "trend entry member"
    } else if member_id.contains("risk") {
        "risk guard member"
    } else if member_id.contains("evidence") {
        "evidence regime member"
    } else {
        "committee member"
    }
}

fn no_committee_result(
    market_data: MarketDataSnapshot,
    selected_scope: MarketScope,
    activation_plan: MemberActivationPlan,
) -> MinimalCommitteeCycleResult {
    MinimalCommitteeCycleResult {
        selected_scope,
        symbol: market_data.symbol,
        routed_packet_count: 0,
        triggered_event_count: 0,
        committee_session_count: 0,
        distributed_member_ids: Vec::new(),
        member_opinions: Vec::new(),
        event: None,
        committee_session: None,
        chairman_decision: None,
        score_updates: Vec::new(),
        activation_plan,
        member_roles: Vec::new(),
        memory_states: Vec::new(),
        learning_journal_entries: Vec::new(),
        learning_journal_entry_count: 0,
        learning_journals: Vec::new(),
        style_card_registry: None,
        three_member_style_mapping: None,
        style_influenced_profiles: Vec::new(),
        safety_summary: safety_summary(),
    }
}

fn apply_style_influence_to_opinion(
    mut opinion: MemberOpinion,
    profile: &StyleInfluencedMemberProfile,
) -> MemberOpinion {
    let blend_weight = profile
        .style_blend
        .archetype_weights
        .iter()
        .map(|weight| weight.weight)
        .sum::<f64>()
        .min(1.0);
    match profile.base_role {
        IndependentMemberRole::TrendEntry
            if matches!(opinion.stance, MemberStance::BuyProposal) =>
        {
            opinion.confidence = clamp_unit(opinion.confidence + 0.02 * blend_weight);
            opinion.evidence_notes.push(
                "style influence: trend/momentum archetype cards lightly support entry proposal"
                    .to_string(),
            );
        }
        IndependentMemberRole::RiskGuard
            if matches!(
                opinion.stance,
                MemberStance::NoTrade | MemberStance::NeedMoreEvidence
            ) =>
        {
            opinion.confidence = clamp_unit(opinion.confidence + 0.02 * blend_weight);
            opinion.risk_hint = clamp_unit(opinion.risk_hint + 0.02 * blend_weight);
            opinion.evidence_notes.push(
                "style influence: risk/volatility archetype cards strengthen defensive view"
                    .to_string(),
            );
        }
        IndependentMemberRole::EvidenceRegime => {
            if matches!(
                opinion.stance,
                MemberStance::Hold | MemberStance::NeedMoreEvidence
            ) {
                opinion.confidence = clamp_unit(opinion.confidence + 0.01 * blend_weight);
            }
            opinion.evidence_notes.push(
                "style influence: evidence-quality archetype cards affect evidence threshold only"
                    .to_string(),
            );
        }
        _ => {}
    }
    opinion
        .evidence_notes
        .push("archetype style card influence only; not a real investor clone".to_string());
    opinion
}

fn deferred_core_opinion(packet: &MemberInputPacket, reason: &str) -> MemberOpinion {
    MemberOpinion {
        member_id: packet.member_id.clone(),
        symbol: packet.market_data.symbol.clone(),
        market_scope: packet.market_data.market_scope,
        stance: MemberStance::NeedMoreEvidence,
        confidence: 0.5,
        expected_return_hint: 0.0,
        risk_hint: clamp_unit(packet.market_data.volatility_hint),
        evidence_notes: vec![
            format!("Mamba3 + Gated DeltaNet core {}", reason),
            "NeedMoreEvidence only; no fake AI inference, training, or order".to_string(),
        ],
        event_triggered: false,
        event_reason: Some(reason.to_string()),
    }
}

fn outcome_from_return(simulated_outcome_return: Option<f64>) -> SimulatedPaperOutcome {
    match simulated_outcome_return {
        Some(value) if value > 0.0 => SimulatedPaperOutcome::Positive,
        Some(value) if value < 0.0 => SimulatedPaperOutcome::Negative,
        Some(_) => SimulatedPaperOutcome::Neutral,
        None => SimulatedPaperOutcome::Unknown,
    }
}

pub fn apply_paper_outcome_feedback(
    members: &[AICommitteeMember],
    member_opinions: &[MemberOpinion],
    chairman_decision: Option<&ChairmanDecision>,
    paper_outcome_feedback: &PaperOutcomeFeedback,
) -> PaperOutcomeFeedbackResult {
    let score_updates: Vec<MemberScoreUpdate> = members
        .iter()
        .filter_map(|member| {
            let opinion = member_opinions
                .iter()
                .find(|opinion| opinion.member_id == member.member_id)?;
            Some(score_update_for(
                member,
                opinion,
                chairman_decision,
                outcome_return_for_feedback(paper_outcome_feedback.simulated_result),
            ))
        })
        .collect();

    let mut learning_journal_entries = Vec::new();
    let mut updated_memory_states = Vec::new();
    for member in members {
        let Some(opinion) = member_opinions
            .iter()
            .find(|opinion| opinion.member_id == member.member_id)
        else {
            continue;
        };
        let update_reason = score_updates
            .iter()
            .find(|update| update.member_id == member.member_id)
            .map(|update| update.update_reason)
            .unwrap_or(MemberScoreUpdateReason::Neutral);
        let learning_signal =
            learning_signal_for(update_reason, paper_outcome_feedback.simulated_result);
        let decision_id = chairman_decision
            .map(|decision| decision.decision_id.clone())
            .unwrap_or_else(|| paper_outcome_feedback.decision_id.clone());
        learning_journal_entries.push(MemberLearningJournalEntry {
            journal_id: format!(
                "journal-{}-{}-{}",
                member.member_id, paper_outcome_feedback.symbol, decision_id
            ),
            member_id: member.member_id.clone(),
            symbol: paper_outcome_feedback.symbol.clone(),
            market_scope: paper_outcome_feedback.market_scope,
            opinion_stance: opinion.stance,
            confidence: opinion.confidence,
            chairman_action: chairman_decision.map(|decision| decision.final_action),
            risk_governor_status: chairman_decision.map(|decision| decision.risk_governor_status),
            simulated_outcome: paper_outcome_feedback.simulated_result,
            learning_signal,
            note: "offline learning journal only; no model training or weight mutation".to_string(),
        });

        let mut memory_state = member
            .memory_state
            .clone()
            .unwrap_or_else(|| MemberMemoryState::new(&member.member_id));
        if !memory_state
            .recent_symbols
            .contains(&paper_outcome_feedback.symbol)
        {
            memory_state
                .recent_symbols
                .push(paper_outcome_feedback.symbol.clone());
        }
        memory_state.recent_opinion_count += 1;
        if opinion.event_triggered {
            memory_state.recent_event_count += 1;
        }
        match update_reason {
            MemberScoreUpdateReason::GoodCall | MemberScoreUpdateReason::HelpfulDissent => {
                memory_state.recent_good_call_count += 1;
            }
            MemberScoreUpdateReason::BadCall
            | MemberScoreUpdateReason::RiskyCall
            | MemberScoreUpdateReason::LowEvidence => {
                memory_state.recent_bad_call_count += 1;
            }
            MemberScoreUpdateReason::Neutral => {}
        }
        if chairman_decision
            .is_some_and(|decision| decision.risk_governor_status == RiskGovernorStatus::Vetoed)
        {
            memory_state.recent_risk_veto_count += 1;
        }
        memory_state
            .notes
            .push(format!("last_learning_signal={:?}", learning_signal));
        updated_memory_states.push(memory_state);
    }

    PaperOutcomeFeedbackResult {
        score_updates,
        learning_journal_entries,
        updated_memory_states,
    }
}

fn outcome_return_for_feedback(outcome: SimulatedPaperOutcome) -> Option<f64> {
    match outcome {
        SimulatedPaperOutcome::Positive => Some(0.01),
        SimulatedPaperOutcome::Negative => Some(-0.01),
        SimulatedPaperOutcome::Neutral => Some(0.0),
        SimulatedPaperOutcome::Unknown => None,
    }
}

fn learning_signal_for(
    update_reason: MemberScoreUpdateReason,
    outcome: SimulatedPaperOutcome,
) -> MemberLearningSignal {
    match update_reason {
        MemberScoreUpdateReason::GoodCall | MemberScoreUpdateReason::HelpfulDissent => {
            MemberLearningSignal::Reinforce
        }
        MemberScoreUpdateReason::BadCall
        | MemberScoreUpdateReason::RiskyCall
        | MemberScoreUpdateReason::LowEvidence => MemberLearningSignal::Penalize,
        MemberScoreUpdateReason::Neutral if matches!(outcome, SimulatedPaperOutcome::Unknown) => {
            MemberLearningSignal::Ignore
        }
        MemberScoreUpdateReason::Neutral => MemberLearningSignal::Watch,
    }
}

fn build_learning_journals(entries: &[MemberLearningJournalEntry]) -> Vec<MemberLearningJournal> {
    let mut member_ids: Vec<String> = entries
        .iter()
        .map(|entry| entry.member_id.clone())
        .collect();
    member_ids.sort();
    member_ids.dedup();
    member_ids
        .into_iter()
        .map(|member_id| {
            let entries: Vec<MemberLearningJournalEntry> = entries
                .iter()
                .filter(|entry| entry.member_id == member_id)
                .cloned()
                .collect();
            let summary = MemberLearningJournalSummary {
                reinforce_count: entries
                    .iter()
                    .filter(|entry| {
                        matches!(entry.learning_signal, MemberLearningSignal::Reinforce)
                    })
                    .count(),
                penalize_count: entries
                    .iter()
                    .filter(|entry| matches!(entry.learning_signal, MemberLearningSignal::Penalize))
                    .count(),
                watch_count: entries
                    .iter()
                    .filter(|entry| matches!(entry.learning_signal, MemberLearningSignal::Watch))
                    .count(),
                ignore_count: entries
                    .iter()
                    .filter(|entry| matches!(entry.learning_signal, MemberLearningSignal::Ignore))
                    .count(),
            };
            MemberLearningJournal {
                member_id,
                entries,
                summary,
            }
        })
        .collect()
}

fn produce_mock_opinion(member: &AICommitteeMember, packet: &MemberInputPacket) -> MemberOpinion {
    let market = &packet.market_data;
    let sentiment = combined_sentiment(&packet.news);
    let style = member.style_profile.to_ascii_lowercase();
    let mut stance = MemberStance::Hold;
    let mut confidence: f64 = 0.52;
    let mut expected_return_hint = market.change_pct / 100.0;
    let mut risk_hint = clamp_unit(market.volatility_hint);
    let mut notes = vec![
        format!("style={}", member.style_profile),
        "deterministic mock opinion; not model inference".to_string(),
    ];
    let mut event_triggered = false;
    let mut event_reason = None;

    if style.contains("risk") && market.volatility_hint >= 0.08 {
        stance = MemberStance::NoTrade;
        confidence = 0.82;
        expected_return_hint = 0.0;
        risk_hint = clamp_unit(market.volatility_hint + 0.2);
        event_triggered = true;
        event_reason = Some("high volatility risk warning".to_string());
        notes.push("risk-style member raised volatility warning".to_string());
    } else if style.contains("evidence") && sentiment == "unknown" {
        stance = MemberStance::NeedMoreEvidence;
        confidence = 0.7;
        expected_return_hint = 0.0;
        risk_hint = clamp_unit(market.volatility_hint + 0.1);
        event_triggered = true;
        event_reason = Some("news sentiment unknown".to_string());
        notes.push("evidence-style member requested more evidence".to_string());
    } else if style.contains("trend") && market.change_pct > 3.0 && sentiment == "positive" {
        stance = MemberStance::BuyProposal;
        confidence = 0.78;
        expected_return_hint = 0.035;
        risk_hint = clamp_unit(market.volatility_hint);
        event_triggered = true;
        event_reason = Some("positive momentum and news".to_string());
        notes.push("trend-style member proposed paper entry".to_string());
    } else if style.contains("value")
        && matches!(
            market.market_scope,
            MarketScope::KoreaLongTerm | MarketScope::UsLongTerm
        )
        && market.volatility_hint <= 0.04
        && sentiment != "negative"
    {
        stance = MemberStance::BuyProposal;
        confidence = 0.66;
        expected_return_hint = 0.02;
        risk_hint = clamp_unit(market.volatility_hint);
        event_triggered = true;
        event_reason = Some("long-term low-volatility setup".to_string());
        notes.push("value-style member proposed paper entry".to_string());
    } else if style.contains("liquidity")
        && matches!(
            market.market_scope,
            MarketScope::CryptoShortTerm | MarketScope::CryptoLongTerm
        )
        && market.volatility_hint >= 0.07
    {
        stance = MemberStance::NoTrade;
        confidence = 0.74;
        expected_return_hint = 0.0;
        risk_hint = clamp_unit(market.volatility_hint + 0.15);
        event_triggered = true;
        event_reason = Some("crypto liquidity volatility warning".to_string());
        notes.push("liquidity-style member warned against crypto risk".to_string());
    }

    MemberOpinion {
        member_id: member.member_id.clone(),
        symbol: market.symbol.clone(),
        market_scope: market.market_scope,
        stance,
        confidence: clamp_unit(confidence),
        expected_return_hint,
        risk_hint,
        evidence_notes: notes,
        event_triggered,
        event_reason,
    }
}

fn combined_sentiment(news: &[NewsSnapshot]) -> &'static str {
    if news
        .iter()
        .any(|item| item.sentiment_hint.eq_ignore_ascii_case("positive"))
    {
        "positive"
    } else if news
        .iter()
        .any(|item| item.sentiment_hint.eq_ignore_ascii_case("negative"))
    {
        "negative"
    } else {
        "unknown"
    }
}

fn select_event(
    market: &MarketDataSnapshot,
    opinions: &[MemberOpinion],
) -> Option<InvestmentEvent> {
    let triggering_opinion = opinions
        .iter()
        .filter(|opinion| {
            opinion.event_triggered && matches!(opinion.stance, MemberStance::BuyProposal)
        })
        .max_by(|left, right| {
            left.confidence
                .partial_cmp(&right.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .or_else(|| {
            opinions
                .iter()
                .filter(|opinion| opinion.event_triggered)
                .max_by(|left, right| {
                    left.confidence
                        .partial_cmp(&right.confidence)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        })?
        .clone();
    let event_type = match triggering_opinion.stance {
        MemberStance::BuyProposal => InvestmentEventType::EntryProposal,
        MemberStance::SellProposal => InvestmentEventType::ExitProposal,
        MemberStance::NoTrade => InvestmentEventType::RiskWarning,
        MemberStance::NeedMoreEvidence => InvestmentEventType::NeedMoreEvidence,
        MemberStance::Hold => InvestmentEventType::NeedMoreEvidence,
    };
    Some(InvestmentEvent {
        event_id: format!(
            "event-{}-{}-{}",
            market.symbol, market.timestamp, triggering_opinion.member_id
        ),
        proposed_by_member_id: triggering_opinion.member_id.clone(),
        symbol: market.symbol.clone(),
        market_scope: market.market_scope,
        event_type,
        triggering_opinion,
        created_at: market.timestamp.clone(),
    })
}

fn build_committee_session(
    market: &MarketDataSnapshot,
    event: InvestmentEvent,
    members: &[AICommitteeMember],
    opinions: &[MemberOpinion],
) -> CommitteeSession {
    let buy_count = opinions
        .iter()
        .filter(|opinion| matches!(opinion.stance, MemberStance::BuyProposal))
        .count();
    let defensive_count = opinions
        .iter()
        .filter(|opinion| {
            matches!(
                opinion.stance,
                MemberStance::NoTrade | MemberStance::NeedMoreEvidence
            )
        })
        .count();
    let total = opinions.len().max(1) as f64;
    let mut risk_flags = Vec::new();
    if market.volatility_hint >= 0.08 {
        risk_flags.push("high_volatility".to_string());
    }
    if opinions
        .iter()
        .any(|opinion| matches!(opinion.stance, MemberStance::NoTrade))
    {
        risk_flags.push("member_no_trade_warning".to_string());
    }
    CommitteeSession {
        session_id: format!("session-{}", event.event_id),
        event,
        invited_members: members
            .iter()
            .map(|member| member.member_id.clone())
            .collect(),
        member_opinions: opinions.to_vec(),
        disagreement_level: ((buy_count as f64 - defensive_count as f64).abs() / total)
            .mul_add(-1.0, 1.0)
            .clamp(0.0, 1.0),
        risk_flags,
    }
}

fn synthesize_chairman_decision(
    session: &CommitteeSession,
    risk_veto_volatility_threshold: f64,
) -> ChairmanDecision {
    let weighted_buy: f64 = session
        .member_opinions
        .iter()
        .filter(|opinion| matches!(opinion.stance, MemberStance::BuyProposal))
        .map(|opinion| opinion.confidence)
        .sum();
    let weighted_defensive: f64 = session
        .member_opinions
        .iter()
        .filter(|opinion| {
            matches!(
                opinion.stance,
                MemberStance::NoTrade | MemberStance::NeedMoreEvidence
            )
        })
        .map(|opinion| opinion.confidence)
        .sum();
    let vetoed = session
        .risk_flags
        .iter()
        .any(|flag| flag == "high_volatility")
        && session
            .member_opinions
            .iter()
            .any(|opinion| opinion.risk_hint >= risk_veto_volatility_threshold);

    let (final_action, risk_governor_status, rationale) = if vetoed {
        (
            ChairmanFinalAction::RiskVetoed,
            RiskGovernorStatus::Vetoed,
            "Risk Governor vetoed high-volatility paper proposal".to_string(),
        )
    } else if weighted_buy > weighted_defensive && weighted_buy >= 0.65 {
        (
            ChairmanFinalAction::PaperBuy,
            RiskGovernorStatus::Passed,
            "Chairman synthesized weighted member opinions into paper buy only".to_string(),
        )
    } else if weighted_defensive > weighted_buy {
        (
            ChairmanFinalAction::PaperNoTrade,
            RiskGovernorStatus::NeedsReview,
            "Defensive or low-evidence opinions outweighed entry proposal".to_string(),
        )
    } else {
        (
            ChairmanFinalAction::NeedMoreEvidence,
            RiskGovernorStatus::NeedsReview,
            "Committee needs more evidence before a paper-only decision".to_string(),
        )
    };

    ChairmanDecision {
        decision_id: format!("decision-{}", session.session_id),
        session_id: session.session_id.clone(),
        final_action,
        rationale,
        winning_arguments: session
            .member_opinions
            .iter()
            .filter(|opinion| match final_action {
                ChairmanFinalAction::RiskVetoed => {
                    matches!(opinion.stance, MemberStance::NoTrade)
                }
                ChairmanFinalAction::PaperBuy => {
                    matches!(opinion.stance, MemberStance::BuyProposal)
                }
                _ => matches!(
                    opinion.stance,
                    MemberStance::NoTrade | MemberStance::NeedMoreEvidence
                ),
            })
            .map(|opinion| format!("{}:{:?}", opinion.member_id, opinion.stance))
            .collect(),
        dissenting_arguments: session
            .member_opinions
            .iter()
            .filter(|opinion| match final_action {
                ChairmanFinalAction::RiskVetoed => {
                    matches!(opinion.stance, MemberStance::BuyProposal)
                }
                ChairmanFinalAction::PaperBuy => {
                    !matches!(opinion.stance, MemberStance::BuyProposal)
                }
                _ => matches!(opinion.stance, MemberStance::BuyProposal),
            })
            .map(|opinion| format!("{}:{:?}", opinion.member_id, opinion.stance))
            .collect(),
        risk_governor_status,
    }
}

fn score_update_for(
    member: &AICommitteeMember,
    opinion: &MemberOpinion,
    decision: Option<&ChairmanDecision>,
    simulated_outcome_return: Option<f64>,
) -> MemberScoreUpdate {
    let update_reason = match decision.map(|decision| decision.final_action) {
        Some(ChairmanFinalAction::RiskVetoed)
            if matches!(opinion.stance, MemberStance::NoTrade) =>
        {
            MemberScoreUpdateReason::HelpfulDissent
        }
        Some(ChairmanFinalAction::RiskVetoed)
            if matches!(opinion.stance, MemberStance::BuyProposal) =>
        {
            MemberScoreUpdateReason::RiskyCall
        }
        Some(ChairmanFinalAction::PaperBuy)
            if matches!(opinion.stance, MemberStance::BuyProposal)
                && simulated_outcome_return.unwrap_or(0.0) > 0.0 =>
        {
            MemberScoreUpdateReason::GoodCall
        }
        Some(ChairmanFinalAction::PaperBuy)
            if matches!(opinion.stance, MemberStance::BuyProposal) =>
        {
            MemberScoreUpdateReason::BadCall
        }
        Some(ChairmanFinalAction::NeedMoreEvidence)
            if matches!(opinion.stance, MemberStance::NeedMoreEvidence) =>
        {
            MemberScoreUpdateReason::LowEvidence
        }
        _ => MemberScoreUpdateReason::Neutral,
    };
    let delta = match update_reason {
        MemberScoreUpdateReason::GoodCall | MemberScoreUpdateReason::HelpfulDissent => 0.04,
        MemberScoreUpdateReason::BadCall | MemberScoreUpdateReason::RiskyCall => -0.05,
        MemberScoreUpdateReason::LowEvidence => -0.01,
        MemberScoreUpdateReason::Neutral => 0.0,
    };
    let new_score = clamp_unit(member.score + delta);
    let new_voice_weight = clamp_unit(member.voice_weight + delta / 2.0);
    MemberScoreUpdate {
        member_id: member.member_id.clone(),
        previous_score: member.score,
        new_score,
        previous_voice_weight: member.voice_weight,
        new_voice_weight,
        update_reason,
        promoted: member.score < 0.8 && new_score >= 0.8,
        demoted: member.score >= 0.3 && new_score < 0.3,
    }
}

fn safety_summary() -> MinimalCommitteeSafetySummary {
    MinimalCommitteeSafetySummary {
        paper_only: true,
        no_real_order_path: true,
        no_broker_order_account: true,
        no_model_training: true,
        no_live_inference: true,
        runtime_mode: "LocalMockOrOfflineFixtureOnly".to_string(),
        notes: vec![
            "program orchestrates; AI members analyze through local mock or offline fixture logic"
                .to_string(),
            "no live trading, broker, order, account, training, or live inference path".to_string(),
        ],
    }
}
