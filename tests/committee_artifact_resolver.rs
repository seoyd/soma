mod common;

use soma_zero::{CommitteeArtifactKind, CommitteeArtifactResolver};

#[test]
fn artifact_resolver_detects_known_artifacts_and_is_deterministic() {
    let fixture = common::output_dir("artifact-resolver-fixture").join("fixture_rows.json");
    let yfinance = common::output_dir("artifact-resolver-yfinance").join("yfinance_report.json");
    let evidence =
        common::output_dir("artifact-resolver-evidence").join("evidence_lane_report.json");
    let core = common::output_dir("artifact-resolver-core").join("core_benchmark_report.json");
    let csv = common::output_dir("artifact-resolver-csv").join("aapl_1d.csv");
    let unknown = common::output_dir("artifact-resolver-unknown").join("mystery.bin");
    std::fs::write(&fixture, r#"{"rows":[{"symbol":"BTC-KRW"}]}"#).expect("write fixture");
    std::fs::write(&yfinance, r#"{"yfinance_symbols":["AAPL"]}"#).expect("write yfinance");
    std::fs::write(&evidence, r#"{"lane_reports":[{"symbol":"AAPL"}]}"#).expect("write evidence");
    std::fs::write(&core, r#"{"final_status":"ok","official":"true"}"#).expect("write core");
    std::fs::write(&csv, "timestamp,open,high,low,close,volume\n1,1,1,1,1,1\n").expect("write csv");
    std::fs::write(&unknown, "not-json").expect("write unknown");
    let resolver = CommitteeArtifactResolver;
    assert_eq!(
        resolver
            .resolve(&fixture.display().to_string())
            .artifact_kind,
        CommitteeArtifactKind::FixtureScenario
    );
    assert_eq!(
        resolver
            .resolve(&yfinance.display().to_string())
            .artifact_kind,
        CommitteeArtifactKind::YahooResearchEvidenceReport
    );
    assert_eq!(
        resolver
            .resolve(&evidence.display().to_string())
            .artifact_kind,
        CommitteeArtifactKind::EvidenceLaneReport
    );
    assert_eq!(
        resolver.resolve(&core.display().to_string()).artifact_kind,
        CommitteeArtifactKind::CoreCheckedBenchmarkReport
    );
    assert_eq!(
        resolver.resolve(&csv.display().to_string()).artifact_kind,
        CommitteeArtifactKind::CanonicalOhlcvCsv
    );
    let first = resolver.resolve(&unknown.display().to_string());
    let second = resolver.resolve(&unknown.display().to_string());
    assert_eq!(first.artifact_kind, CommitteeArtifactKind::Unknown);
    assert_eq!(first, second);
}
