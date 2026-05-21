mod common;

use soma_zero::{CsvFormatDetectionConfidence, CsvFormatDetector, CustomColumnMap, ReasonCode};

#[test]
fn generic_ohlcv_header_detected_with_high_confidence() {
    let detector = CsvFormatDetector::default();
    let result = detector
        .detect_from_path(&common::fixture_path("generic_ohlcv_valid.csv"), None)
        .expect("detect generic");
    assert_eq!(result.confidence, CsvFormatDetectionConfidence::High);
    assert_eq!(
        result.detected_format.map(|value| format!("{value:?}")),
        Some("GenericOhlcv".to_string())
    );
}

#[test]
fn binance_upbit_and_krx_like_headers_are_detected() {
    let detector = CsvFormatDetector::default();
    let binance = detector
        .detect_from_path(&common::fixture_path("binance_kline_like.csv"), None)
        .expect("detect binance");
    let upbit = detector
        .detect_from_path(&common::fixture_path("upbit_candle_like.csv"), None)
        .expect("detect upbit");
    let krx = detector
        .detect_from_path(&common::fixture_path("krx_ohlcv_like.csv"), None)
        .expect("detect krx");

    assert_eq!(
        binance.detected_format.map(|value| format!("{value:?}")),
        Some("BinanceKline".to_string())
    );
    assert_eq!(
        upbit.detected_format.map(|value| format!("{value:?}")),
        Some("UpbitCandle".to_string())
    );
    assert_eq!(
        krx.detected_format.map(|value| format!("{value:?}")),
        Some("KrxOhlcv".to_string())
    );
}

#[test]
fn ambiguous_header_returns_ambiguous() {
    let detector = CsvFormatDetector::default();
    let result = detector.detect_from_str("timestamp_ms,open,high,low,close\n1,2,3,4,5\n", None);
    assert_eq!(result.confidence, CsvFormatDetectionConfidence::Ambiguous);
    assert!(
        result
            .reason_codes
            .contains(&ReasonCode::CsvFormatAmbiguous)
    );
}

#[test]
fn custom_column_map_overrides_detection_and_is_deterministic() {
    let detector = CsvFormatDetector::default();
    let input = "ts,o,h,l,c,v\n1,10,12,9,11,100\n2,11,13,10,12,110\n";
    let custom_map = CustomColumnMap {
        timestamp: "ts".to_string(),
        open: "o".to_string(),
        high: "h".to_string(),
        low: "l".to_string(),
        close: "c".to_string(),
        volume: "v".to_string(),
        trade_value: None,
        bid: None,
        ask: None,
        spread_bps: None,
    };
    let result_a = detector.detect_from_str(input, Some(&custom_map));
    let result_b = detector.detect_from_str(input, Some(&custom_map));
    assert_eq!(result_a, result_b);
    assert!(
        result_a
            .reason_codes
            .contains(&ReasonCode::CustomColumnMapApplied)
    );
    assert_eq!(result_a.confidence, CsvFormatDetectionConfidence::High);
}
