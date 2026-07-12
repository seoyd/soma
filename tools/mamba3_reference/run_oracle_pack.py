#!/usr/bin/env python3
"""Run selected tiny official Mamba-3 oracle cases after pre-flight succeeds."""

import argparse
import os
from pathlib import Path

from generate_oracle import CASE_IDS, CASE_SEEDS, fail, reject_unsafe_output_path, require_environment, write_fixture
from verify_environment import preflight


def parse_cases(value: str) -> list[str]:
    cases = [case.strip().upper() for case in value.split(",") if case.strip()]
    if not cases or any(case not in CASE_IDS for case in cases) or len(set(cases)) != len(cases):
        raise argparse.ArgumentTypeError("cases must be a unique comma-separated subset of A,B,C,D,E")
    return cases


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--reference-root", default=os.environ.get("SOMA_MAMBA_OFFICIAL_DIR"))
    parser.add_argument("--output-dir", default=os.environ.get("SOMA_MAMBA_ORACLE_OUT", "target/mamba3_oracle"))
    parser.add_argument("--cases", type=parse_cases, default=list(CASE_IDS))
    parser.add_argument("--device", default=os.environ.get("SOMA_MAMBA_ORACLE_DEVICE", "cuda:0"))
    parser.add_argument("--dtype", default=os.environ.get("SOMA_MAMBA_ORACLE_DTYPE", "float32"))
    parser.add_argument("--seed", type=int)
    parser.add_argument("--instrumentation-patch")
    parser.add_argument("--overwrite", action="store_true")
    args = parser.parse_args()
    if not args.reference_root:
        fail("pass --reference-root or set SOMA_MAMBA_OFFICIAL_DIR")
    if args.instrumentation_patch:
        fail("instrumentation patches are not supported; the official cache API already exposes required state")
    reference_root = Path(args.reference_root)
    output_dir = Path(args.output_dir)
    report = preflight(reference_root, output_dir, args.device, args.dtype)
    if report["status"] != "ReadyF32":
        fail(f"pre-flight did not permit F32 fixture generation: {report['status']}: {report['detail']}")
    torch, mamba3 = require_environment(reference_root, args.device, args.dtype)
    for case_index, case_id in enumerate(args.cases):
        output_path = output_dir / f"official_siso_reference_case_{case_id.lower()}.json"
        reject_unsafe_output_path(output_path)
        if output_path.exists() and not args.overwrite:
            fail(f"fixture already exists for case {case_id}; pass --overwrite to replace it")
        seed = CASE_SEEDS[case_id] if args.seed is None else args.seed + case_index
        write_fixture(output_path, reference_root, torch, mamba3, case_id, args.device, seed, None)


if __name__ == "__main__":
    main()
