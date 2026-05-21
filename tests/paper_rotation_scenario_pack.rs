mod support;

use soma_zero::{
    PaperRotationMarketCoverage, PaperRotationRegimeCoverage, PaperRotationScenarioPackStatus,
};
use support::sprint102_support::run_sprint102;

#[test]
fn paper_rotation_scenario_pack_and_context_are_ready() {
    let bundle = run_sprint102(
        "soma_paper_rotation_scenario_pack.toml",
        "sprint102-scenario-pack",
    );
    assert_eq!(
        bundle.paper_rotation_scenario_pack.pack_status,
        PaperRotationScenarioPackStatus::ScenarioPackReady
    );
    assert_eq!(bundle.paper_rotation_scenario_pack.scenario_count, 7);
    assert!(
        bundle
            .paper_rotation_scenario_pack
            .market_coverage
            .contains(&PaperRotationMarketCoverage::KoreanEquity)
    );
    assert!(
        bundle
            .paper_rotation_scenario_pack
            .market_coverage
            .contains(&PaperRotationMarketCoverage::USEquity)
    );
    assert!(
        bundle
            .paper_rotation_scenario_pack
            .market_coverage
            .contains(&PaperRotationMarketCoverage::Crypto)
    );
    assert!(
        bundle
            .paper_rotation_scenario_pack
            .regime_coverage
            .contains(&PaperRotationRegimeCoverage::HighVolatility)
    );
    assert!(
        bundle
            .paper_rotation_scenario_pack
            .regime_coverage
            .contains(&PaperRotationRegimeCoverage::InsufficientEvidence)
    );
    let context = &bundle.paper_rotation_market_context_set;
    assert_eq!(
        context.context_count,
        bundle.paper_rotation_scenario_pack.scenario_count
    );
    assert_eq!(context.contexts_with_source_boundary, context.context_count);
    assert_eq!(
        context.contexts_with_no_lookahead_proof,
        context.context_count
    );
    assert_eq!(
        context.contexts[0],
        bundle.paper_rotation_market_context_set.contexts[0]
    );
}
