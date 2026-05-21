#!/usr/bin/env python3
import argparse
import csv
import datetime as dt
import json
import os
import pathlib
import sys

try:
    import tomllib
except ModuleNotFoundError:
    tomllib = None


ALLOWED_INTERVALS = {"1d", "1h", "1m", "5m"}
ALLOWED_PERIODS = {"5d", "1mo", "3mo", "6mo", "1y"}
MAX_TICKERS = 5
MAX_ROWS_PER_TICKER = 5000


def parse_args():
    parser = argparse.ArgumentParser(
        description="Research-only yfinance adapter. Writes bounded local canonical CSV files only."
    )
    parser.add_argument("--config")
    parser.add_argument("--fixture")
    parser.add_argument("--out")
    parser.add_argument("--tickers")
    parser.add_argument("--interval")
    parser.add_argument("--period")
    parser.add_argument("--start")
    parser.add_argument("--end")
    parser.add_argument("--adjusted-price-policy", default="adjusted")
    parser.add_argument("--run-preflight", action="store_true")
    return parser.parse_args()


def load_config(args):
    config = {
        "tickers": [],
        "interval": "1d",
        "period": "1mo",
        "fixture": args.fixture,
        "out": args.out,
        "start": args.start,
        "end": args.end,
        "adjusted_price_policy": args.adjusted_price_policy,
        "run_preflight": args.run_preflight,
    }
    if args.config:
        if "://" in args.config:
            raise SystemExit("config path must be local")
        if tomllib is None:
            loaded = load_simple_toml(args.config)
        else:
            with open(args.config, "rb") as fh:
                loaded = tomllib.load(fh)
        config.update(loaded)
    if args.tickers:
        config["tickers"] = [value.strip() for value in args.tickers.split(",") if value.strip()]
    return config


def validate_config(config):
    if not config.get("out") or "://" in config["out"]:
        raise SystemExit("out path must be local")
    if config.get("fixture") and "://" in config["fixture"]:
        raise SystemExit("fixture path must be local")
    if config.get("interval", "1d") not in ALLOWED_INTERVALS:
        raise SystemExit("unsupported interval")
    if config.get("period", "1mo") not in ALLOWED_PERIODS:
        raise SystemExit("unsupported period")
    tickers = config.get("tickers") or []
    if len(tickers) == 0:
        raise SystemExit("at least one ticker is required")
    if len(tickers) > MAX_TICKERS:
        raise SystemExit("too many tickers requested")


def load_simple_toml(path):
    result = {}
    with open(path, "r", encoding="utf-8") as fh:
        for raw_line in fh:
            line = raw_line.strip()
            if not line or line.startswith("#"):
                continue
            if "=" not in line:
                continue
            key, value = [part.strip() for part in line.split("=", 1)]
            if value.startswith("[") and value.endswith("]"):
                inner = value[1:-1].strip()
                if not inner:
                    result[key] = []
                else:
                    result[key] = [
                        part.strip().strip('"').strip("'")
                        for part in inner.split(",")
                        if part.strip()
                    ]
            elif value.lower() in {"true", "false"}:
                result[key] = value.lower() == "true"
            else:
                result[key] = value.strip('"').strip("'")
    return result


def parse_timestamp(value):
    value = value.strip()
    if len(value) == 10 and value[4] == "-" and value[7] == "-":
        return int(dt.datetime.fromisoformat(value).replace(tzinfo=dt.timezone.utc).timestamp() * 1000)
    if value.endswith("Z"):
        return int(dt.datetime.fromisoformat(value.replace("Z", "+00:00")).timestamp() * 1000)
    return int(dt.datetime.fromisoformat(value).replace(tzinfo=dt.timezone.utc).timestamp() * 1000)


def load_fixture_rows(config):
    fixture = config.get("fixture")
    if not fixture:
        return fetch_live_rows(config)
    rows = []
    with open(fixture, newline="", encoding="utf-8") as fh:
        reader = csv.DictReader(fh)
        for row in reader:
            ticker = row.get("Ticker") or row.get("ticker") or (config["tickers"][0] if len(config["tickers"]) == 1 else "")
            if ticker not in config["tickers"]:
                continue
            rows.append(
                {
                    "ticker": ticker,
                    "timestamp_ms": parse_timestamp(row["Date"]),
                    "open": row["Open"],
                    "high": row["High"],
                    "low": row["Low"],
                    "close": row["Close"],
                    "volume": row["Volume"],
                }
            )
    return rows


