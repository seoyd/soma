use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use encoding_rs::EUC_KR;
use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapshotTextEncoding {
    Utf8,
    Cp949,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KrxSnapshotImportConfig {
    pub import_id: String,
    pub input_path: String,
    pub output_root: String,
    #[serde(default)]
    pub snapshot_date: Option<String>,
    #[serde(default)]
    pub symbol_filter: Option<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl KrxSnapshotImportConfig {
    pub fn validate_local_paths(&self) -> Vec<ReasonCode> {
        let mut reasons = self.reason_codes.clone();
        if self.input_path.contains("://") || self.output_root.contains("://") {
            reasons.push(ReasonCode::LocalPathRejected);
        }
        dedupe_reasons(reasons)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KrxSnapshotCanonicalRow {
    pub timestamp_ms: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub trade_value: f64,
    pub symbol: String,
    pub market: String,
    pub name: String,
    pub snapshot_date: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KrxSnapshotSymbolReport {
    pub symbol: String,
    pub name: String,
    pub market: String,
    pub output_path: String,
    pub rows_written: usize,
    pub total_rows_after_merge: usize,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KrxSnapshotImportReport {
    pub import_id: String,
    pub input_path: String,
    pub output_root: String,
    pub decoded_encoding: SnapshotTextEncoding,
    pub snapshot_date: String,
    pub timestamp_ms: u64,
    pub symbol_filter: Option<String>,
    pub parsed_row_count: usize,
    pub imported_symbol_count: usize,
    pub symbol_reports: Vec<KrxSnapshotSymbolReport>,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl KrxSnapshotImportReport {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        let symbol_lines = self
            .symbol_reports
            .iter()
            .map(|report| {
                format!(
                    "{} {} {:?} rows_written={} total_rows_after_merge={} output={}",
                    report.symbol,
                    report.name,
                    report.reason_codes,
                    report.rows_written,
                    report.total_rows_after_merge,
                    report.output_path
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        [
            format!("import_id={}", self.import_id),
            format!("input_path={}", self.input_path),
            format!("output_root={}", self.output_root),
            format!("decoded_encoding={:?}", self.decoded_encoding),
            format!("snapshot_date={}", self.snapshot_date),
            format!("timestamp_ms={}", self.timestamp_ms),
            format!(
                "symbol_filter={}",
                self.symbol_filter.clone().unwrap_or_default()
            ),
            format!("parsed_row_count={}", self.parsed_row_count),
            format!("imported_symbol_count={}", self.imported_symbol_count),
            format!("blockers={}", self.blockers.join(" | ")),
            format!("warnings={}", self.warnings.join(" | ")),
            "symbols:".to_string(),
            symbol_lines,
        ]
        .join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("krx_snapshot_import_report.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("krx_snapshot_import_report.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct KrxSnapshotImporter;

impl KrxSnapshotImporter {
    pub fn import(
        &self,
        config: &KrxSnapshotImportConfig,
    ) -> Result<KrxSnapshotImportReport, String> {
        let invalid_paths = config.validate_local_paths();
        if invalid_paths.contains(&ReasonCode::LocalPathRejected) {
            return Err("krx-snapshot-import paths must be local".to_string());
        }
        let input = Path::new(&config.input_path);
        if !input.exists() {
            return Err("krx snapshot input file does not exist".to_string());
        }
        let (text, decoded_encoding) = decode_snapshot_text(input)?;
        let snapshot_date = config
            .snapshot_date
            .clone()
            .or_else(|| infer_snapshot_date_from_path(input))
            .ok_or_else(|| {
                "could not infer snapshot date from path; pass --date YYYYMMDD".to_string()
            })?;
        validate_snapshot_date(&snapshot_date)?;
        let timestamp_ms = yyyymmdd_to_unix_ms(&snapshot_date)?;
        let parsed = parse_snapshot_rows(&text, &snapshot_date, timestamp_ms)?;
        let filtered = parsed
            .into_iter()
            .filter(|row| {
                config
                    .symbol_filter
                    .as_ref()
                    .map(|symbol| normalize_symbol(symbol) == row.symbol)
                    .unwrap_or(true)
            })
            .collect::<Vec<_>>();
        fs::create_dir_all(&config.output_root).map_err(|err| err.to_string())?;
        let mut symbol_reports = Vec::new();
        for row in &filtered {
            symbol_reports.push(write_canonical_row(Path::new(&config.output_root), row)?);
        }
        let mut reason_codes = vec![ReasonCode::KrxSnapshotImported];
        if decoded_encoding == SnapshotTextEncoding::Cp949 {
            reason_codes.push(ReasonCode::SnapshotEncodingFallback);
        }
        if config.snapshot_date.is_none() {
            reason_codes.push(ReasonCode::SnapshotDateInferred);
        }
        if config.symbol_filter.is_some() {
            reason_codes.push(ReasonCode::SnapshotSymbolFiltered);
        }
        let report = KrxSnapshotImportReport {
            import_id: config.import_id.clone(),
            input_path: config.input_path.clone(),
            output_root: config.output_root.clone(),
            decoded_encoding,
            snapshot_date,
            timestamp_ms,
            symbol_filter: config.symbol_filter.as_deref().map(normalize_symbol),
            parsed_row_count: filtered.len(),
            imported_symbol_count: symbol_reports.len(),
            symbol_reports,
            blockers: Vec::new(),
            warnings: if filtered.is_empty() {
                vec!["no symbols matched snapshot import filter".to_string()]
            } else {
                Vec::new()
            },
            reason_codes: dedupe_reasons([config.reason_codes.clone(), reason_codes].concat()),
        };
        report.write_to_dir(Path::new(&config.output_root))?;
        Ok(report)
    }
}

fn decode_snapshot_text(path: &Path) -> Result<(String, SnapshotTextEncoding), String> {
    let bytes = fs::read(path).map_err(|err| err.to_string())?;
    if let Ok(text) = String::from_utf8(bytes.clone()) {
        return Ok((text, SnapshotTextEncoding::Utf8));
    }
    let (decoded, _, had_errors) = EUC_KR.decode(&bytes);
    if had_errors {
        Err("snapshot file is neither valid utf-8 nor cp949/euc-kr".to_string())
    } else {
        Ok((decoded.into_owned(), SnapshotTextEncoding::Cp949))
    }
}

fn parse_snapshot_rows(
    text: &str,
    snapshot_date: &str,
    timestamp_ms: u64,
) -> Result<Vec<KrxSnapshotCanonicalRow>, String> {
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let header_line = lines
        .next()
        .ok_or_else(|| "snapshot file is empty".to_string())?;
    let header = parse_csv_record(header_line);
    let header_index = header
        .iter()
        .enumerate()
        .map(|(index, value)| (normalize_header(value), index))
        .collect::<BTreeMap<_, _>>();
    let required = required_snapshot_columns();
    for column in required {
        if !header_index.contains_key(column) {
            return Err(format!("snapshot file missing required column: {column}"));
        }
    }
    let mut rows = Vec::new();
    for line in lines {
        let fields = parse_csv_record(line);
        if fields.iter().all(|value| value.trim().is_empty()) {
            continue;
        }
        let symbol = normalize_symbol(field(&fields, &header_index, "종목코드")?);
        let name = field(&fields, &header_index, "종목명")?.to_string();
        let market = field(&fields, &header_index, "시장구분")?.to_string();
        rows.push(KrxSnapshotCanonicalRow {
            timestamp_ms,
            open: parse_krx_number(field(&fields, &header_index, "시가")?)?,
            high: parse_krx_number(field(&fields, &header_index, "고가")?)?,
            low: parse_krx_number(field(&fields, &header_index, "저가")?)?,
            close: parse_krx_number(field(&fields, &header_index, "종가")?)?,
            volume: parse_krx_number(field(&fields, &header_index, "거래량")?)?,
            trade_value: parse_krx_number(field(&fields, &header_index, "거래대금")?)?,
            symbol,
            market,
            name,
            snapshot_date: snapshot_date.to_string(),
        });
    }
    Ok(rows)
}

fn write_canonical_row(
    output_root: &Path,
    row: &KrxSnapshotCanonicalRow,
) -> Result<KrxSnapshotSymbolReport, String> {
    let file_path = output_root.join(format!("{}_krx_1d.csv", row.symbol));
    let mut merged = load_existing_canonical_rows(&file_path)?;
    merged.insert(row.timestamp_ms, row.clone());
    let rows = merged.into_values().collect::<Vec<_>>();
    let mut contents =
        "timestamp_ms,open,high,low,close,volume,trade_value,symbol,market,name,snapshot_date\n"
            .to_string();
    for item in &rows {
        contents.push_str(&format!(
            "{},{:.0},{:.0},{:.0},{:.0},{:.0},{:.0},{},{},{},{}\n",
            item.timestamp_ms,
            item.open,
            item.high,
            item.low,
            item.close,
            item.volume,
            item.trade_value,
            item.symbol,
            item.market,
            item.name,
            item.snapshot_date
        ));
    }
    fs::write(&file_path, contents).map_err(|err| err.to_string())?;
    Ok(KrxSnapshotSymbolReport {
        symbol: row.symbol.clone(),
        name: row.name.clone(),
        market: row.market.clone(),
        output_path: file_path.display().to_string(),
        rows_written: 1,
        total_rows_after_merge: rows.len(),
        reason_codes: vec![ReasonCode::SnapshotStoreWritten],
    })
}

fn load_existing_canonical_rows(
    path: &Path,
) -> Result<BTreeMap<u64, KrxSnapshotCanonicalRow>, String> {
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let _header = lines.next();
    let mut rows = BTreeMap::new();
    for line in lines {
        let fields = parse_csv_record(line);
        if fields.len() < 11 {
            continue;
        }
        let row = KrxSnapshotCanonicalRow {
            timestamp_ms: fields[0].parse::<u64>().map_err(|err| err.to_string())?,
            open: fields[1].parse::<f64>().map_err(|err| err.to_string())?,
            high: fields[2].parse::<f64>().map_err(|err| err.to_string())?,
            low: fields[3].parse::<f64>().map_err(|err| err.to_string())?,
            close: fields[4].parse::<f64>().map_err(|err| err.to_string())?,
            volume: fields[5].parse::<f64>().map_err(|err| err.to_string())?,
            trade_value: fields[6].parse::<f64>().map_err(|err| err.to_string())?,
            symbol: fields[7].clone(),
            market: fields[8].clone(),
            name: fields[9].clone(),
            snapshot_date: fields[10].clone(),
        };
        rows.insert(row.timestamp_ms, row);
    }
    Ok(rows)
}

fn field<'a>(
    values: &'a [String],
    header_index: &BTreeMap<String, usize>,
    header: &str,
) -> Result<&'a str, String> {
    let index = header_index
        .get(header)
        .copied()
        .ok_or_else(|| format!("missing header index for {header}"))?;
    values
        .get(index)
        .map(|value| value.trim())
        .ok_or_else(|| format!("missing field for {header}"))
}

fn required_snapshot_columns() -> [&'static str; 8] {
    [
        "종목코드",
        "종목명",
        "시장구분",
        "종가",
        "시가",
        "고가",
        "저가",
        "거래량",
    ]
}

fn parse_csv_record(line: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                if in_quotes && chars.peek() == Some(&'"') {
                    current.push('"');
                    let _ = chars.next();
                } else {
                    in_quotes = !in_quotes;
                }
            }
            ',' if !in_quotes => {
                values.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    values.push(current.trim().to_string());
    values
}

fn normalize_header(value: &str) -> String {
    value.trim().to_string()
}

fn normalize_symbol(value: &str) -> String {
    value.trim().trim_matches('"').to_string()
}

fn parse_krx_number(value: &str) -> Result<f64, String> {
    let normalized = value
        .trim()
        .trim_matches('"')
        .replace(',', "")
        .replace('_', "");
    if normalized.is_empty() {
        Ok(0.0)
    } else {
        normalized.parse::<f64>().map_err(|err| err.to_string())
    }
}

fn infer_snapshot_date_from_path(path: &Path) -> Option<String> {
    let filename = path.file_name()?.to_string_lossy();
    let digits = filename.chars().collect::<Vec<_>>();
    for window in digits.windows(8) {
        if window.iter().all(|ch| ch.is_ascii_digit()) {
            let candidate = window.iter().collect::<String>();
            if validate_snapshot_date(&candidate).is_ok() {
                return Some(candidate);
            }
        }
    }
    None
}

fn validate_snapshot_date(value: &str) -> Result<(), String> {
    if value.len() != 8 || !value.chars().all(|ch| ch.is_ascii_digit()) {
        return Err("snapshot date must be YYYYMMDD".to_string());
    }
    let year = value[0..4].parse::<i32>().map_err(|err| err.to_string())?;
    let month = value[4..6].parse::<u32>().map_err(|err| err.to_string())?;
    let day = value[6..8].parse::<u32>().map_err(|err| err.to_string())?;
    if year < 1970 || !(1..=12).contains(&month) || day == 0 || day > days_in_month(year, month) {
        return Err("snapshot date must be a valid calendar day".to_string());
    }
    Ok(())
}

fn yyyymmdd_to_unix_ms(value: &str) -> Result<u64, String> {
    validate_snapshot_date(value)?;
    let year = value[0..4].parse::<i32>().map_err(|err| err.to_string())?;
    let month = value[4..6].parse::<u32>().map_err(|err| err.to_string())?;
    let day = value[6..8].parse::<u32>().map_err(|err| err.to_string())?;
    let days = days_from_civil(year, month, day);
    let millis = days
        .checked_mul(86_400_000)
        .ok_or_else(|| "snapshot timestamp overflow".to_string())?;
    u64::try_from(millis).map_err(|_| "snapshot timestamp overflow".to_string())
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

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
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

pub fn default_import_report_dir(output_root: &str) -> PathBuf {
    PathBuf::from(output_root)
}
