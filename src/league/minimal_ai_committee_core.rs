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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
pub struct MinimalAiCommitteeCycleConfig {
    #[serde(default = "default_input_path")]
    pub input_path: Option<String>,
    #[serde(default)]
    pub offline_member_opinion_path: Option<String>,
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
        toml::from_str(&text).map_err(|err| err.to_string())
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
}

pub fn run_minimal_committee_cycle_from_config_path(
    path: &Path,
) -> Result<MinimalCommitteeCycleResult, String> {
    let config = MinimalAiCommitteeCycleConfig::from_toml_path(path)?;
    run_minimal_committee_cycle(config.load_input()?)
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
