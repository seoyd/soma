use soma_zero::{
    CandidateEvidenceClass, CandidateGenerationFromEvidence, CandidateGenerationInput,
    CandidateGenerationSettings, CandidateGenerationStatus, CandidateSourceKind,
};
use soma_zero::{ProviderMarket, Regime};

#[test]
fn candidate_generation_preserves_source_boundaries_and_is_deterministic() {
    let inputs = vec![
        CandidateGenerationInput {
            source_kind: CandidateSourceKind::KISOfficialEvidence,
            source_report_path: "official.json".to_string(),
            symbol: "005930".to_string(),
            market: ProviderMarket::KoreanEquity,
            timeframe: "1d".to_string(),
            horizon_bars: 20,
            evidence_status: "ready".to_string(),
            data_quality_score: Some(0.95),
            outcome_link_count: Some(3),
            counterfactual_count: Some(2),
            signal_summary: Some("official".to_string()),
            expected_edge: Some(0.03),
            expected_drawdown: Some(0.01),
            timestamp_ms: 1,
            confidence: Some(0.8),
            spread_bps: Some(3.0),
            trade_value: Some(500000.0),
            regime: Some(Regime::TrendUp),
            paper_outcome_hint: None,
            reason_codes: vec![],
        },
        CandidateGenerationInput {
            source_kind: CandidateSourceKind::ResearchOnly,
            source_report_path: "research.json".to_string(),
            symbol: "AAPL".to_string(),
            market: ProviderMarket::USEquity,
            timeframe: "1d".to_string(),
            horizon_bars: 16,
            evidence_status: "ready".to_string(),
            data_quality_score: Some(0.85),
            outcome_link_count: Some(1),
            counterfactual_count: Some(1),
            signal_summary: Some("research".to_string()),
            expected_edge: Some(0.01),
            expected_drawdown: Some(0.02),
            timestamp_ms: 2,
            confidence: Some(0.65),
            spread_bps: Some(4.0),
            trade_value: Some(500000.0),
            regime: Some(Regime::RiskOn),
            paper_outcome_hint: None,
            reason_codes: vec![],
        },
        CandidateGenerationInput {
            source_kind: CandidateSourceKind::DiagnosticOnly,
            source_report_path: "fixture.json".to_string(),
            symbol: "FIXTURE1".to_string(),
            market: ProviderMarket::GlobalEquity,
            timeframe: "1d".to_string(),
            horizon_bars: 10,
            evidence_status: "ready".to_string(),
            data_quality_score: Some(0.9),
            outcome_link_count: Some(1),
            counterfactual_count: Some(1),
            signal_summary: Some("diagnostic".to_string()),
            expected_edge: Some(0.01),
            expected_drawdown: Some(0.02),
            timestamp_ms: 3,
            confidence: Some(0.6),
            spread_bps: Some(5.0),
            trade_value: Some(500000.0),
            regime: Some(Regime::Range),
            paper_outcome_hint: None,
            reason_codes: vec![],
        },
        CandidateGenerationInput {
            source_kind: CandidateSourceKind::CryptoOnly,
            source_report_path: "crypto.json".to_string(),
            symbol: "BTCUSDT".to_string(),
            market: ProviderMarket::Crypto,
            timeframe: "4h".to_string(),
            horizon_bars: 12,
            evidence_status: "ready".to_string(),
            data_quality_score: Some(0.9),
            outcome_link_count: Some(1),
            counterfactual_count: Some(1),
            signal_summary: Some("crypto".to_string()),
            expected_edge: Some(0.02),
            expected_drawdown: Some(0.03),
            timestamp_ms: 4,
            confidence: Some(0.7),
            spread_bps: Some(6.0),
            trade_value: Some(500000.0),
            regime: Some(Regime::RiskOn),
            paper_outcome_hint: None,
            reason_codes: vec![],
        },
        CandidateGenerationInput {
            source_kind: CandidateSourceKind::KISOfficialEvidence,
            source_report_path: "weak.json".to_string(),
            symbol: "000660".to_string(),
            market: ProviderMarket::KoreanEquity,
            timeframe: "1d".to_string(),
            horizon_bars: 20,
            evidence_status: "need_more_evidence".to_string(),
            data_quality_score: Some(0.5),
            outcome_link_count: Some(0),
            counterfactual_count: Some(0),
            signal_summary: Some("weak".to_string()),
            expected_edge: Some(0.001),
            expected_drawdown: Some(0.03),
            timestamp_ms: 5,
            confidence: Some(0.5),
            spread_bps: Some(5.0),
            trade_value: Some(100000.0),
            regime: Some(Regime::Range),
            paper_outcome_hint: None,
            reason_codes: vec![],
        },
    ];

    let settings = CandidateGenerationSettings {
        require_official_evidence_for_official_candidates: true,
        allow_research_only_candidates: true,
        allow_diagnostic_candidates: true,
        allow_crypto_only_candidates: true,
    };
    let report = CandidateGenerationFromEvidence::default().generate(&inputs, &settings);
    assert_eq!(
        report.generation_status,
        CandidateGenerationStatus::CandidatesGenerated
    );
    assert_eq!(report.official_candidates, 1);
    assert_eq!(report.research_only_candidates, 1);
    assert_eq!(report.diagnostic_candidates, 1);
    assert_eq!(report.crypto_candidates, 1);
    assert_eq!(report.skipped_candidates.len(), 1);
    assert!(
        report
            .generated_candidates
            .iter()
            .any(|candidate| candidate.evidence_class == CandidateEvidenceClass::Official)
    );
    assert_eq!(
        report.fingerprint,
        CandidateGenerationFromEvidence::default()
            .generate(&inputs, &settings)
            .fingerprint
    );
}