def fetch_live_rows(config):
    try:
        import yfinance as yf
    except ModuleNotFoundError:
        raise SystemExit("live mode requires yfinance; fixture mode does not")
    rows = []
    for ticker in config["tickers"]:
        history = yf.Ticker(ticker).history(
            interval=config.get("interval", "1d"),
            period=config.get("period", "1mo"),
            start=config.get("start"),
            end=config.get("end"),
            auto_adjust=(config.get("adjusted_price_policy", "adjusted") == "adjusted"),
        )
        if history.empty:
            continue
        for index, item in history.iterrows():
            timestamp_ms = int(index.to_pydatetime().replace(tzinfo=dt.timezone.utc).timestamp() * 1000)
            rows.append(
                {
                    "ticker": ticker,
                    "timestamp_ms": timestamp_ms,
                    "open": str(item["Open"]),
                    "high": str(item["High"]),
                    "low": str(item["Low"]),
                    "close": str(item["Close"]),
                    "volume": str(item["Volume"]),
                }
            )
    return rows


def write_outputs(config, rows):
    out_dir = pathlib.Path(config["out"])
    canonical_dir = out_dir / "canonical"
    provenance_dir = out_dir / "provenance"
    manifest_dir = out_dir / "manifests"
    canonical_dir.mkdir(parents=True, exist_ok=True)
    provenance_dir.mkdir(parents=True, exist_ok=True)
    manifest_dir.mkdir(parents=True, exist_ok=True)

    grouped = {}
    for row in sorted(rows, key=lambda item: (item["ticker"], item["timestamp_ms"])):
        grouped.setdefault(row["ticker"], [])
        grouped[row["ticker"]].append(row)

    reports = []
    for ticker, ticker_rows in grouped.items():
        ticker_rows = ticker_rows[:MAX_ROWS_PER_TICKER]
        stem = f"{ticker.lower()}_{config.get('interval', '1d')}"
        canonical_path = canonical_dir / f"{stem}.csv"
        provenance_path = provenance_dir / f"{stem}.provenance.json"
        manifest_path = manifest_dir / f"{stem}.manifest.json"

        with open(canonical_path, "w", newline="", encoding="utf-8") as fh:
            writer = csv.writer(fh)
            writer.writerow(["timestamp_ms", "open", "high", "low", "close", "volume"])
            for row in ticker_rows:
                writer.writerow(
                    [
                        row["timestamp_ms"],
                        row["open"],
                        row["high"],
                        row["low"],
                        row["close"],
                        row["volume"],
                    ]
                )

        provenance = {
            "source_kind": "YFinanceResearch",
            "source_label": f"yfinance-{ticker.lower()}-{config.get('interval', '1d')}",
            "provider_label": "yfinance",
            "upstream_label": "Yahoo Finance",
            "local_path": str(canonical_path),
            "generated_by": "research/yfinance_fetch.py",
            "user_supplied": True,
            "downloaded_by_soma": False,
            "remote_url_present": False,
            "official_provider": False,
            "affiliated_or_endorsed": False,
            "intended_use": "research-only unofficial supplemental benchmark data",
            "readiness_eligible": False,
            "benchmark_eligible": len(ticker_rows) >= 20,
            "license_note": "User must verify Yahoo Finance and yfinance usage/licensing before use.",
            "notes": "Research-only yfinance fixture/live adapter output.",
            "reason_codes": ["YFinanceCanonicalized", "YFinanceUnofficialEvidence"],
        }
        with open(provenance_path, "w", encoding="utf-8") as fh:
            json.dump(provenance, fh, indent=2, sort_keys=True)

        manifest = {
            "manifest_version": 1,
            "source_kind": "YFinanceResearch",
            "provider_label": "yfinance",
            "upstream_label": "Yahoo Finance",
            "symbol": ticker,
            "interval": config.get("interval", "1d"),
            "row_count": len(ticker_rows),
            "first_timestamp_ms": ticker_rows[0]["timestamp_ms"] if ticker_rows else 0,
            "last_timestamp_ms": ticker_rows[-1]["timestamp_ms"] if ticker_rows else 0,
            "adjusted_price_policy": config.get("adjusted_price_policy", "adjusted"),
            "corporate_action_adjusted": config.get("adjusted_price_policy", "adjusted") == "adjusted",
            "canonical_csv": str(canonical_path),
            "provenance_path": str(provenance_path),
            "readiness_eligible": False,
            "benchmark_eligible": len(ticker_rows) >= 20,
            "reason_codes": ["YFinanceCanonicalized", "YFinanceUnofficialEvidence"],
        }
        with open(manifest_path, "w", encoding="utf-8") as fh:
            json.dump(manifest, fh, indent=2, sort_keys=True)

        reports.append(
            {
                "ticker": ticker,
                "canonical_csv": str(canonical_path),
                "provenance_path": str(provenance_path),
                "manifest_path": str(manifest_path),
                "row_count": len(ticker_rows),
            }
        )

    if config.get("run_preflight"):
        cmd = "soma-experiment yfinance-import --config <generated-toml>"
        print(f"suggested_preflight_command={cmd}")
    print(json.dumps({"reports": reports}, indent=2, sort_keys=True))


def main():
    args = parse_args()
    config = load_config(args)
    validate_config(config)
    rows = load_fixture_rows(config)
    if not rows:
        raise SystemExit("no rows found for requested tickers")
    write_outputs(config, rows)


if __name__ == "__main__":
    main()
