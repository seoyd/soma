pub mod acquisition;
pub mod alpaca_provider;
pub mod collector;
pub mod config_gen;
pub mod cost_profile;
pub mod credential_profiles;
pub mod csv_format;
pub mod csv_loader;
pub mod datagokr_provider;
pub mod entitlement;
pub mod evidence_estimate;
pub mod format_detect;
pub mod freshness;
pub mod kis_market_data;
pub mod krx_snapshot;
pub mod manifest;
pub mod official_collection;
pub mod onboarding;
pub mod preflight;
pub mod provenance;
pub mod provider_auth;
pub mod provider_catalog;
pub mod provider_selection;
pub mod quality;
pub mod resample;
pub mod source;
pub mod symbol;
pub mod timeframe;
pub mod upbit_historical_pilot;
pub mod validation;
pub mod yfinance_bridge;

pub use acquisition::{
    AcquisitionMarketScope, AcquisitionMode, AcquisitionPlan, AcquisitionPolicy,
    AcquisitionReceipt, AcquisitionReceiptStatus, AgentDataIntent, AgentDataPolicy,
    AgentEvidenceBundle, AgentProposalEvidenceBinding, AutonomousDataCycleInput,
    AutonomousDataCyclePlan, AutonomousDataCycleResult, BrokerExecutionResult, ConfiguredUniverse,
    DataAcquisitionBroker, DataLookback, DataPriority, DataSnapshot, DatasetKind,
    EvidenceDecisionGate, EvidenceFreshnessStatus, FrozenSnapshotSet, InMemorySnapshotStore,
    MockReadOnlyProvider, ProviderCapabilities, ProviderFetchFailure, ReadOnlyMarketDataProvider,
    ReadOnlyProviderRegistry, ReadOnlyProviderRequest, ReadOnlyProviderResponse,
    RejectedAcquisitionRequest, SnapshotProvenance, SnapshotQualitySummary, SnapshotSourceType,
    StaleDataPolicy, bind_proposal_to_frozen_evidence, build_acquisition_plan,
    build_agent_evidence_bundles, canonical_snapshot_semantic_digest_v1,
    default_agent_data_policies, execute_autonomous_data_cycle, freeze_decision_snapshot_set,
    historical_replay_dataset_digest_v0, plan_agent_data_intent, plan_autonomous_data_cycle,
    snapshot_id_from_semantic_digest_v1,
};
pub use alpaca_provider::{
    AlpacaHistoricalBarsImportConfig, AlpacaHistoricalBarsImportReport, AlpacaProviderStatus,
    default_alpaca_output_dir, parse_alpaca_historical_bars_fixture,
    run_alpaca_historical_bars_import,
};
pub use collector::{
    AdjustedPricePolicy, AuthConfig, AuthRequirement, CandleFetchRequest, CandleFetchResult,
    CollectionBudgetReport, CollectionOutputSize, CollectionSizePolicy, CollectorRunner,
    CollectorSourceKind, CurlHttpClient, FillMissingPolicy, FixtureHttpClient, GenericHttpFixture,
    GenericHttpFixtureResponse, HttpClientError, MarketDataHttpClient, MarketDataProvider,
    ProviderCapability, ProviderKind, RateLimitConfig, RawArchivePolicy, RequestedOutputSize,
    RetentionPolicy,
};
pub use config_gen::{
    ConfigGenerationPolicy, GeneratedConfigBundle, RealEvidenceRerunPlan,
    build_real_evidence_rerun_plan, generate_config_bundle,
};
pub use cost_profile::{
    ProviderCostProfile, ProviderCostTier, default_provider_cost_profiles, provider_cost_profile,
};
pub use credential_profiles::{
    ProviderAuthCheckMode, ProviderCredentialProfile, ProviderCredentialStatus,
    ProviderCredentialStatusKind, ProviderSecretValuePolicy, default_provider_credential_profiles,
    evaluate_provider_credential_profile, evaluate_provider_credential_profiles,
};
pub use csv_format::{
    CandleCsvConfig, CandleCsvFormat, CustomColumnMap, TimestampFormat, logical_column_map,
};
pub use csv_loader::{CandleCsvLoader, CandleLoadFailure, LoadedCandleData};
pub use datagokr_provider::{
    DataGoKrFscStockPriceImportConfig, DataGoKrFscStockPriceImportReport, DataGoKrProviderStatus,
    default_datagokr_output_dir, parse_datagokr_fsc_stock_price_fixture,
    run_datagokr_fsc_stock_price_import,
};
pub use entitlement::{
    ProviderEntitlementPreflightConfig, ProviderEntitlementPreflightRunner,
    ProviderEntitlementStatus, ProviderEntitlementStatusKind, ProviderEntitlementUseCase,
};
pub use evidence_estimate::{EvidenceTargetEstimate, estimate_evidence_targets};
pub use format_detect::{
    CsvFormatCandidateMapping, CsvFormatDetectionConfidence, CsvFormatDetectionResult,
    CsvFormatDetector,
};
pub use freshness::{
    DataFreshnessTier, ProviderDataSubject, ProviderFreshnessProfile,
    default_provider_freshness_profiles, provider_freshness_profile,
};
pub use kis_market_data::{KisMarketDataRequest, KisMarketEndpoint, build_kis_daily_chart_request};
pub use krx_snapshot::{
    KrxSnapshotCanonicalRow, KrxSnapshotImportConfig, KrxSnapshotImportReport, KrxSnapshotImporter,
    KrxSnapshotSymbolReport, SnapshotTextEncoding, default_import_report_dir,
};
pub use manifest::DataManifest;
pub use official_collection::{
    CompressionMode, CompressionPolicy, OfficialCollectionEntry, OfficialCollectionEntryReport,
    OfficialCollectionEntryStatus, OfficialCollectionPlan, OfficialCollectionReport,
    OfficialCollectionRunner, StorageBudget, StorageBudgetReport,
};
pub use onboarding::LocalDataOnboardingConfig;
pub use preflight::{
    PreflightCheck, PreflightCheckResult, PreflightCheckStatus, PreflightFinalStatus,
    PreflightReport, PreflightValidator,
};
pub use provenance::DataProvenance;
pub use provider_auth::{
    ProviderAuthEnvRequirement, ProviderAuthPreflightConfig, ProviderAuthPreflightReport,
    ProviderAuthPreflightRunner, ProviderAuthStatus, ProviderAuthStatusKind,
};
pub use provider_catalog::{
    MarketDataProviderCatalog, ProviderCatalogEntry, ProviderImplementedStatus, ProviderMarket,
    ProviderSourceClass, ProviderSupportedOutput, build_default_provider_catalog,
};
pub use provider_selection::{
    ProviderSelectionPolicy, ProviderSelectionResult, ProviderSelectionResultStatus,
    default_provider_selection_policies, select_provider,
};
pub use quality::{DataQualityReport, DataQualitySeverity, build_data_quality_report};
pub use resample::{ResampleConfig, ResampleMethod, ResampleResult, Resampler};
pub use source::{DataSourceKind, EvidenceSourceKind, EvidenceUse, infer_source_kind_from_path};
pub use symbol::{AssetClass, MarketVenue, SymbolRegistry, SymbolSpec};
pub use timeframe::TimeframeSpec;
pub use upbit_historical_pilot::{
    FirstHistoricalHarvestResultV0, FirstHistoricalHarvestStatusV0,
    HistoricalProviderQualificationStatusV0, HistoricalProviderQualificationV0,
    HistoricalProviderSelectionStatusV0, HistoricalProviderSelectionV0, NetworkConsentV0,
    SnapshotCodec, SnapshotStorageFormat, UpbitDailyOhlcvProviderV0,
    UpbitHistoricalBackfillResultV0, UpbitHistoricalBackfillStatusV0, UpbitHistoricalPageReceiptV0,
    UpbitHistoricalPilotConfigV0, UpbitHistoricalPreflightStatusV0, UpbitHistoricalPreflightV0,
    merge_existing_upbit_snapshot_v0, merge_upbit_historical_pages_v0,
    migrate_legacy_json_snapshot_v0, parse_upbit_daily_ohlcv_v0,
    preflight_upbit_historical_backfill_v0, qualify_upbit_historical_provider_v0,
    read_local_snapshot_protobuf_v1, run_manual_upbit_historical_backfill_v0,
    run_manual_upbit_historical_smoke_v0, select_upbit_historical_provider_v0,
    write_and_verify_local_snapshot_v0,
};
pub use validation::{
    CandleParseError, CandleParseIssue, DataValidationConfig, ValidationStats,
    detect_temporal_issues, validate_candle,
};
pub use yfinance_bridge::{
    YFinanceImportConfig, YFinancePreflightBridge, YFinanceResearchManifest,
    build_yfinance_local_onboarding_config, run_yfinance_preflight_bridge,
};
