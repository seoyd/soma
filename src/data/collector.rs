use std::cell::RefCell;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::backtest::{Candle, Timeframe};
use crate::core::ReasonCode;

use super::{
    AssetClass, CandleCsvConfig, CandleCsvFormat, CandleCsvLoader, ConfigGenerationPolicy,
    DataManifest, DataProvenance, EvidenceSourceKind, LocalDataOnboardingConfig, MarketVenue,
    PreflightFinalStatus, PreflightValidator, SymbolSpec, TimeframeSpec,
    build_real_evidence_rerun_plan,
};

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum ProviderKind {
    #[default]
    Upbit,
    Binance,
    Korbit,
    KrxOpenApi,
    DataGoKrFscStockPrice,
    AlphaVantage,
    Alpaca,
    KoreaInvestmentMarketData,
    PolygonProfessional,
    NasdaqDataLink,
    KoscomProfessional,
    MockFixture,
    Unknown,
}

impl ProviderKind {
    pub fn profile(self) -> MarketDataProvider {
        provider_profile(self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollectorSourceKind {
    OfficialPublicApi,
    OfficialAuthenticatedMarketDataApi,
    FixtureReplay,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthRequirement {
    None,
    ApiKeyHeader,
    ApiKeyQueryParam,
    ApiKeySecretHeader,
    OAuthDeferred,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderCapability {
    PublicMarketData,
    AuthenticatedMarketData,
    HistoricalBars,
    DailyBars,
    IntradayBars,
    RealTimeDeferred,
    TradingNotSupported,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FillMissingPolicy {
    #[default]
    LeaveGaps,
    InsertEmptyRows,
    RejectIfGaps,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdjustedPricePolicy {
    Raw,
    Adjusted,
    BothIfAvailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollectionOutputSize {
    Compact,
    FullDisallowed,
    FullAllowedOnlyWithExplicitFlag,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestedOutputSize {
    Compact,
    Full,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RawArchivePolicy {
    None,
    HeadersOnly,
    CompactJson,
    FullRawAllowedOnlyWithExplicitFlag,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetentionPolicy {
    KeepLatestOnly,
    KeepLastNFiles(usize),
    KeepAllWithinBudget,
    DeleteRawAfterCanonicalAndManifest,
    ArchiveCompressedRawOnly,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self::KeepLastNFiles(3)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub max_requests_per_second: f64,
    pub min_delay_ms: u64,
    pub max_retries: usize,
    pub retry_backoff_ms: u64,
    pub respect_provider_limit: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests_per_second: 8.0,
            min_delay_ms: 125,
            max_retries: 2,
            retry_backoff_ms: 250,
            respect_provider_limit: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CollectionSizePolicy {
    pub max_symbols_per_run: usize,
    pub max_rows_per_symbol: usize,
    pub max_total_rows_per_run: usize,
    pub max_raw_bytes_per_run: usize,
    pub max_canonical_bytes_per_run: usize,
    pub max_requests_per_run: usize,
    pub max_days_per_run: usize,
    pub default_outputsize: CollectionOutputSize,
    pub raw_archive_policy: RawArchivePolicy,
    pub retention_policy: RetentionPolicy,
    pub allow_full_history: bool,
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for CollectionSizePolicy {
    fn default() -> Self {
        Self {
            max_symbols_per_run: 5,
            max_rows_per_symbol: 500,
            max_total_rows_per_run: 2_000,
            max_raw_bytes_per_run: 5 * 1024 * 1024,
            max_canonical_bytes_per_run: 2 * 1024 * 1024,
            max_requests_per_run: 20,
            max_days_per_run: 365,
            default_outputsize: CollectionOutputSize::Compact,
            raw_archive_policy: RawArchivePolicy::CompactJson,
            retention_policy: RetentionPolicy::KeepLastNFiles(3),
            allow_full_history: false,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl CollectionSizePolicy {
    pub fn max_effective_rows(&self) -> usize {
        self.max_rows_per_symbol
            .min(self.max_total_rows_per_run)
            .max(1)
    }

    pub fn summary_string(&self) -> String {
        format!(
            "symbols<={}; rows_per_symbol<={}; total_rows<={}; requests<={}; raw_bytes<={}; canonical_bytes<={}; days<={}; output={:?}; raw={:?}; retention={:?}; allow_full_history={}",
            self.max_symbols_per_run,
            self.max_rows_per_symbol,
            self.max_total_rows_per_run,
            self.max_requests_per_run,
            self.max_raw_bytes_per_run,
            self.max_canonical_bytes_per_run,
            self.max_days_per_run,
            self.default_outputsize,
            self.raw_archive_policy,
            self.retention_policy,
            self.allow_full_history
        )
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuthConfig {
    pub provider_kind: ProviderKind,
    #[serde(default)]
    pub api_key_env_var: Option<String>,
    #[serde(default)]
    pub api_secret_env_var: Option<String>,
    #[serde(default)]
    pub auth_header_name: Option<String>,
    #[serde(default)]
    pub query_param_name: Option<String>,
    #[serde(default)]
    pub allow_missing_for_mock: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl AuthConfig {
    pub fn to_deterministic_string(&self) -> String {
        [
            format!("provider_kind={:?}", self.provider_kind),
            format!(
                "api_key_env_var={}",
                self.api_key_env_var.clone().unwrap_or_default()
            ),
            format!(
                "api_secret_env_var={}",
                self.api_secret_env_var.clone().unwrap_or_default()
            ),
            format!(
                "auth_header_name={}",
                self.auth_header_name.clone().unwrap_or_default()
            ),
            format!(
                "query_param_name={}",
                self.query_param_name.clone().unwrap_or_default()
            ),
            format!("allow_missing_for_mock={}", self.allow_missing_for_mock),
            format!(
                "reason_codes={}",
                self.reason_codes
                    .iter()
                    .map(|reason| format!("{reason:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
        ]
        .join("\n")
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarketDataProvider {
    pub kind: ProviderKind,
    pub provider_id: String,
    pub display_name: String,
    pub source_kind: CollectorSourceKind,
    pub venue: MarketVenue,
    pub asset_class: AssetClass,
    pub auth_requirement: AuthRequirement,
    pub capabilities: Vec<ProviderCapability>,
    pub supports_trading: bool,
    pub supports_account: bool,
    pub public_candles_only: bool,
    pub max_candles_per_request: usize,
    pub supported_timeframes: Vec<Timeframe>,
    pub rate_limit: RateLimitConfig,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CandleFetchRequest {
    pub request_id: String,
    pub provider_kind: ProviderKind,
    pub symbol: String,
    #[serde(default)]
    pub market_venue: Option<MarketVenue>,
    pub asset_class: AssetClass,
    pub timeframe: Timeframe,
    #[serde(default)]
    pub start_timestamp_ms: Option<u64>,
    #[serde(default)]
    pub end_timestamp_ms: Option<u64>,
    pub output_root: String,
    #[serde(default)]
    pub limit_per_request: Option<usize>,
    #[serde(default = "default_true")]
    pub include_raw_archive: bool,
    #[serde(default)]
    pub fill_missing_policy: FillMissingPolicy,
    #[serde(default)]
    pub fixture_path: Option<String>,
    #[serde(default = "default_adjusted_price_policy")]
    pub adjusted_price_policy: AdjustedPricePolicy,
    #[serde(default)]
    pub collection_size_policy: CollectionSizePolicy,
    #[serde(default)]
    pub auth_config: Option<AuthConfig>,
    #[serde(default)]
    pub endpoint_template: Option<String>,
    #[serde(default)]
    pub requested_output_size: Option<RequestedOutputSize>,
    #[serde(default)]
    pub allow_full_history_override: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CollectionBudgetReport {
    pub provider_id: String,
    pub symbol_count: usize,
    pub planned_request_count: usize,
    pub planned_row_budget: usize,
    pub actual_row_count: usize,
    pub raw_bytes_written: usize,
    pub canonical_bytes_written: usize,
    pub truncated: bool,
    pub row_limit_applied: bool,
    pub raw_archive_enabled: bool,
    pub full_history_requested: bool,
    pub collection_size_policy_summary: String,
    pub raw_archive_policy: RawArchivePolicy,
    pub reason_codes: Vec<ReasonCode>,
}

impl CollectionBudgetReport {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        [
            format!("provider_id={}", self.provider_id),
            format!("symbol_count={}", self.symbol_count),
            format!("planned_request_count={}", self.planned_request_count),
            format!("planned_row_budget={}", self.planned_row_budget),
            format!("actual_row_count={}", self.actual_row_count),
            format!("raw_bytes_written={}", self.raw_bytes_written),
            format!("canonical_bytes_written={}", self.canonical_bytes_written),
            format!("truncated={}", self.truncated),
            format!("row_limit_applied={}", self.row_limit_applied),
            format!("raw_archive_enabled={}", self.raw_archive_enabled),
            format!("full_history_requested={}", self.full_history_requested),
            format!(
                "collection_size_policy_summary={}",
                self.collection_size_policy_summary
            ),
            format!("raw_archive_policy={:?}", self.raw_archive_policy),
            format!(
                "reason_codes={}",
                self.reason_codes
                    .iter()
                    .map(|reason| format!("{reason:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
        ]
        .join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let text_path = output_dir.join("collection_budget_report.txt");
        let json_path = output_dir.join("collection_budget_report.json");
        fs::write(&text_path, self.to_text()).map_err(|err| err.to_string())?;
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(text_path)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CandleFetchResult {
    pub request_id: String,
    pub provider_kind: ProviderKind,
    pub provider_id: String,
    pub symbol: String,
    pub normalized_symbol: String,
    pub venue: MarketVenue,
    pub asset_class: AssetClass,
    pub timeframe: Timeframe,
    pub output_dir: String,
    pub request_count: usize,
    pub row_count: usize,
    pub truncated: bool,
    pub row_limit_applied: bool,
    pub raw_request_paths: Vec<String>,
    pub canonical_csv_path: String,
    pub manifest_path: String,
    pub provenance_path: String,
    pub budget_report_path: String,
    pub preflight_status: PreflightFinalStatus,
    pub ready_for_real_evidence: bool,
    pub quality_score: f64,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl CandleFetchResult {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        [
            format!("request_id={}", self.request_id),
            format!("provider={}", self.provider_id),
            format!("symbol={}", self.normalized_symbol),
            format!("venue={:?}", self.venue),
            format!("asset_class={:?}", self.asset_class),
            format!("timeframe={:?}", self.timeframe),
            format!("output_dir={}", self.output_dir),
            format!("request_count={}", self.request_count),
            format!("row_count={}", self.row_count),
            format!("truncated={}", self.truncated),
            format!("row_limit_applied={}", self.row_limit_applied),
            format!("canonical_csv_path={}", self.canonical_csv_path),
            format!("manifest_path={}", self.manifest_path),
            format!("provenance_path={}", self.provenance_path),
            format!("budget_report_path={}", self.budget_report_path),
            format!("preflight_status={:?}", self.preflight_status),
            format!("ready_for_real_evidence={}", self.ready_for_real_evidence),
            format!("quality_score={:.6}", self.quality_score),
            format!("warnings={}", self.warnings.join(" | ")),
        ]
        .join("\n")
    }

    pub fn write_to_dir(&self) -> Result<(), String> {
        let output_dir = Path::new(&self.output_dir);
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("collector_result.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        fs::write(output_dir.join("collector_result.txt"), self.to_text())
            .map_err(|err| err.to_string())?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GenericHttpFixture {
    pub responses: Vec<GenericHttpFixtureResponse>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GenericHttpFixtureResponse {
    pub match_substring: String,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub body_path: Option<String>,
    #[serde(default)]
    pub fail_times: usize,
    #[serde(default)]
    pub permanent_failure: bool,
}

#[derive(Clone, Debug, PartialEq)]
struct ProviderRequestWindow {
    url: String,
    raw_file_name: String,
}

#[derive(Clone, Debug, PartialEq)]
struct CanonicalCollection {
    output_dir: PathBuf,
    raw_dir: PathBuf,
    canonical_dir: PathBuf,
    manifest_path: PathBuf,
    provenance_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq)]
struct CollectionArtifacts {
    raw_request_paths: Vec<String>,
    canonical_csv_path: PathBuf,
    manifest_path: PathBuf,
    provenance_path: PathBuf,
    budget_report_path: PathBuf,
    row_count: usize,
    truncated: bool,
    row_limit_applied: bool,
    quality_score: f64,
    preflight_status: PreflightFinalStatus,
    ready_for_real_evidence: bool,
    warnings: Vec<String>,
    reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CollectorRunner;

impl CollectorRunner {
    pub fn run(&self, request: &CandleFetchRequest) -> Result<CandleFetchResult, String> {
        validate_request(request)?;
        if let Some(fixture_path) = request.fixture_path.as_deref() {
            let client = client_from_fixture_path(Path::new(fixture_path))?;
            self.run_with_client(request, client.as_ref())
        } else {
            let client = CurlHttpClient;
            self.run_with_client(request, &client)
        }
    }

    pub fn run_with_client(
        &self,
        request: &CandleFetchRequest,
        client: &dyn MarketDataHttpClient,
    ) -> Result<CandleFetchResult, String> {
        validate_request(request)?;
        let provider = provider_profile(request.provider_kind);
        let venue = request.market_venue.unwrap_or(provider.venue);
        let asset_class = if request.asset_class == AssetClass::Unknown {
            provider.asset_class
        } else {
            request.asset_class
        };
        let symbol_spec = SymbolSpec::new(request.symbol.clone(), venue, asset_class);
        let collection = build_collection_paths(request, &provider, &symbol_spec, venue);
        let auth_value = resolve_auth_token(request, &provider)?;
        fs::create_dir_all(&collection.canonical_dir).map_err(|err| err.to_string())?;
        if raw_archive_enabled(request) {
            fs::create_dir_all(&collection.raw_dir).map_err(|err| err.to_string())?;
        }

        let plan = plan_request_windows(request, &provider, auth_value.as_deref(), venue)?;
        let mut reason_codes = request.reason_codes.clone();
        reason_codes.extend(request.collection_size_policy.reason_codes.iter().cloned());
        reason_codes.push(ReasonCode::MarketDataCollectionStarted);
        reason_codes.push(ReasonCode::ProviderCapabilityValidated);
        reason_codes.push(ReasonCode::ProviderRequestPlanned);
        if request.provider_kind == ProviderKind::MockFixture {
            reason_codes.push(ReasonCode::MockFixtureLoaded);
        }
        if request.auth_config.is_some() {
            reason_codes.push(ReasonCode::AuthConfigValidated);
        }
        let artifacts = collect_provider_data(
            request,
            &provider,
            &symbol_spec,
            venue,
            &collection,
            &plan,
            client,
        )?;
        reason_codes.extend(artifacts.reason_codes.iter().cloned());
        let result = CandleFetchResult {
            request_id: request.request_id.clone(),
            provider_kind: request.provider_kind,
            provider_id: provider.provider_id,
            symbol: request.symbol.clone(),
            normalized_symbol: symbol_spec.normalized_symbol,
            venue,
            asset_class,
            timeframe: request.timeframe,
            output_dir: collection.output_dir.display().to_string(),
            request_count: plan.windows.len(),
            row_count: artifacts.row_count,
            truncated: artifacts.truncated,
            row_limit_applied: artifacts.row_limit_applied,
            raw_request_paths: artifacts.raw_request_paths,
            canonical_csv_path: artifacts.canonical_csv_path.display().to_string(),
            manifest_path: artifacts.manifest_path.display().to_string(),
            provenance_path: artifacts.provenance_path.display().to_string(),
            budget_report_path: artifacts.budget_report_path.display().to_string(),
            preflight_status: artifacts.preflight_status,
            ready_for_real_evidence: artifacts.ready_for_real_evidence,
            quality_score: artifacts.quality_score,
            warnings: artifacts.warnings,
            reason_codes: dedupe_reasons(reason_codes),
        };
        result.write_to_dir()?;
        Ok(result)
    }
}

pub trait MarketDataHttpClient {
    fn get(&self, url: &str) -> Result<String, HttpClientError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HttpClientError {
    Transient(String),
    Permanent(String),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CurlHttpClient;

impl MarketDataHttpClient for CurlHttpClient {
    fn get(&self, url: &str) -> Result<String, HttpClientError> {
        let output = Command::new("curl")
            .args(["--silent", "--show-error", "--fail", url])
            .output()
            .map_err(|err| HttpClientError::Permanent(err.to_string()))?;
        if output.status.success() {
            String::from_utf8(output.stdout)
                .map_err(|err| HttpClientError::Permanent(err.to_string()))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(HttpClientError::Transient(if stderr.is_empty() {
                "curl request failed".to_string()
            } else {
                stderr
            }))
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FixtureHttpClient {
    base_dir: PathBuf,
    fixture: GenericHttpFixture,
    calls: RefCell<BTreeMap<String, usize>>,
}

impl FixtureHttpClient {
    pub fn from_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_text(
            path.parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from(".")),
            &text,
        )
    }

    fn from_text(base_dir: PathBuf, text: &str) -> Result<Self, String> {
        let fixture =
            serde_json::from_str::<GenericHttpFixture>(text).map_err(|err| err.to_string())?;
        Ok(Self {
            base_dir,
            fixture,
            calls: RefCell::new(BTreeMap::new()),
        })
    }
}

impl MarketDataHttpClient for FixtureHttpClient {
    fn get(&self, url: &str) -> Result<String, HttpClientError> {
        for response in &self.fixture.responses {
            if !url.contains(&response.match_substring) {
                continue;
            }
            let mut calls = self.calls.borrow_mut();
            let call_count = calls.entry(response.match_substring.clone()).or_insert(0);
            *call_count += 1;
            if *call_count <= response.fail_times {
                return Err(HttpClientError::Transient(format!(
                    "fixture transient failure for {}",
                    response.match_substring
                )));
            }
            if response.permanent_failure {
                return Err(HttpClientError::Permanent(format!(
                    "fixture permanent failure for {}",
                    response.match_substring
                )));
            }
            if let Some(body) = &response.body {
                return Ok(body.clone());
            }
            if let Some(body_path) = &response.body_path {
                return fs::read_to_string(self.base_dir.join(body_path))
                    .map_err(|err| HttpClientError::Permanent(err.to_string()));
            }
            return Ok("[]".to_string());
        }
        Err(HttpClientError::Permanent(format!(
            "no fixture response matched request: {url}"
        )))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StaticBodyHttpClient {
    body: String,
}

impl StaticBodyHttpClient {
    fn new(body: String) -> Self {
        Self { body }
    }
}

impl MarketDataHttpClient for StaticBodyHttpClient {
    fn get(&self, _url: &str) -> Result<String, HttpClientError> {
        Ok(self.body.clone())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct UpbitCandleResponse {
    timestamp: u64,
    opening_price: f64,
    high_price: f64,
    low_price: f64,
    trade_price: f64,
    candle_acc_trade_price: f64,
    candle_acc_trade_volume: f64,
}

#[derive(Clone, Debug, PartialEq)]
struct RequestPlan {
    windows: Vec<ProviderRequestWindow>,
    planned_row_budget: usize,
    truncated: bool,
    row_limit_applied: bool,
    full_history_requested: bool,
}

fn collect_provider_data(
    request: &CandleFetchRequest,
    provider: &MarketDataProvider,
    symbol_spec: &SymbolSpec,
    venue: MarketVenue,
    collection: &CanonicalCollection,
    plan: &RequestPlan,
    client: &dyn MarketDataHttpClient,
) -> Result<CollectionArtifacts, String> {
    let timeframe_spec = TimeframeSpec::from_timeframe(request.timeframe);
    if !timeframe_spec.is_supported() {
        return Err("collector timeframe is unsupported".to_string());
    }

    let mut reason_codes = Vec::new();
    let mut warnings = Vec::new();
    let mut raw_request_paths = Vec::new();
    let mut raw_bytes_written = 0usize;
    let mut fetched = Vec::new();
    if provider.source_kind != CollectorSourceKind::FixtureReplay {
        reason_codes.push(ReasonCode::OfficialApiCollected);
    }

    for window in &plan.windows {
        let body = fetch_with_retry(client, provider, &window.url, &mut reason_codes)?;
        if let Some(raw_path) = archive_raw_response(
            request,
            &window.url,
            &window.raw_file_name,
            &body,
            collection,
            &mut raw_bytes_written,
            &mut reason_codes,
        )? {
            raw_request_paths.push(raw_path.display().to_string());
        }
        fetched.extend(parse_provider_body(provider.kind, &body)?);
    }

    let mut candles = normalize_provider_candles(
        fetched,
        request.start_timestamp_ms,
        request.end_timestamp_ms,
    );
    if candles.is_empty() {
        return Err("provider returned no candles in requested window".to_string());
    }

    let row_limit = request.collection_size_policy.max_effective_rows();
    let mut row_limit_applied = plan.row_limit_applied;
    let mut truncated = plan.truncated;
    if candles.len() > row_limit {
        let start = candles.len().saturating_sub(row_limit);
        candles = candles[start..].to_vec();
        row_limit_applied = true;
        truncated = true;
        reason_codes.push(ReasonCode::RowLimitApplied);
        reason_codes.push(ReasonCode::TruncatedCollectionRecorded);
    }

    reason_codes.extend(apply_fill_missing_policy(
        &mut candles,
        timeframe_spec.expected_ms_step,
        request.fill_missing_policy,
    )?);

    let first_timestamp_ms = candles
        .first()
        .map(|candle| candle.timestamp_ms)
        .ok_or_else(|| "canonical candle series became empty".to_string())?;
    let last_timestamp_ms = candles
        .last()
        .map(|candle| candle.timestamp_ms)
        .ok_or_else(|| "canonical candle series became empty".to_string())?;
    let canonical_csv_path = collection.canonical_dir.join(format!(
        "{}_{}_{}_{}_compact.csv",
        symbol_spec.normalized_symbol,
        timeframe_slug(request.timeframe),
        first_timestamp_ms,
        last_timestamp_ms
    ));
    let canonical_contents = render_canonical_csv(&candles);
    let canonical_bytes = canonical_contents.len();
    if canonical_bytes > request.collection_size_policy.max_canonical_bytes_per_run {
        reason_codes.push(ReasonCode::CollectionBudgetExceeded);
        return Err("canonical CSV would exceed max_canonical_bytes_per_run".to_string());
    }
    fs::write(&canonical_csv_path, &canonical_contents).map_err(|err| err.to_string())?;
    reason_codes.push(ReasonCode::CanonicalCandleWritten);

    apply_retention_policy(
        &collection.canonical_dir,
        request.collection_size_policy.retention_policy,
    )?;
    if raw_archive_enabled(request) {
        apply_retention_policy(
            &collection.raw_dir,
            request.collection_size_policy.retention_policy,
        )?;
    }

    let csv_config = CandleCsvConfig {
        format: CandleCsvFormat::GenericOhlcv,
        symbol: symbol_spec.raw_symbol.clone(),
        timeframe: request.timeframe,
        has_header: true,
        delimiter: ',',
        ..CandleCsvConfig::default()
    };
    let loaded = CandleCsvLoader::default()
        .load_from_path(&canonical_csv_path, &csv_config)
        .map_err(|failure| {
            let details = failure
                .issues
                .iter()
                .map(|issue| format!("{:?}", issue.error))
                .collect::<Vec<_>>()
                .join(", ");
            if details.is_empty() {
                "canonical csv validation failed".to_string()
            } else {
                format!("canonical csv validation failed: {details}")
            }
        })?;
    if request.fill_missing_policy == FillMissingPolicy::RejectIfGaps
        && loaded.quality_report.gap_count > 0
    {
        return Err(
            "collector rejected output because gaps remained in canonical series".to_string(),
        );
    }

    let mut provenance_reason_codes = vec![ReasonCode::DeterministicPath];
    if truncated {
        provenance_reason_codes.push(ReasonCode::TruncatedCollectionRecorded);
    }
    if row_limit_applied {
        provenance_reason_codes.push(ReasonCode::RowLimitApplied);
    }
    let provenance = collected_provenance(
        request,
        provider,
        &canonical_csv_path,
        venue,
        provenance_reason_codes,
    );
    fs::write(
        &collection.provenance_path,
        provenance.to_deterministic_string(),
    )
    .map_err(|err| err.to_string())?;

    let mut manifest = DataManifest::build(
        &loaded.series,
        &loaded.symbol_spec,
        &loaded.timeframe_spec,
        &loaded.quality_report,
        provenance.source_kind,
        Some(canonical_csv_path.display().to_string()),
        Some(provenance.clone()),
        Some(first_timestamp_ms),
    );
    manifest.venue = venue;
    manifest.asset_class = symbol_spec.asset_class;
    manifest.adjusted_price_policy_summary = Some(format!("{:?}", request.adjusted_price_policy));
    manifest.corporate_action_adjusted = Some(matches!(
        request.adjusted_price_policy,
        AdjustedPricePolicy::Adjusted
    ));
    manifest.provider_symbol = Some(request.symbol.clone());
    manifest.collection_size_policy_summary = Some(request.collection_size_policy.summary_string());
    manifest.truncated = truncated;
    manifest.row_limit_applied = row_limit_applied;
    manifest.raw_archive_policy_summary = Some(format!(
        "{:?}",
        request.collection_size_policy.raw_archive_policy
    ));
    manifest.auth_requirement_summary = Some(format!("{:?}", provider.auth_requirement));
    manifest
        .reason_codes
        .push(ReasonCode::AdjustedPricePolicyRecorded);
    if truncated {
        manifest
            .reason_codes
            .push(ReasonCode::TruncatedCollectionRecorded);
    }
    if row_limit_applied {
        manifest.reason_codes.push(ReasonCode::RowLimitApplied);
    }
    fs::write(
        &collection.manifest_path,
        manifest.to_deterministic_string(),
    )
    .map_err(|err| err.to_string())?;

    let budget_report = CollectionBudgetReport {
        provider_id: provider.provider_id.clone(),
        symbol_count: 1,
        planned_request_count: plan.windows.len(),
        planned_row_budget: plan.planned_row_budget,
        actual_row_count: loaded.series.len(),
        raw_bytes_written,
        canonical_bytes_written: canonical_bytes,
        truncated,
        row_limit_applied,
        raw_archive_enabled: raw_archive_enabled(request),
        full_history_requested: plan.full_history_requested,
        collection_size_policy_summary: request.collection_size_policy.summary_string(),
        raw_archive_policy: request.collection_size_policy.raw_archive_policy,
        reason_codes: dedupe_reasons({
            let mut values = vec![
                ReasonCode::CollectionBudgetApplied,
                ReasonCode::CollectionBudgetReportBuilt,
            ];
            if row_limit_applied {
                values.push(ReasonCode::RowLimitApplied);
            }
            if truncated {
                values.push(ReasonCode::TruncatedCollectionRecorded);
            }
            values
        }),
    };
    let budget_report_path = budget_report.write_to_dir(&collection.output_dir)?;

    let mut onboarding_reason_codes = vec![ReasonCode::CollectorPreflightAutoRun];
    if truncated {
        onboarding_reason_codes.push(ReasonCode::TruncatedCollectionRecorded);
        warnings.push("collection was truncated to remain within declared size policy".to_string());
    }
    if row_limit_applied {
        onboarding_reason_codes.push(ReasonCode::RowLimitApplied);
    }
    if loaded.quality_report.gap_count > 0 {
        warnings.push(format!(
            "detected {} temporal gaps in canonical series",
            loaded.quality_report.gap_count
        ));
    }
    if provider.kind == ProviderKind::MockFixture {
        warnings.push("mock fixture provider does not count as readiness evidence".to_string());
    }
    let onboarding = LocalDataOnboardingConfig {
        onboarding_id: format!("{}-{}", request.request_id, provider.provider_id),
        input_path: canonical_csv_path.display().to_string(),
        output_root: collection.output_dir.display().to_string(),
        symbol: Some(symbol_spec.raw_symbol.clone()),
        venue: Some(venue),
        asset_class: Some(symbol_spec.asset_class),
        timeframe: Some(request.timeframe),
        csv_format_hint: Some(CandleCsvFormat::GenericOhlcv),
        custom_column_map: None,
        source_kind: Some(provenance.source_kind),
        user_supplied: false,
        source_label: Some(provenance.source_label.clone()),
        strict: true,
        allow_format_autodetect: true,
        allow_sort_repair: false,
        allow_duplicate_drop: false,
        min_rows_for_preflight: request
            .collection_size_policy
            .max_rows_per_symbol
            .min(200)
            .max(40),
        target_min_outcomes: request
            .collection_size_policy
            .max_rows_per_symbol
            .min(200)
            .max(20),
        target_min_comparable_variants: 2,
        target_min_usable_datasets: 1,
        walk_forward_config: None,
        triple_barrier_config: None,
        cost_model: None,
        reason_codes: onboarding_reason_codes,
    };
    let preflight = PreflightValidator::default().run(&onboarding);
    let rerun_plan = build_real_evidence_rerun_plan(
        &onboarding,
        preflight.clone(),
        ConfigGenerationPolicy::ReadyOnly,
    );
    rerun_plan.write_to_dir(&collection.output_dir)?;

    Ok(CollectionArtifacts {
        raw_request_paths,
        canonical_csv_path,
        manifest_path: collection.manifest_path.clone(),
        provenance_path: collection.provenance_path.clone(),
        budget_report_path,
        row_count: loaded.series.len(),
        truncated,
        row_limit_applied,
        quality_score: loaded.quality_report.data_quality_score,
        preflight_status: preflight.final_status,
        ready_for_real_evidence: preflight.final_status
            == PreflightFinalStatus::ReadyForRealEvidence,
        warnings,
        reason_codes: dedupe_reasons(reason_codes),
    })
}

fn provider_profile(kind: ProviderKind) -> MarketDataProvider {
    match kind {
        ProviderKind::Upbit => MarketDataProvider {
            kind,
            provider_id: "upbit".to_string(),
            display_name: "Upbit public candles".to_string(),
            source_kind: CollectorSourceKind::OfficialPublicApi,
            venue: MarketVenue::Upbit,
            asset_class: AssetClass::Crypto,
            auth_requirement: AuthRequirement::None,
            capabilities: vec![
                ProviderCapability::PublicMarketData,
                ProviderCapability::HistoricalBars,
                ProviderCapability::DailyBars,
                ProviderCapability::IntradayBars,
                ProviderCapability::TradingNotSupported,
            ],
            supports_trading: false,
            supports_account: false,
            public_candles_only: true,
            max_candles_per_request: 200,
            supported_timeframes: vec![
                Timeframe::OneMinute,
                Timeframe::FiveMinute,
                Timeframe::FifteenMinute,
                Timeframe::OneHour,
                Timeframe::OneDay,
            ],
            rate_limit: RateLimitConfig::default(),
            notes: vec![
                "research-only public candle provider".to_string(),
                "no auth, no trading, no account scope".to_string(),
            ],
        },
        ProviderKind::KrxOpenApi => MarketDataProvider {
            kind,
            provider_id: "krx".to_string(),
            display_name: "KRX Open API daily provider".to_string(),
            source_kind: CollectorSourceKind::OfficialAuthenticatedMarketDataApi,
            venue: MarketVenue::KRX,
            asset_class: AssetClass::Equity,
            auth_requirement: AuthRequirement::ApiKeyHeader,
            capabilities: vec![
                ProviderCapability::AuthenticatedMarketData,
                ProviderCapability::HistoricalBars,
                ProviderCapability::DailyBars,
                ProviderCapability::TradingNotSupported,
            ],
            supports_trading: false,
            supports_account: false,
            public_candles_only: false,
            max_candles_per_request: 5_000,
            supported_timeframes: vec![Timeframe::OneDay],
            rate_limit: RateLimitConfig {
                max_requests_per_second: 2.0,
                min_delay_ms: 500,
                ..RateLimitConfig::default()
            },
            notes: vec![
                "requires auth key and service approval".to_string(),
                "fixture/live request builder only; no trading/account scope".to_string(),
            ],
        },
        ProviderKind::AlphaVantage => MarketDataProvider {
            kind,
            provider_id: "alphavantage".to_string(),
            display_name: "AlphaVantage compact daily provider".to_string(),
            source_kind: CollectorSourceKind::OfficialAuthenticatedMarketDataApi,
            venue: MarketVenue::US,
            asset_class: AssetClass::Equity,
            auth_requirement: AuthRequirement::ApiKeyQueryParam,
            capabilities: vec![
                ProviderCapability::AuthenticatedMarketData,
                ProviderCapability::HistoricalBars,
                ProviderCapability::DailyBars,
                ProviderCapability::IntradayBars,
                ProviderCapability::TradingNotSupported,
            ],
            supports_trading: false,
            supports_account: false,
            public_candles_only: false,
            max_candles_per_request: 100,
            supported_timeframes: vec![Timeframe::OneDay, Timeframe::OneMinute],
            rate_limit: RateLimitConfig {
                max_requests_per_second: 1.0,
                min_delay_ms: 1_000,
                ..RateLimitConfig::default()
            },
            notes: vec![
                "compact mode is the default safe path".to_string(),
                "API key is passed by env-var name only".to_string(),
            ],
        },
        ProviderKind::Alpaca => MarketDataProvider {
            kind,
            provider_id: "alpaca".to_string(),
            display_name: "Alpaca historical bars stub".to_string(),
            source_kind: CollectorSourceKind::OfficialAuthenticatedMarketDataApi,
            venue: MarketVenue::US,
            asset_class: AssetClass::Equity,
            auth_requirement: AuthRequirement::ApiKeySecretHeader,
            capabilities: vec![
                ProviderCapability::AuthenticatedMarketData,
                ProviderCapability::HistoricalBars,
                ProviderCapability::DailyBars,
                ProviderCapability::IntradayBars,
                ProviderCapability::TradingNotSupported,
            ],
            supports_trading: false,
            supports_account: false,
            public_candles_only: false,
            max_candles_per_request: 500,
            supported_timeframes: vec![Timeframe::OneDay],
            rate_limit: RateLimitConfig::default(),
            notes: vec![
                "provider metadata is present but collection stays deferred in Sprint 19"
                    .to_string(),
            ],
        },
        ProviderKind::DataGoKrFscStockPrice => MarketDataProvider {
            kind,
            provider_id: "data-go-kr-fsc".to_string(),
            display_name: "data.go.kr FSC stock price provider card".to_string(),
            source_kind: CollectorSourceKind::OfficialAuthenticatedMarketDataApi,
            venue: MarketVenue::KRX,
            asset_class: AssetClass::Equity,
            auth_requirement: AuthRequirement::ApiKeyQueryParam,
            capabilities: vec![
                ProviderCapability::AuthenticatedMarketData,
                ProviderCapability::HistoricalBars,
                ProviderCapability::DailyBars,
                ProviderCapability::TradingNotSupported,
            ],
            supports_trading: false,
            supports_account: false,
            public_candles_only: false,
            max_candles_per_request: 500,
            supported_timeframes: vec![Timeframe::OneDay],
            rate_limit: RateLimitConfig::default(),
            notes: vec![
                "service-key-based government stock price path".to_string(),
                "live endpoint profile remains explicit and bounded".to_string(),
            ],
        },
        ProviderKind::KoreaInvestmentMarketData => MarketDataProvider {
            kind,
            provider_id: "kis-market-data".to_string(),
            display_name: "KIS market-data-only provider".to_string(),
            source_kind: CollectorSourceKind::OfficialAuthenticatedMarketDataApi,
            venue: MarketVenue::KRX,
            asset_class: AssetClass::Equity,
            auth_requirement: AuthRequirement::ApiKeySecretHeader,
            capabilities: vec![
                ProviderCapability::AuthenticatedMarketData,
                ProviderCapability::HistoricalBars,
                ProviderCapability::DailyBars,
                ProviderCapability::TradingNotSupported,
            ],
            supports_trading: false,
            supports_account: false,
            public_candles_only: false,
            max_candles_per_request: 500,
            supported_timeframes: vec![Timeframe::OneDay],
            rate_limit: RateLimitConfig::default(),
            notes: vec![
                "market-data-only provider managed by Sprint 51 activation flow".to_string(),
                "order/account endpoints remain forbidden".to_string(),
            ],
        },
        ProviderKind::PolygonProfessional => MarketDataProvider {
            kind,
            provider_id: "polygon".to_string(),
            display_name: "Polygon professional provider card".to_string(),
            source_kind: CollectorSourceKind::OfficialAuthenticatedMarketDataApi,
            venue: MarketVenue::US,
            asset_class: AssetClass::Equity,
            auth_requirement: AuthRequirement::ApiKeyHeader,
            capabilities: vec![
                ProviderCapability::AuthenticatedMarketData,
                ProviderCapability::HistoricalBars,
                ProviderCapability::DailyBars,
                ProviderCapability::IntradayBars,
                ProviderCapability::TradingNotSupported,
            ],
            supports_trading: false,
            supports_account: false,
            public_candles_only: false,
            max_candles_per_request: 5_000,
            supported_timeframes: vec![Timeframe::OneMinute, Timeframe::OneDay],
            rate_limit: RateLimitConfig::default(),
            notes: vec!["professional paid provider card only".to_string()],
        },
        ProviderKind::NasdaqDataLink => MarketDataProvider {
            kind,
            provider_id: "nasdaq-data-link".to_string(),
            display_name: "Nasdaq Data Link provider card".to_string(),
            source_kind: CollectorSourceKind::OfficialAuthenticatedMarketDataApi,
            venue: MarketVenue::US,
            asset_class: AssetClass::Equity,
            auth_requirement: AuthRequirement::ApiKeyHeader,
            capabilities: vec![
                ProviderCapability::AuthenticatedMarketData,
                ProviderCapability::HistoricalBars,
                ProviderCapability::DailyBars,
                ProviderCapability::TradingNotSupported,
            ],
            supports_trading: false,
            supports_account: false,
            public_candles_only: false,
            max_candles_per_request: 5_000,
            supported_timeframes: vec![Timeframe::OneDay],
            rate_limit: RateLimitConfig::default(),
            notes: vec!["professional paid provider card only".to_string()],
        },
        ProviderKind::KoscomProfessional => MarketDataProvider {
            kind,
            provider_id: "koscom".to_string(),
            display_name: "Koscom professional provider card".to_string(),
            source_kind: CollectorSourceKind::OfficialAuthenticatedMarketDataApi,
            venue: MarketVenue::KRX,
            asset_class: AssetClass::Equity,
            auth_requirement: AuthRequirement::ApiKeyHeader,
            capabilities: vec![
                ProviderCapability::AuthenticatedMarketData,
                ProviderCapability::HistoricalBars,
                ProviderCapability::DailyBars,
                ProviderCapability::TradingNotSupported,
            ],
            supports_trading: false,
            supports_account: false,
            public_candles_only: false,
            max_candles_per_request: 5_000,
            supported_timeframes: vec![Timeframe::OneDay],
            rate_limit: RateLimitConfig::default(),
            notes: vec!["professional Korean equity provider card only".to_string()],
        },
        ProviderKind::Korbit => MarketDataProvider {
            kind,
            provider_id: "korbit".to_string(),
            display_name: "Korbit optional crypto provider card".to_string(),
            source_kind: CollectorSourceKind::OfficialPublicApi,
            venue: MarketVenue::Generic,
            asset_class: AssetClass::Crypto,
            auth_requirement: AuthRequirement::None,
            capabilities: vec![
                ProviderCapability::PublicMarketData,
                ProviderCapability::HistoricalBars,
                ProviderCapability::DailyBars,
                ProviderCapability::TradingNotSupported,
            ],
            supports_trading: false,
            supports_account: false,
            public_candles_only: true,
            max_candles_per_request: 200,
            supported_timeframes: vec![Timeframe::OneDay],
            rate_limit: RateLimitConfig::default(),
            notes: vec!["optional/deferred crypto provider card".to_string()],
        },
        ProviderKind::MockFixture => MarketDataProvider {
            kind,
            provider_id: "mock-fixture".to_string(),
            display_name: "Fixture replay collector".to_string(),
            source_kind: CollectorSourceKind::FixtureReplay,
            venue: MarketVenue::Generic,
            asset_class: AssetClass::Unknown,
            auth_requirement: AuthRequirement::None,
            capabilities: vec![
                ProviderCapability::HistoricalBars,
                ProviderCapability::DailyBars,
                ProviderCapability::IntradayBars,
                ProviderCapability::TradingNotSupported,
            ],
            supports_trading: false,
            supports_account: false,
            public_candles_only: false,
            max_candles_per_request: 1,
            supported_timeframes: vec![
                Timeframe::OneMinute,
                Timeframe::FiveMinute,
                Timeframe::FifteenMinute,
                Timeframe::OneHour,
                Timeframe::OneDay,
            ],
            rate_limit: RateLimitConfig::default(),
            notes: vec![
                "offline replay path for deterministic tests".to_string(),
                "can auto-detect upbit/krx/alphavantage shaped payloads".to_string(),
            ],
        },
        ProviderKind::Binance => MarketDataProvider {
            kind,
            provider_id: "binance".to_string(),
            display_name: "Binance deferred provider".to_string(),
            source_kind: CollectorSourceKind::OfficialPublicApi,
            venue: MarketVenue::Binance,
            asset_class: AssetClass::Crypto,
            auth_requirement: AuthRequirement::None,
            capabilities: vec![
                ProviderCapability::PublicMarketData,
                ProviderCapability::HistoricalBars,
                ProviderCapability::DailyBars,
                ProviderCapability::IntradayBars,
                ProviderCapability::TradingNotSupported,
            ],
            supports_trading: false,
            supports_account: false,
            public_candles_only: true,
            max_candles_per_request: 1_000,
            supported_timeframes: vec![Timeframe::OneMinute, Timeframe::OneDay],
            rate_limit: RateLimitConfig::default(),
            notes: vec!["deferred to prioritize equity providers in Sprint 19".to_string()],
        },
        ProviderKind::Unknown => MarketDataProvider {
            kind,
            provider_id: "unknown".to_string(),
            display_name: "Unknown provider".to_string(),
            source_kind: CollectorSourceKind::Unknown,
            venue: MarketVenue::Unknown,
            asset_class: AssetClass::Unknown,
            auth_requirement: AuthRequirement::Unknown,
            capabilities: vec![ProviderCapability::TradingNotSupported],
            supports_trading: false,
            supports_account: false,
            public_candles_only: false,
            max_candles_per_request: 1,
            supported_timeframes: vec![Timeframe::OneDay],
            rate_limit: RateLimitConfig::default(),
            notes: vec!["unknown provider".to_string()],
        },
    }
}

fn validate_request(request: &CandleFetchRequest) -> Result<(), String> {
    if request.output_root.contains("://") {
        return Err("collector output_root must be local".to_string());
    }
    if request
        .fixture_path
        .as_deref()
        .is_some_and(|value| value.contains("://"))
    {
        return Err("collector fixture path must be local".to_string());
    }
    if request.symbol.trim().is_empty() {
        return Err("collector symbol must not be empty".to_string());
    }
    if request.collection_size_policy.max_symbols_per_run == 0 {
        return Err("collector max_symbols_per_run must be >= 1".to_string());
    }
    if request.collection_size_policy.max_symbols_per_run < 1 {
        return Err("collector max_symbols_per_run is invalid".to_string());
    }
    if let (Some(start), Some(end)) = (request.start_timestamp_ms, request.end_timestamp_ms)
        && end < start
    {
        return Err("collector end timestamp must be >= start timestamp".to_string());
    }
    let provider = provider_profile(request.provider_kind);
    if provider.supports_trading || provider.supports_account {
        return Err("collector providers must be market-data-only".to_string());
    }
    if !provider.supported_timeframes.contains(&request.timeframe) {
        return Err("collector timeframe is not supported by provider".to_string());
    }
    if request.provider_kind == ProviderKind::MockFixture && request.fixture_path.is_none() {
        return Err("mock-fixture provider requires fixture_path".to_string());
    }
    if matches!(
        request.provider_kind,
        ProviderKind::Upbit | ProviderKind::KrxOpenApi | ProviderKind::Alpaca
    ) && (request.start_timestamp_ms.is_none() || request.end_timestamp_ms.is_none())
    {
        return Err("selected provider requires both start and end timestamps".to_string());
    }
    if provider.auth_requirement != AuthRequirement::None
        && request.fixture_path.is_none()
        && request.provider_kind != ProviderKind::MockFixture
    {
        let auth = request.auth_config.as_ref().ok_or_else(|| {
            "MissingApiKey: auth_config is required for this provider".to_string()
        })?;
        if auth.api_key_env_var.is_none() {
            return Err("MissingApiKey: api_key_env_var is required for this provider".to_string());
        }
    }
    if request.provider_kind == ProviderKind::KrxOpenApi
        && request.fixture_path.is_none()
        && request.endpoint_template.is_none()
    {
        return Err(
            "KRXProviderConfigured: endpoint_template is required for KRX live collection"
                .to_string(),
        );
    }
    Ok(())
}

fn resolve_auth_token(
    request: &CandleFetchRequest,
    provider: &MarketDataProvider,
) -> Result<Option<String>, String> {
    if provider.auth_requirement == AuthRequirement::None
        || request.fixture_path.is_some()
        || request.provider_kind == ProviderKind::MockFixture
    {
        return Ok(None);
    }
    let auth = request
        .auth_config
        .as_ref()
        .ok_or_else(|| "MissingApiKey: auth_config is required".to_string())?;
    let env_var = auth
        .api_key_env_var
        .as_deref()
        .ok_or_else(|| "MissingApiKey: api_key_env_var is required".to_string())?;
    let value = env::var(env_var)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("MissingApiKey: environment variable {env_var} is not set"))?;
    Ok(Some(value))
}

fn build_collection_paths(
    request: &CandleFetchRequest,
    provider: &MarketDataProvider,
    symbol_spec: &SymbolSpec,
    venue: MarketVenue,
) -> CanonicalCollection {
    let output_dir = Path::new(&request.output_root)
        .join(&provider.provider_id)
        .join(venue_slug(venue))
        .join(&symbol_spec.normalized_symbol)
        .join(timeframe_slug(request.timeframe));
    CanonicalCollection {
        output_dir: output_dir.clone(),
        raw_dir: output_dir.join("raw"),
        canonical_dir: output_dir.join("canonical"),
        manifest_path: output_dir.join("data_manifest.txt"),
        provenance_path: output_dir.join("data_provenance.txt"),
    }
}

fn plan_request_windows(
    request: &CandleFetchRequest,
    provider: &MarketDataProvider,
    auth_token: Option<&str>,
    venue: MarketVenue,
) -> Result<RequestPlan, String> {
    if request.collection_size_policy.max_symbols_per_run < 1 {
        return Err("collector size policy does not allow any symbols".to_string());
    }
    let full_history_requested = request.allow_full_history_override
        || matches!(
            request.requested_output_size,
            Some(RequestedOutputSize::Full)
        );
    if full_history_requested && !request.collection_size_policy.allow_full_history {
        return Err("FullHistoryDenied: policy does not allow full history collection".to_string());
    }
    match provider.kind {
        ProviderKind::Upbit => plan_upbit_windows(request, provider),
        ProviderKind::KrxOpenApi => plan_krx_windows(request, provider, auth_token, venue),
        ProviderKind::AlphaVantage => {
            plan_alphavantage_windows(request, provider, auth_token, full_history_requested)
        }
        ProviderKind::Alpaca => {
            Err("AlpacaProviderDeferred: Alpaca collection is deferred".to_string())
        }
        ProviderKind::DataGoKrFscStockPrice => Err(
            "DataGoKrProviderDeferred: endpoint profile must be configured explicitly".to_string(),
        ),
        ProviderKind::KoreaInvestmentMarketData => {
            Err(
                "KisMarketDataActivationOnly: use soma-experiment kis-market-data-activate or kis-collection-plan for Sprint 51 bounded KIS workflows".to_string(),
            )
        }
        ProviderKind::PolygonProfessional => {
            Err("PolygonProfessional is documented-only in Sprint 29".to_string())
        }
        ProviderKind::NasdaqDataLink => {
            Err("NasdaqDataLink is documented-only in Sprint 29".to_string())
        }
        ProviderKind::KoscomProfessional => {
            Err("KoscomProfessional is documented-only in Sprint 29".to_string())
        }
        ProviderKind::MockFixture => Ok(RequestPlan {
            windows: vec![ProviderRequestWindow {
                url: format!(
                    "https://fixture.local/mock?symbol={}&timeframe={}",
                    request.symbol,
                    timeframe_slug(request.timeframe)
                ),
                raw_file_name: "request_000001.json".to_string(),
            }],
            planned_row_budget: request.collection_size_policy.max_effective_rows(),
            truncated: false,
            row_limit_applied: false,
            full_history_requested,
        }),
        ProviderKind::Binance => Err("Binance remains deferred in Sprint 19".to_string()),
        ProviderKind::Korbit => Err("Korbit remains optional/deferred in Sprint 29".to_string()),
        ProviderKind::Unknown => Err("unknown provider cannot be planned".to_string()),
    }
}

fn plan_upbit_windows(
    request: &CandleFetchRequest,
    provider: &MarketDataProvider,
) -> Result<RequestPlan, String> {
    let start = request
        .start_timestamp_ms
        .ok_or_else(|| "upbit collection requires start timestamp".to_string())?;
    let end = request
        .end_timestamp_ms
        .ok_or_else(|| "upbit collection requires end timestamp".to_string())?;
    let step_ms = TimeframeSpec::from_timeframe(request.timeframe).expected_ms_step;
    if step_ms == 0 {
        return Err("cannot plan upbit requests for unsupported timeframe".to_string());
    }
    let requested_rows = ((end - start) / step_ms).saturating_add(1) as usize;
    let requested_days = days_spanned(start, end);
    if requested_days > request.collection_size_policy.max_days_per_run
        && !request.allow_full_history_override
    {
        return Err("FullHistoryDenied: requested day span exceeds max_days_per_run".to_string());
    }
    let row_budget = requested_rows.min(request.collection_size_policy.max_effective_rows());
    let row_limit_applied = requested_rows > row_budget;
    let truncated = row_limit_applied;
    let adjusted_start = if row_budget == 0 {
        end
    } else {
        end.saturating_sub((row_budget as u64 - 1).saturating_mul(step_ms))
    };
    let max_per_request = request
        .limit_per_request
        .unwrap_or(provider.max_candles_per_request)
        .min(provider.max_candles_per_request)
        .max(1);
    let mut remaining = row_budget as u64;
    let mut windows = Vec::new();
    let mut cursor_end_ms = end;
    let granularity = upbit_granularity(request.timeframe)?;
    let mut index = 1usize;
    while remaining > 0 {
        let count = remaining.min(max_per_request as u64) as usize;
        let url = match granularity {
            UpbitGranularity::Minutes(unit) => format!(
                "https://api.upbit.com/v1/candles/minutes/{unit}?market={}&to={}&count={count}",
                request.symbol,
                timestamp_ms_to_iso8601(cursor_end_ms)?
            ),
            UpbitGranularity::Days => format!(
                "https://api.upbit.com/v1/candles/days?market={}&to={}&count={count}",
                request.symbol,
                timestamp_ms_to_iso8601(cursor_end_ms)?
            ),
        };
        windows.push(ProviderRequestWindow {
            url,
            raw_file_name: format!("request_{index:06}.json"),
        });
        remaining = remaining.saturating_sub(count as u64);
        if remaining == 0 {
            break;
        }
        let window_span_ms = (count as u64)
            .checked_mul(step_ms)
            .ok_or_else(|| "provider window span overflow".to_string())?;
        cursor_end_ms = cursor_end_ms
            .checked_sub(window_span_ms)
            .ok_or_else(|| "provider request planning underflow".to_string())?;
        if cursor_end_ms < adjusted_start {
            break;
        }
        index += 1;
    }
    if windows.len() > request.collection_size_policy.max_requests_per_run {
        return Err(
            "CollectionBudgetExceeded: request count exceeds max_requests_per_run".to_string(),
        );
    }
    Ok(RequestPlan {
        windows,
        planned_row_budget: row_budget,
        truncated,
        row_limit_applied,
        full_history_requested: false,
    })
}

fn plan_krx_windows(
    request: &CandleFetchRequest,
    _provider: &MarketDataProvider,
    auth_token: Option<&str>,
    venue: MarketVenue,
) -> Result<RequestPlan, String> {
    let start = request
        .start_timestamp_ms
        .ok_or_else(|| "krx collection requires start timestamp".to_string())?;
    let end = request
        .end_timestamp_ms
        .ok_or_else(|| "krx collection requires end timestamp".to_string())?;
    let requested_days = days_spanned(start, end);
    if requested_days > request.collection_size_policy.max_days_per_run
        && !request.allow_full_history_override
    {
        return Err("FullHistoryDenied: requested day span exceeds max_days_per_run".to_string());
    }
    let template = request.endpoint_template.as_deref().unwrap_or(
        "https://fixture.krx.local/daily?symbol={symbol}&venue={venue}&from={start_yyyymmdd}&to={end_yyyymmdd}&api_key={api_key}",
    );
    let token = auth_token.unwrap_or("MOCK");
    let url = template
        .replace("{symbol}", &request.symbol)
        .replace("{venue}", venue_slug(venue))
        .replace("{start_yyyymmdd}", &timestamp_ms_to_yyyymmdd(start)?)
        .replace("{end_yyyymmdd}", &timestamp_ms_to_yyyymmdd(end)?)
        .replace("{api_key}", token);
    Ok(RequestPlan {
        windows: vec![ProviderRequestWindow {
            url,
            raw_file_name: "request_000001.json".to_string(),
        }],
        planned_row_budget: request.collection_size_policy.max_effective_rows(),
        truncated: false,
        row_limit_applied: false,
        full_history_requested: false,
    })
}

fn plan_alphavantage_windows(
    request: &CandleFetchRequest,
    _provider: &MarketDataProvider,
    auth_token: Option<&str>,
    full_history_requested: bool,
) -> Result<RequestPlan, String> {
    let requested_output_size = resolve_requested_output_size(request, full_history_requested)?;
    let token = auth_token.unwrap_or("MOCK");
    let function = match request.timeframe {
        Timeframe::OneDay => "TIME_SERIES_DAILY",
        Timeframe::OneMinute => "TIME_SERIES_INTRADAY",
        _ => return Err("AlphaVantage supports only 1d and 1m in Sprint 19".to_string()),
    };
    let outputsize = match requested_output_size {
        RequestedOutputSize::Compact => "compact",
        RequestedOutputSize::Full => "full",
    };
    let interval = if request.timeframe == Timeframe::OneMinute {
        "&interval=1min"
    } else {
        ""
    };
    let url = format!(
        "https://www.alphavantage.co/query?function={function}&symbol={}&outputsize={outputsize}&datatype=json{interval}&apikey={token}",
        request.symbol
    );
    Ok(RequestPlan {
        windows: vec![ProviderRequestWindow {
            url,
            raw_file_name: "request_000001.json".to_string(),
        }],
        planned_row_budget: request.collection_size_policy.max_effective_rows(),
        truncated: false,
        row_limit_applied: false,
        full_history_requested,
    })
}

fn resolve_requested_output_size(
    request: &CandleFetchRequest,
    full_history_requested: bool,
) -> Result<RequestedOutputSize, String> {
    if let Some(size) = request.requested_output_size {
        match size {
            RequestedOutputSize::Compact => return Ok(size),
            RequestedOutputSize::Full => {
                if !request.collection_size_policy.allow_full_history
                    && !request.allow_full_history_override
                {
                    return Err(
                        "FullHistoryDenied: full outputsize requires explicit override".to_string(),
                    );
                }
                return Ok(size);
            }
        }
    }
    match request.collection_size_policy.default_outputsize {
        CollectionOutputSize::Compact => Ok(RequestedOutputSize::Compact),
        CollectionOutputSize::FullDisallowed => {
            if full_history_requested {
                Err("FullHistoryDenied: policy disallows full outputsize".to_string())
            } else {
                Ok(RequestedOutputSize::Compact)
            }
        }
        CollectionOutputSize::FullAllowedOnlyWithExplicitFlag => {
            if full_history_requested {
                Ok(RequestedOutputSize::Full)
            } else {
                Ok(RequestedOutputSize::Compact)
            }
        }
    }
}

fn fetch_with_retry(
    client: &dyn MarketDataHttpClient,
    provider: &MarketDataProvider,
    url: &str,
    reason_codes: &mut Vec<ReasonCode>,
) -> Result<String, String> {
    let mut attempts = 0usize;
    loop {
        attempts += 1;
        match client.get(url) {
            Ok(body) => return Ok(body),
            Err(HttpClientError::Transient(err))
                if attempts <= provider.rate_limit.max_retries + 1 =>
            {
                if attempts <= provider.rate_limit.max_retries {
                    reason_codes.push(ReasonCode::ProviderRequestRetried);
                    continue;
                }
                reason_codes.push(ReasonCode::ProviderRequestFailed);
                return Err(err);
            }
            Err(HttpClientError::Transient(err)) | Err(HttpClientError::Permanent(err)) => {
                reason_codes.push(ReasonCode::ProviderRequestFailed);
                return Err(err);
            }
        }
    }
}

fn archive_raw_response(
    request: &CandleFetchRequest,
    url: &str,
    raw_file_name: &str,
    body: &str,
    collection: &CanonicalCollection,
    raw_bytes_written: &mut usize,
    reason_codes: &mut Vec<ReasonCode>,
) -> Result<Option<PathBuf>, String> {
    if !raw_archive_enabled(request) {
        reason_codes.push(ReasonCode::RawArchiveDisabled);
        return Ok(None);
    }
    let raw_text = match request.collection_size_policy.raw_archive_policy {
        RawArchivePolicy::None => {
            reason_codes.push(ReasonCode::RawArchiveDisabled);
            return Ok(None);
        }
        RawArchivePolicy::HeadersOnly => serde_json::to_string_pretty(&serde_json::json!({
            "request_url": url,
            "body_bytes": body.len(),
        }))
        .map_err(|err| err.to_string())?,
        RawArchivePolicy::CompactJson => compact_json(body),
        RawArchivePolicy::FullRawAllowedOnlyWithExplicitFlag => {
            if !request.allow_full_history_override {
                reason_codes.push(ReasonCode::RawArchiveDisabled);
                return Ok(None);
            }
            body.to_string()
        }
    };
    let projected = raw_bytes_written.saturating_add(raw_text.len());
    if projected > request.collection_size_policy.max_raw_bytes_per_run {
        reason_codes.push(ReasonCode::RawArchiveBudgetExceeded);
        reason_codes.push(ReasonCode::RawArchiveDisabled);
        return Ok(None);
    }
    let raw_path = collection.raw_dir.join(raw_file_name);
    fs::write(&raw_path, raw_text).map_err(|err| err.to_string())?;
    *raw_bytes_written = projected;
    reason_codes.push(ReasonCode::ProviderResponseArchived);
    Ok(Some(raw_path))
}

fn parse_provider_body(provider_kind: ProviderKind, body: &str) -> Result<Vec<Candle>, String> {
    match provider_kind {
        ProviderKind::Upbit => parse_upbit_like_body(body),
        ProviderKind::KrxOpenApi => parse_krx_daily_body(body),
        ProviderKind::AlphaVantage => parse_alphavantage_body(body),
        ProviderKind::MockFixture => parse_mock_fixture_body(body),
        ProviderKind::Alpaca => Err("AlpacaProviderDeferred: parser not enabled".to_string()),
        _ => Err("provider parser is not implemented".to_string()),
    }
}

fn parse_upbit_like_body(body: &str) -> Result<Vec<Candle>, String> {
    let responses =
        serde_json::from_str::<Vec<UpbitCandleResponse>>(body).map_err(|err| err.to_string())?;
    Ok(responses
        .into_iter()
        .map(|item| Candle {
            timestamp_ms: item.timestamp,
            open: item.opening_price,
            high: item.high_price,
            low: item.low_price,
            close: item.trade_price,
            volume: item.candle_acc_trade_volume,
            trade_value: Some(item.candle_acc_trade_price),
            bid: None,
            ask: None,
            spread_bps: None,
        })
        .collect())
}

fn parse_krx_daily_body(body: &str) -> Result<Vec<Candle>, String> {
    let value = serde_json::from_str::<serde_json::Value>(body).map_err(|err| err.to_string())?;
    let rows = value
        .get("rows")
        .or_else(|| value.get("output"))
        .and_then(|value| value.as_array())
        .ok_or_else(|| "krx fixture must contain rows/output array".to_string())?;
    let mut candles = Vec::new();
    for row in rows {
        let date = first_string(row, &["date", "basDt", "trdDd"])?;
        candles.push(Candle {
            timestamp_ms: parse_date_to_timestamp_ms(&date)?,
            open: first_number(row, &["open", "tddOpnprc", "stck_oprc"])?,
            high: first_number(row, &["high", "tddHgprc", "stck_hgpr"])?,
            low: first_number(row, &["low", "tddLwprc", "stck_lwpr"])?,
            close: first_number(row, &["close", "tddClsprc", "stck_clpr"])?,
            volume: first_number(row, &["volume", "accTrdvol", "acml_vol"])?,
            trade_value: first_number_opt(row, &["trade_value", "accTrdval", "acml_tr_pbmn"]),
            bid: None,
            ask: None,
            spread_bps: None,
        });
    }
    Ok(candles)
}

fn parse_alphavantage_body(body: &str) -> Result<Vec<Candle>, String> {
    let value = serde_json::from_str::<serde_json::Value>(body).map_err(|err| err.to_string())?;
    let series = value
        .get("Time Series (Daily)")
        .or_else(|| value.get("Time Series (1min)"))
        .and_then(|value| value.as_object())
        .ok_or_else(|| "alphavantage response missing expected time-series object".to_string())?;
    let mut candles = Vec::new();
    for (date, row) in series {
        candles.push(Candle {
            timestamp_ms: parse_date_to_timestamp_ms(date)?,
            open: first_number(row, &["1. open"])?,
            high: first_number(row, &["2. high"])?,
            low: first_number(row, &["3. low"])?,
            close: first_number(row, &["4. close"])?,
            volume: first_number(row, &["5. volume"])?,
            trade_value: None,
            bid: None,
            ask: None,
            spread_bps: None,
        });
    }
    Ok(candles)
}

fn parse_mock_fixture_body(body: &str) -> Result<Vec<Candle>, String> {
    parse_upbit_like_body(body)
        .or_else(|_| parse_alphavantage_body(body))
        .or_else(|_| parse_krx_daily_body(body))
}

fn normalize_provider_candles(
    candles: Vec<Candle>,
    start_timestamp_ms: Option<u64>,
    end_timestamp_ms: Option<u64>,
) -> Vec<Candle> {
    let mut merged = BTreeMap::new();
    for candle in candles {
        if start_timestamp_ms.is_some_and(|start| candle.timestamp_ms < start)
            || end_timestamp_ms.is_some_and(|end| candle.timestamp_ms > end)
        {
            continue;
        }
        merged.insert(candle.timestamp_ms, candle);
    }
    merged.into_values().collect()
}

fn apply_fill_missing_policy(
    candles: &mut Vec<Candle>,
    step_ms: u64,
    policy: FillMissingPolicy,
) -> Result<Vec<ReasonCode>, String> {
    if candles.is_empty() || step_ms == 0 {
        return Ok(Vec::new());
    }
    let mut reason_codes = Vec::new();
    let mut has_gap = false;
    for pair in candles.windows(2) {
        if pair[1].timestamp_ms.saturating_sub(pair[0].timestamp_ms) > step_ms {
            has_gap = true;
            break;
        }
    }
    if !has_gap {
        return Ok(reason_codes);
    }
    reason_codes.push(ReasonCode::GapDetected);
    match policy {
        FillMissingPolicy::LeaveGaps => Ok(reason_codes),
        FillMissingPolicy::RejectIfGaps => Ok(reason_codes),
        FillMissingPolicy::InsertEmptyRows => {
            let mut filled = Vec::with_capacity(candles.len());
            for window in candles.windows(2) {
                let current = window[0].clone();
                let next = &window[1];
                filled.push(current.clone());
                let mut timestamp = current.timestamp_ms.saturating_add(step_ms);
                while timestamp < next.timestamp_ms {
                    filled.push(Candle {
                        timestamp_ms: timestamp,
                        open: current.close,
                        high: current.close,
                        low: current.close,
                        close: current.close,
                        volume: 0.0,
                        trade_value: Some(0.0),
                        bid: None,
                        ask: None,
                        spread_bps: None,
                    });
                    timestamp = timestamp.saturating_add(step_ms);
                }
            }
            if let Some(last) = candles.last().cloned() {
                filled.push(last);
            }
            *candles = filled;
            Ok(reason_codes)
        }
    }
}

fn render_canonical_csv(candles: &[Candle]) -> String {
    let mut contents =
        "timestamp_ms,open,high,low,close,volume,trade_value,bid,ask,spread_bps\n".to_string();
    for candle in candles {
        contents.push_str(&format!(
            "{},{:.8},{:.8},{:.8},{:.8},{:.8},{},,,\n",
            candle.timestamp_ms,
            candle.open,
            candle.high,
            candle.low,
            candle.close,
            candle.volume,
            format_optional_decimal(candle.trade_value)
        ));
    }
    contents
}

fn collected_provenance(
    request: &CandleFetchRequest,
    provider: &MarketDataProvider,
    canonical_csv_path: &Path,
    venue: MarketVenue,
    reason_codes: Vec<ReasonCode>,
) -> DataProvenance {
    let source_kind = match provider.source_kind {
        CollectorSourceKind::OfficialPublicApi
        | CollectorSourceKind::OfficialAuthenticatedMarketDataApi => {
            EvidenceSourceKind::OfficialApiCollected
        }
        CollectorSourceKind::FixtureReplay | CollectorSourceKind::Unknown => {
            EvidenceSourceKind::TestFixture
        }
    };
    DataProvenance {
        source_kind,
        source_label: format!(
            "{}/{}/{}/{}",
            provider.provider_id,
            venue_slug(venue),
            request.symbol,
            timeframe_slug(request.timeframe)
        ),
        provider_label: Some(provider.provider_id.clone()),
        upstream_label: Some(format!("{:?}", provider.kind)),
        local_path: Some(canonical_csv_path.display().to_string()),
        generated_by: Some("soma_experiment.collect-candles".to_string()),
        user_supplied: false,
        downloaded_by_soma: source_kind == EvidenceSourceKind::OfficialApiCollected,
        remote_url_present: false,
        official_provider: Some(source_kind == EvidenceSourceKind::OfficialApiCollected),
        affiliated_or_endorsed: Some(source_kind == EvidenceSourceKind::OfficialApiCollected),
        intended_use: Some("official bounded collection research".to_string()),
        readiness_eligible: Some(source_kind.readiness_eligible()),
        benchmark_eligible: Some(true),
        license_note: Some(
            "Collected market data remains research-only; verify provider licensing before redistribution."
                .to_string(),
        ),
        notes: Some(format!(
            "Canonical OHLCV normalized with {:?} adjusted-price policy and bounded collection policy.",
            request.adjusted_price_policy
        )),
        reason_codes: dedupe_reasons(reason_codes),
    }
}

fn client_from_fixture_path(path: &Path) -> Result<Box<dyn MarketDataHttpClient>, String> {
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let base_dir = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    if serde_json::from_str::<GenericHttpFixture>(&text).is_ok() {
        Ok(Box::new(FixtureHttpClient::from_text(base_dir, &text)?))
    } else {
        Ok(Box::new(StaticBodyHttpClient::new(text)))
    }
}

fn raw_archive_enabled(request: &CandleFetchRequest) -> bool {
    request.include_raw_archive
        && request.collection_size_policy.raw_archive_policy != RawArchivePolicy::None
}

fn apply_retention_policy(dir: &Path, policy: RetentionPolicy) -> Result<(), String> {
    let keep = match policy {
        RetentionPolicy::KeepLatestOnly => Some(1usize),
        RetentionPolicy::KeepLastNFiles(count) => Some(count.max(1)),
        RetentionPolicy::KeepAllWithinBudget
        | RetentionPolicy::DeleteRawAfterCanonicalAndManifest
        | RetentionPolicy::ArchiveCompressedRawOnly => None,
    };
    let Some(keep) = keep else {
        return Ok(());
    };
    if !dir.exists() {
        return Ok(());
    }
    let mut entries = fs::read_dir(dir)
        .map_err(|err| err.to_string())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_type()
                .map(|kind| kind.is_file())
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());
    let remove_count = entries.len().saturating_sub(keep);
    for entry in entries.into_iter().take(remove_count) {
        fs::remove_file(entry.path()).map_err(|err| err.to_string())?;
    }
    Ok(())
}

fn compact_json(body: &str) -> String {
    serde_json::from_str::<serde_json::Value>(body)
        .and_then(|value| serde_json::to_string(&value))
        .unwrap_or_else(|_| body.to_string())
}

fn venue_slug(venue: MarketVenue) -> &'static str {
    match venue {
        MarketVenue::Generic => "generic",
        MarketVenue::Binance => "binance",
        MarketVenue::Upbit => "upbit",
        MarketVenue::KRX => "krx",
        MarketVenue::KOSPI => "kospi",
        MarketVenue::KOSDAQ => "kosdaq",
        MarketVenue::NASDAQ => "nasdaq",
        MarketVenue::NYSE => "nyse",
        MarketVenue::AMEX => "amex",
        MarketVenue::US => "us",
        MarketVenue::Unknown => "unknown",
    }
}

fn timeframe_slug(timeframe: Timeframe) -> &'static str {
    match timeframe {
        Timeframe::OneMinute => "1m",
        Timeframe::FiveMinute => "5m",
        Timeframe::FifteenMinute => "15m",
        Timeframe::OneHour => "1h",
        Timeframe::OneDay => "1d",
        Timeframe::Custom { .. } => "custom",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UpbitGranularity {
    Minutes(u32),
    Days,
}

fn upbit_granularity(timeframe: Timeframe) -> Result<UpbitGranularity, String> {
    match timeframe {
        Timeframe::OneMinute => Ok(UpbitGranularity::Minutes(1)),
        Timeframe::FiveMinute => Ok(UpbitGranularity::Minutes(5)),
        Timeframe::FifteenMinute => Ok(UpbitGranularity::Minutes(15)),
        Timeframe::OneHour => Ok(UpbitGranularity::Minutes(60)),
        Timeframe::OneDay => Ok(UpbitGranularity::Days),
        Timeframe::Custom { .. } => {
            Err("upbit collector does not support custom timeframe".to_string())
        }
    }
}

fn timestamp_ms_to_iso8601(timestamp_ms: u64) -> Result<String, String> {
    let seconds = timestamp_ms / 1_000;
    let days = i64::try_from(seconds / 86_400).map_err(|_| "timestamp overflow".to_string())?;
    let secs_of_day = seconds % 86_400;
    let (year, month, day) = civil_from_days(days);
    let hour = secs_of_day / 3_600;
    let minute = (secs_of_day % 3_600) / 60;
    let second = secs_of_day % 60;
    Ok(format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z"
    ))
}

fn timestamp_ms_to_yyyymmdd(timestamp_ms: u64) -> Result<String, String> {
    let seconds = timestamp_ms / 1_000;
    let days = i64::try_from(seconds / 86_400).map_err(|_| "timestamp overflow".to_string())?;
    let (year, month, day) = civil_from_days(days);
    Ok(format!("{year:04}{month:02}{day:02}"))
}

fn parse_date_to_timestamp_ms(value: &str) -> Result<u64, String> {
    if value.len() >= 10 && value.as_bytes().get(4) == Some(&b'-') {
        let year = value[0..4].parse::<i32>().map_err(|err| err.to_string())?;
        let month = value[5..7].parse::<u32>().map_err(|err| err.to_string())?;
        let day = value[8..10].parse::<u32>().map_err(|err| err.to_string())?;
        return civil_to_timestamp_ms(year, month, day);
    }
    if value.len() == 8 && value.chars().all(|ch| ch.is_ascii_digit()) {
        let year = value[0..4].parse::<i32>().map_err(|err| err.to_string())?;
        let month = value[4..6].parse::<u32>().map_err(|err| err.to_string())?;
        let day = value[6..8].parse::<u32>().map_err(|err| err.to_string())?;
        return civil_to_timestamp_ms(year, month, day);
    }
    Err("unsupported date format".to_string())
}

fn civil_to_timestamp_ms(year: i32, month: u32, day: u32) -> Result<u64, String> {
    let days = days_from_civil(year, month, day);
    let millis = days
        .checked_mul(86_400_000)
        .ok_or_else(|| "timestamp overflow".to_string())?;
    u64::try_from(millis).map_err(|_| "timestamp overflow".to_string())
}

fn days_spanned(start_timestamp_ms: u64, end_timestamp_ms: u64) -> usize {
    (((end_timestamp_ms.saturating_sub(start_timestamp_ms)) / 86_400_000).saturating_add(1))
        as usize
}

fn first_string(value: &serde_json::Value, keys: &[&str]) -> Result<String, String> {
    for key in keys {
        if let Some(text) = value.get(*key).and_then(|value| value.as_str()) {
            return Ok(text.to_string());
        }
    }
    Err(format!("missing required string field: {}", keys.join("/")))
}

fn first_number(value: &serde_json::Value, keys: &[&str]) -> Result<f64, String> {
    first_number_opt(value, keys)
        .ok_or_else(|| format!("missing required numeric field: {}", keys.join("/")))
}

fn first_number_opt(value: &serde_json::Value, keys: &[&str]) -> Option<f64> {
    for key in keys {
        if let Some(raw) = value.get(*key) {
            if let Some(number) = raw.as_f64() {
                return Some(number);
            }
            if let Some(text) = raw.as_str() {
                let cleaned = text.replace(',', "");
                if let Ok(number) = cleaned.parse::<f64>() {
                    return Some(number);
                }
            }
        }
    }
    None
}

fn format_optional_decimal(value: Option<f64>) -> String {
    value.map(|value| format!("{value:.8}")).unwrap_or_default()
}

fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + i64::from(month <= 2);
    (year as i32, month as u32, day as u32)
}

fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let year = year - i32::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let mp = month as i32 + if month > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + day as i32 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    i64::from(era * 146_097 + doe - 719_468)
}

fn dedupe_reasons(values: Vec<ReasonCode>) -> Vec<ReasonCode> {
    let mut deduped = Vec::new();
    for value in values {
        if !deduped.contains(&value) {
            deduped.push(value);
        }
    }
    deduped
}

fn default_true() -> bool {
    true
}

fn default_adjusted_price_policy() -> AdjustedPricePolicy {
    AdjustedPricePolicy::Raw
}
